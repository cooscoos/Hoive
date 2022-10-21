use std::error::Error;
use std::time::{Duration, Instant};
use std::usize;

use actix::prelude::*;
use actix_web_actors::ws;
use actix_web_actors::ws::WebsocketContext;
use hoive::game::actions::BoardAction;
use hoive::game::comps::convert_static_basic;
use hoive::game::movestatus::MoveStatus;
use hoive::maths::coord::DoubleHeight;
use hoive::pmoore;

use crate::api;
use crate::chat_server;
use rustrict::CensorStr;

/// How often heartbeat pings are sent
const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);

/// How long before lack of client response causes a timeout
const CLIENT_TIMEOUT: Duration = Duration::from_secs(10);

#[derive(Debug, Clone)]
pub struct WsChatSession {
    /// unique client session id (User id in db)
    pub id: usize,

    /// Client must send ping at least once per 10 seconds (CLIENT_TIMEOUT),
    /// otherwise we drop connection.
    pub hb: Instant,

    /// joined game (game_state id in the db)
    pub game_room: String,

    /// peer name
    pub name: Option<String>,

    /// Whether the player is actively taking a turn
    pub active: bool,

    /// Command list for executing a move
    pub cmdlist: BoardAction,

    /// Chat server
    pub addr: Addr<chat_server::ChatServer>,
}

impl WsChatSession {
    /// helper method that sends ping to client every 5 seconds (HEARTBEAT_INTERVAL).
    ///
    /// also this method checks heartbeats from client
    fn hb(&self, ctx: &mut ws::WebsocketContext<Self>) {
        ctx.run_interval(HEARTBEAT_INTERVAL, |act, ctx| {
            // check client heartbeats
            if Instant::now().duration_since(act.hb) > CLIENT_TIMEOUT {
                // heartbeat timed out
                println!("Websocket Client heartbeat failed, disconnecting!");

                // notify chat server
                act.addr.do_send(chat_server::Disconnect {
                    id: act.id,
                    name: act.name.clone(),
                });

                // stop actor
                ctx.stop();

                // don't try to send a ping
                return;
            }

            ctx.ping(b"");
        });
    }
}

impl Actor for WsChatSession {
    type Context = ws::WebsocketContext<Self>;

    /// Method is called on actor start.
    /// We register ws session with ChatServer
    fn started(&mut self, ctx: &mut Self::Context) {
        // we'll start heartbeat process on session start.
        self.hb(ctx);

        // Default name is just your randomly generated id
        let namey = self.id.to_string();

        // register self in chat server. `AsyncContext::wait` register
        // future within context, but context waits until this future resolves
        // before processing any other events.
        // HttpContext::state() is instance of WsChatSessionState, state is shared
        // across all routes within application
        let addr = ctx.address();
        self.addr
            .send(chat_server::Connect {
                addr: addr.recipient(),
                name: Some(namey),
            })
            .into_actor(self)
            .then(|res, act, ctx| {
                match res {
                    Ok(res) => act.id = res,
                    // something is wrong with chat server
                    _ => ctx.stop(),
                }
                fut::ready(())
            })
            .wait(ctx);
    }

    fn stopping(&mut self, _: &mut Self::Context) -> Running {
        // notify chat server
        self.addr.do_send(chat_server::Disconnect {
            id: self.id,
            name: self.name.clone(),
        });
        Running::Stop
    }
}

/// Handle messages from chat server, we simply send it to peer websocket
impl Handler<chat_server::Message> for WsChatSession {
    type Result = ();

    fn handle(&mut self, msg: chat_server::Message, ctx: &mut Self::Context) {
        ctx.text(msg.0);
    }
}

/// WebSocket message handler
impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for WsChatSession {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        let msg = match msg {
            Err(_) => {
                ctx.stop();
                return;
            }
            Ok(msg) => msg,
        };

        log::info!("WEBSOCKET MESSAGE: {msg:?}");
        match msg {
            ws::Message::Ping(msg) => {
                self.hb = Instant::now();
                ctx.pong(&msg);
            }
            ws::Message::Pong(_) => {
                self.hb = Instant::now();
            }
            ws::Message::Text(text) => {
                let result = match self.game_room == "main" {
                    true => main_lobby_parser(self, text.to_string(), ctx),
                    false => in_game_parser(self, text.to_string(), ctx),
                };

                match result {
                    Ok(()) => {}
                    Err(err) => ctx.text(format!("Error: {err}")),
                }
            }
            ws::Message::Binary(_) => println!("Unexpected binary"),
            ws::Message::Close(reason) => {
                ctx.close(reason);
                ctx.stop();
            }
            ws::Message::Continuation(_) => {
                ctx.stop();
            }
            ws::Message::Nop => (),
        }
    }
}

/// Parses user inputs when they're typed in the main lobby
fn main_lobby_parser(
    chatsess: &mut WsChatSession,
    text: String,
    ctx: &mut WebsocketContext<WsChatSession>,
) -> Result<(), Box<dyn Error>> {
    // Don't do anything if user hits enter. This should be caught and prevented at the client end anyway.
    if text == "\n" {
        return Ok(());
    }

    let m = text.trim();
    // we check for /sss type of messages
    if m.starts_with('/') {
        let v: Vec<&str> = m.splitn(2, ' ').collect();

        if chatsess.name.is_none() && v[0] != "/name" {
            ctx.text("Define a username before chatting. Type your username below:");
            return Ok(());
        }

        match v[0] {
            "/join" => {
                if v.len() == 2 {
                    // Check the db to see if there's a session with this id
                    //let session_id = v[1].to_owned();
                    // no function to do this yet, create one later
                    ctx.text("Joining specific games is unimplemented. Just type /join");
                } else {
                    // Join an empty game if there is one available
                    match api::find()? {
                        Some(game_state) => {
                            // Join the game
                            let session_id = game_state.id.to_owned();

                            // Join on the db
                            api::join(&session_id, &chatsess.id)?;

                            // Join in the chat
                            chatsess.game_room = session_id.to_owned();
                            chatsess.addr.do_send(chat_server::Join {
                                id: chatsess.id,
                                name: chatsess.game_room.clone(),
                                username: chatsess.name.as_ref().unwrap().to_owned(),
                            });

                            ctx.text(format!("You joined game room {}", session_id));

                            let game_state = api::get_game_state(&session_id)?;

                            // send a new game command to everyone in the game room
                            // and define the team colours
                            chatsess.addr.do_send(chat_server::NewGame {
                                session_id,
                                game_state,
                            });
                        }
                        None => ctx.text("No empty games available. Try /create one!"),
                    }
                }
            }
            "/name" => {
                if let Some(name) = &chatsess.name {
                    ctx.text(format!("You already have the name {name}!"));
                } else if v.len() != 2 {
                    ctx.text("You need to input a name!");
                } else if v[1].is_inappropriate() || v[1].starts_with('/') {
                    // Filter profanity and usernames that start with /
                    ctx.text("Invalid username.");
                } else {
                    // Try register the username on the game db.
                    let user_name = v[1];
                    match api::register_user(user_name, chatsess.id)? {
                        false => ctx.text("Username already exists. Pick another."),
                        true => {
                            // Assign username in the chat session
                            chatsess.name = Some(user_name.to_owned());

                            // Update the chat session's visitor list
                            chatsess.addr.do_send(chat_server::NewName {
                                name: user_name.to_owned(),
                                id: chatsess.id,
                            });

                            ctx.text(format!("//cmd yourid {}", chatsess.id));
                            ctx.text(format!("Welcome {}. Begin typing to chat.", user_name));
                            // reset the client's precursor
                            ctx.text("//cmd default");
                        }
                    }
                }
            }
            "/wipe" => {
                // For debug
                match api::delete_all() {
                    Ok(_) => ctx.text("Database wiped"),
                    Err(err) => panic!("Error {}", err),
                };
            }
            "/id" => {
                // Display info to user on themselves
                ctx.text(format!(
                    "Your user id is: {}, and username is {:?}. You're in game_session: {}",
                    chatsess.id, chatsess.name, chatsess.game_room
                ));
            }
            "/who" => {
                // Display who is online
                chatsess
                    .addr
                    .send(chat_server::Who {})
                    .into_actor(chatsess)
                    .then(|res, _, ctx| {
                        match res {
                            Ok(res) => ctx.text(res),
                            _ => ctx.stop(),
                        }
                        fut::ready(())
                    })
                    .wait(ctx);
            }
            "/create" => {
                // Create a new game on the db, register creator as user_1
                let session_id = api::new_game(&chatsess.id)?;

                // Join the game session's chat room
                chatsess.game_room = session_id.to_owned();
                chatsess.addr.do_send(chat_server::Join {
                    id: chatsess.id,
                    name: chatsess.game_room.clone(),
                    username: chatsess.name.as_ref().unwrap().to_owned(),
                });
                ctx.text(format!("You joined game room {}", session_id));
            }
            _ => ctx.text(format!("!!! unknown command: {m:?}")),
        }
    } else {
        let msg = format!("\x1b[36;2m{}:\x1b[0m {m}", &chatsess.name.as_ref().unwrap());

        // send message to chat server
        chatsess.addr.do_send(chat_server::ClientMessage {
            id: chatsess.id,
            msg,
            room: chatsess.game_room.clone(),
        })
    }

    Ok(())
}

/// Parses user inputs when they're typed in game
fn in_game_parser(
    chatsess: &mut WsChatSession,
    text: String,
    ctx: &mut WebsocketContext<WsChatSession>,
) -> Result<(), Box<dyn Error>> {
    // Don't do anything if user hits enter. This should be caught and prevented at the client end anyway.
    // if text == "\n" {
    //     return Ok(());
    // }

    let m = text.trim();
    // we check for /sss type of messages
    if m.starts_with('/') {
        let v: Vec<&str> = m.splitn(2, ' ').collect();
        match v[0] {
            // "/join" => {
            //     if v.len() == 2 {
            //         let session_id = v[1].to_owned();
            //         // Check the db to see if there's a session with this id
            //         // no function to do this yet, create one later

            //         // If there's a match, then join the session, and join the chat for that room
            //         return Ok(());
            //     } else {
            //         // Join an empty game if there is one available
            //         match api::find()? {
            //             Some(game_state) => {
            //                 // Join the game
            //                 let session_id = game_state.id.to_owned();

            //                 // Join on the db
            //                 api::join(&session_id, &chatsess.id)?;

            //                 // Join in the chat
            //                 chatsess.game_room = session_id.to_owned();
            //                 chatsess.addr.do_send(chat_server::Join {
            //                     id: chatsess.id,
            //                     name: chatsess.game_room.clone(),
            //                     username: chatsess.name.as_ref().unwrap().to_owned(),
            //                 });

            //                 ctx.text(format!("You joined game room {}", session_id));
            //             }
            //             None => ctx.text("No empty games available. Try /create one!"),
            //         }
            //     }
            // }
            // "/leave" => {
            // create an api to remove self from session, lose game, join main etc.
            // api::remove
            // }
            "/id" => {
                // Display info to user on themselves
                ctx.text(format!(
                    "Your user id is: {}, and username is {:?}. You're in game_session: {}",
                    chatsess.id, chatsess.name, chatsess.game_room
                ));
            }
            "/who" => {
                // Display who is online
                chatsess
                    .addr
                    .send(chat_server::Who {})
                    .into_actor(chatsess)
                    .then(|res, _, ctx| {
                        match res {
                            Ok(res) => ctx.text(res),
                            _ => ctx.stop(),
                        }
                        fut::ready(())
                    })
                    .wait(ctx);
            }
            "/t" | "/tell" => {
                let words = v[1];
                let msg = format!(
                    "\x1b[36;2m{}:\x1b[0m {words}",
                    &chatsess.name.as_ref().unwrap()
                );
                // send message to chat server
                chatsess.addr.do_send(chat_server::ClientMessage {
                    id: chatsess.id,
                    msg,
                    room: chatsess.game_room.clone(),
                })
            }
            "/help" => {
                ctx.text(pmoore::help_me());
            }
            "/xylophone" => {
                ctx.text(pmoore::xylophone());
            }
            "/quit" => {}
            "/play" => {
                // This should auto happen later.
                // If this is our first rodeo then we're going to check if the player is the active player
                // Get the gamestate and make sure it's this player's turn
                let gamestate = api::get_game_state(&chatsess.game_room)?;
                if chatsess.id.to_string() != gamestate.last_user_id.unwrap() {
                    // Player is initiating a play, so prompt them to select a chip.
                    ctx.text("Select a tile from the board or your hand to move.");
                    // set the player's active state in the chat struct to true. This reduces how often we have to query the db.
                    // It'll get set back to false later.
                    chatsess.active = true;
                    // tell the client to get into a select state
                    //ctx.text("//cmd select");
                } else {
                    ctx.text("It's not your turn");
                }
            }
            "/select" if chatsess.active => {

                    // Go ahead
                    let textin = v[1].to_owned();
                    ctx.text(format!("You're selecting {textin}"));

                    // Empty. Stage 0, we should be fed a chip name, or a skip request
                    let chip_name = match textin {
                        _ if textin == "w" => {
                            // Atempt to skip turn, return db response
                            None
                        }
                        _ if textin == "mb" => {
                            // The player is probably trying to select their mosquito acting like a beetle
                            convert_static_basic("m1".to_string())
                        }
                        _ if textin.contains('*') => {
                            // The player is probably trying to select a beetle (or a mosquito acting like one).
                            // Grab the first 2 chars of the string
                            let (mut first, _) = textin.split_at(2);

                            // If the first two chars are mosquito, convert to m1
                            if first.contains('m') {
                                first = "m1";
                            }
                            convert_static_basic(first.to_string())
                        }
                        _ if textin
                            .starts_with(|c| c == 'l' || c == 'p' || c == 'q' || c == 'm') =>
                        {
                            let proper_str = match textin.chars().next().unwrap() {
                                'l' => "l1",
                                'p' => "p1",
                                'q' => "q1",
                                'm' => "m1",
                                _ => panic!("unreachable"),
                            };
                            convert_static_basic(proper_str.to_string())
                        }
                        c => {
                            // Try and match a chip by this name
                            let chip_str = convert_static_basic(c);

                            match chip_str.is_some() {
                                true => chip_str,
                                false => {
                                    println!("You don't have this tile in your hand.");
                                    None
                                }
                            }
                        }
                    };

                    // Stage 0. We're expecting a chip name, try find a valid chip on the board and pass back a response
                    // for the user and for the client program. E.g. if it's a pillbug, provide guidance on what to do next.

                    match chip_name {
                        Some(value) if value == "p1" => {
                            chatsess.cmdlist.name = value.to_string();
                            ctx.text("Hit m to sumo a neighbour, or select co-ordinate to move to. If moving, input column then row, separated by comma, e.g.: 0, 0. Hit x to abort the move.");
                        
                            ctx.text("//cmd moveto");
                        }
                        Some(value) if value == "m1" => {
                            chatsess.cmdlist.name = value.to_string();
                            ctx.text("Hit m to suck a neighbour now, here is some neighbours to choose from:");
                            
                            ctx.text("//cmd moveto");
                        }
                        None => {
                            // Repeat yourself
                            ctx.text("Select a tile from the board or your hand to move.");
                        }
                        Some(value) => {
                            chatsess.cmdlist.name = value.to_string();
                            ctx.text("Select co-ordinate to move to. Input column then row, separated by comma, e.g.: 0, 0. Hit x to abort the move.");
                            ctx.text("//cmd moveto");
                        }
                    }
                
            }
            "/moveto" if chatsess.active => {

                // We're expect comma separated values to doubleheight or the letter m to enter special state

                let textin = v[1].to_owned();
                println!("recieved moveto: {textin}");

                if textin == "m" {
                        ctx.text("Select a neighbour to special from the following choices...");
                        ctx.text("//cmd special");
                } else {
                    //attempt to parse a move
                    let usr_hex = pmoore::coord_from_string(textin);
                    println!("user hex = {:?}", usr_hex);
                    
                    chatsess.cmdlist.rowcol = match usr_hex[..] {
                        [Some(x), Some(y)] => {
                            match (x + y) % 2 {
                                // The sum of doubleheight coords should always be an even no.
                                0 => {
                                    // Tell the client to tell us to execute
                                    
                                    Some(DoubleHeight::from((x, y)))
         
                                },
                                _ => {
                                    ctx.text("Invalid co-ordinates, try again or hit x to abort.");
                                    // don't update the client state
                                    None
                                }
                            }
                        }
                        _ => {
                            ctx.text("Try again: enter two numbers separated by a comma or hit x to abort.");
                            None
                        }
                    };

                    if chatsess.cmdlist.rowcol.is_some(){
                        ctx.text("//cmd execute");
                    }

                
            }
        }
            "/execute" if chatsess.active => {
                

                    let board_action = &chatsess.cmdlist;

                    let result = api::make_action(board_action,&chatsess.game_room)?;

                    match result {
                        MoveStatus::Success => {
                            // no longer this player's turn
                            chatsess.active = false;
                            chatsess.cmdlist = BoardAction::default();

                            // update all player gamestates
                            let session_id = chatsess.game_room.to_owned();
                            let game_state = api::get_game_state(&session_id)?;

                            // Tell all players the gamestate has updated
                            chatsess.addr.do_send(chat_server::UpdateGame {
                                session_id,
                                game_state,
                            });


                        }
                        _ => {
                            // Get the client back into the select phase
                            ctx.text("//cmd select");
                        },
                    }

                    ctx.text(result.to_string());
               


                

            }
            "/abort" if chatsess.active => {
                // Abort the move go back into select phase
                ctx.text("Aborting move. Select a chip.");
                chatsess.cmdlist = BoardAction::default();
                ctx.text("//cmd select");
            }
            "/mosquito" if chatsess.active => {
                // Do a mosquito suck

            }
            _ => ctx.text(format!("Invalid command.")),
        }
    } else {
        // Default is off in game
        ctx.text("Normal chat is off during games. Use /tell or /t to talk to the other player");
    }

    Ok(())
}
