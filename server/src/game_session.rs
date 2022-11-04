//! Module to create and define the behaviour of a client game session (WsGameSession)
//!
use std::error::Error;
use std::time::{Duration, Instant};
use std::usize;

use actix::prelude::*;
use actix_web_actors::ws;
use actix_web_actors::ws::WebsocketContext;

use hoive::game::movestatus::MoveStatus;
use rustrict::CensorStr;

use crate::api;
use crate::game_server;

use hoive::game::{
    actions::BoardAction,
    board::Board,
    comps::Team};
use hoive::maths::coord::Cube;
use hoive::pmoore;

/// How often heartbeat pings are sent to server
const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);

/// How long before lack of client response causes a timeout
const CLIENT_TIMEOUT: Duration = Duration::from_secs(10);

/// WsGameSession: the websocket client session
#[derive(Debug, Clone)]
pub struct WsGameSession {
    /// unique client session id (mirrors the user_id in the sqlite db)
    pub id: usize,

    /// Client must ping once per CLIENT_TIMEOUT seconds, or get dropped
    pub hb: Instant,

    /// Chat server
    pub addr: Addr<game_server::GameServer>,

    /// Joined room, ("main" or game_state id, mirrored in sqlite db)
    pub room: String,

    /// Username
    pub name: Option<String>,

    /// In-game: Is it the client's turn in game?
    pub active: bool,

    /// In-game: Actions used to execute moves in Hoive games
    pub action: BoardAction,

    /// In-game: The current board
    pub board: Board<Cube>,

    /// In-game: What team the player is on
    pub team: Team,
}

impl WsGameSession {
    /// Send ping to client every HEARTBEAT_INTERVAL, and check heartbeat from client
    fn hb(&self, ctx: &mut ws::WebsocketContext<Self>) {
        ctx.run_interval(HEARTBEAT_INTERVAL, |act, ctx| {
            // Check client heartbeats
            if Instant::now().duration_since(act.hb) > CLIENT_TIMEOUT {
                // Heartbeat timed out
                println!("Websocket Client heartbeat failed, disconnecting!");

                // Notify chat server
                act.addr.do_send(game_server::Disconnect {
                    id: act.id,
                    name: act.name.clone(),
                });

                // Stop actor
                ctx.stop();

                // Don't try to send a ping
                return;
            }

            ctx.ping(b"");
        });
    }
}

impl Actor for WsGameSession {
    type Context = ws::WebsocketContext<Self>;

    /// On actor start: register websocket session with GameServer
    fn started(&mut self, ctx: &mut Self::Context) {
        // Start heartbeat process
        self.hb(ctx);

        // For now, default username is the same as the randomly generated user id
        // The session user will be asked to change it before they can do anything.
        let def_name = self.id.to_string();

        // Register self in chat server. `AsyncContext::wait` register
        // future within context, but context waits until this future resolves
        // before processing any other events.
        // HttpContext::state() is instance of WsGameSessionState, state is shared
        // across all routes within application.
        let addr = ctx.address();
        self.addr
            .send(game_server::Connect {
                addr: addr.recipient(),
                name: Some(def_name),
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

    /// On actor stop: disconnect session.
    fn stopping(&mut self, _: &mut Self::Context) -> Running {
        // Notify chat server
        self.addr.do_send(game_server::Disconnect {
            id: self.id,
            name: self.name.clone(),
        });
        Running::Stop
    }
}

/// Handle messages from the chat server: simply send them to the peer websocket.
impl Handler<game_server::Message> for WsGameSession {
    type Result = ();

    fn handle(&mut self, msg: game_server::Message, ctx: &mut Self::Context) {
        ctx.text(msg.0);
    }
}

/// WebSocket message handler: how is text received from a client handled by WsGameSession?
impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for WsGameSession {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        // Get the content of the msg
        let msg = match msg {
            Err(_) => {
                ctx.stop();
                return;
            }
            Ok(msg) => msg,
        };

        //log::info!("WEBSOCKET MESSAGE: {msg:?}");

        match msg {
            ws::Message::Ping(msg) => {
                // If ping, send back pong
                self.hb = Instant::now();
                ctx.pong(&msg);
            }
            ws::Message::Pong(_) => {
                // If pong, reset the clock
                self.hb = Instant::now();
            }
            ws::Message::Text(text) => {
                // Response to other messages depends on whether client is in main lobby or in-game
                let result = match self.room == "main" {
                    true => main_lobby_parser(self, text.to_string(), ctx),
                    false => in_game_parser(self, text.to_string(), ctx),
                };

                match result {
                    Ok(()) => {}
                    Err(err) => ctx.text(format!("Error: {err}")),
                }
            }
            // Handle other possible inputs, like binary, requests to close sessions.
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

/// Parse user inputs when they're typed in the main lobby
fn main_lobby_parser(
    gamesess: &mut WsGameSession,
    text: String,
    ctx: &mut WebsocketContext<WsGameSession>,
) -> Result<(), Box<dyn Error>> {
    //  This should be caught by the user's local client: but don't do anything if user has hit sent blank message.
    // if text == "\n" {
    //     return Ok(());
    // }

    // Trim the whitespace from the input message
    let m = text.trim();

    // Don't let user do anything if they haven't got a username and aren't trying to define one
    if gamesess.name.is_none() && !m.starts_with("/name") {
        ctx.text("Define a username before chatting. Type your username below:");
        return Ok(());
    }

    // Anything that begins with a / is a command.
    if m.starts_with('/') {
        let v: Vec<&str> = m.splitn(2, ' ').collect();
        match v[0] {
            "/name" => {
                // Let the user define their username
                if let Some(name) = &gamesess.name {
                    ctx.text(format!("You already have the name {name}!"));
                } else if v.len() != 2 {
                    ctx.text("You need to input a name!");
                } else if v[1].is_inappropriate() || v[1].starts_with('/') {
                    // Filter profanity and usernames that start with /
                    ctx.text("Invalid username.");
                } else {
                    // Try register the username on the game db.
                    let user_name = v[1];
                    match api::register_user(user_name, gamesess.id)? {
                        false => ctx.text("Username already exists. Pick another."),
                        true => {
                            // Assign username in the chat session
                            gamesess.name = Some(user_name.to_owned());

                            // Update the chat session's visitor list
                            gamesess.addr.do_send(game_server::NewName {
                                name: user_name.to_owned(),
                                id: gamesess.id,
                            });

                            // Notify the player's local client what their user id is
                            ctx.text(format!("//cmd;yourid;{}", gamesess.id));
                            ctx.text(format!("Welcome {}. Begin typing to chat.", user_name));
                            // Reset the local client
                            ctx.text("//cmd;default");
                        }
                    }
                }
            }
            "/help" => {
                // User wants help on commands they can use
                ctx.text(helpme());
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
                    gamesess.id, gamesess.name, gamesess.room
                ));
            }
            "/who" => {
                // Display who is online
                gamesess
                    .addr
                    .send(game_server::Who {})
                    .into_actor(gamesess)
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
                let session_id = api::new_game(&gamesess.id)?;

                // Join the game session's chat room
                gamesess.room = session_id.to_owned();
                gamesess.addr.do_send(game_server::Join {
                    id: gamesess.id,
                    room: gamesess.room.clone(),
                    username: gamesess.name.as_ref().unwrap().to_owned(),
                });

                // Set player to team black and notify the client
                gamesess.team = Team::Black;
                ctx.text("//cmd;team;B");

                ctx.text(format!(
                    "You have created and joined game room {}.\nNow waiting for an opponent...",
                    session_id
                ));
            }
            "/join" => {
                if v.len() == 2 {
                    // Check the db to see if there's a session with this id
                    // let session_id = v[1].to_owned();
                    // no function to do this yet, create one later
                    ctx.text("Joining specific games is unimplemented. Just type /join to see if any are available.");
                } else {
                    // Join an empty game if one is available
                    match api::find()? {
                        Some(game_state) => {
                            // The game_state's id will define the game room name
                            let session_id = game_state.id.to_owned();

                            // Join on the sqlite db
                            api::join(&session_id, &gamesess.id)?;

                            // Join in the chat
                            gamesess.room = session_id.to_owned();
                            gamesess.addr.do_send(game_server::Join {
                                id: gamesess.id,
                                room: gamesess.room.clone(),
                                username: gamesess.name.as_ref().unwrap().to_owned(),
                            });

                            // Set joining player to team white and notify their local client
                            gamesess.team = Team::White;
                            ctx.text("//cmd;team;W");
                            ctx.text(format!("You joined game room {}", session_id));

                            // Get updated GameState and notify both players of what it is
                            let game_state = api::get_game_state(&session_id)?;
                            gamesess.addr.do_send(game_server::NewGame {
                                session_id,
                                game_state,
                            });
                        }
                        None => ctx.text("No empty games available. Try \x1b[31;1m/create\x1b[0m one!"),
                    }
                }
            }
            _ => ctx.text(format!("!!! unknown command: {m:?}")),
        }
    } else {
        // Anything that doesn't start with a / is a chat msg
        let msg = format!("\x1b[36;2m{}:\x1b[0m {m}", &gamesess.name.as_ref().unwrap());

        // Send msg to everyone in the same room.
        gamesess.addr.do_send(game_server::ClientMessage {
            id: gamesess.id,
            msg,
            room: gamesess.room.clone(),
        })
    }

    Ok(())
}

/// Parses user inputs when they're typed in game
fn in_game_parser(
    gamesess: &mut WsGameSession,
    text: String,
    ctx: &mut WebsocketContext<WsGameSession>,
) -> Result<(), Box<dyn Error>> {
    //  This should be caught by the user's local client: but don't do anything if user has hit sent blank message.
    // if text == "\n" {
    //     return Ok(());
    // }

    let m = text.trim();
    // Anything that begins with a / is a command
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
            // create an api to remove self from session, lose game, join main etc. Mimic join above.
            // api::remove
            // }
            "/id" => {
                // Display info to user on themselves
                ctx.text(format!(
                    "Your user id is: {}, and username is {:?}. You're in game_session: {}",
                    gamesess.id, gamesess.name, gamesess.room
                ));
            }
            "/who" => {
                // Display who is online
                gamesess
                    .addr
                    .send(game_server::Who {})
                    .into_actor(gamesess)
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
                // User wants to send msg to opponent
                let words = v[1];
                let msg = format!(
                    "\x1b[36;2m{}:\x1b[0m {words}",
                    &gamesess.name.as_ref().unwrap()
                );
                // Send msg to opponent
                gamesess.addr.do_send(game_server::ClientMessage {
                    id: gamesess.id,
                    msg,
                    room: gamesess.room.clone(),
                })
            }
            "/help" => {
                // User wants help on game controls
                ctx.text(pmoore::help_me());
            }
            "/xylophone" => {
                ctx.text(pmoore::xylophone());
            }
            "/play" => {
                // Get the gamestate from the db and make sure it is this player's turn
                let gamestate = api::get_game_state(&gamesess.room)?;

                if gamesess.id.to_string() != gamestate.last_user_id.unwrap() {
                    // This is the first thing a player does on their turn, so first, make sure the board is up to date
                    let gamestate = api::get_game_state(&gamesess.room)?;

                    // Save copy of the board to WsGameSession so that we don't have to keep querying the sqlite db
                    let mut board = Board::<Cube>::default();
                    board = board.decode_spiral(gamestate.board.unwrap());
                    gamesess.board = board;

                    // Set this player as active and ask them to select a chip to move
                    gamesess.active = true;
                    ctx.text("//cmd;msg;Select a chip from the board or your hand to move.");
                    ctx.text(hoive::game::ask::Req::Select.to_string())
                } else {
                    ctx.text("It's not your turn");
                }
            }
            "/select" | "/mosquito" | "/pillbug" | "/sumo" | "/skip" | "/moveto" | "/forfeit"
                if gamesess.active =>
            {
                // All of these are standard commands covered by pmoore.
                match v[0] {
                    "/select" => pmoore::select_chip_prompts(
                        &mut gamesess.action,
                        v[1],
                        &gamesess.board,
                        gamesess.team,
                    )?,
                    "/mosquito" => {
                        pmoore::mosquito_prompts(&mut gamesess.action, v[1], &gamesess.board)?
                    }
                    "/pillbug" => pmoore::pillbug_prompts(&mut gamesess.action, v[1])?,
                    "/sumo" => {
                        pmoore::sumo_victim_prompts(&mut gamesess.action, v[1], &gamesess.board)?
                    }
                    "/skip" => pmoore::skip_turn(&mut gamesess.action),
                    "/moveto" => pmoore::move_chip_prompts(&mut gamesess.action, v[1])?,
                    "/forfeit" => pmoore::forfeit(&mut gamesess.action),
                    _ => !unreachable!(),
                }

                ctx.text(format!("//cmd;msg;{}",gamesess.action.message.to_owned()));
                ctx.text(gamesess.action.request.to_string());
            }
            "/execute" if gamesess.active => {
                // Ask the server to execute the move and return the response
                let result = api::make_action(&gamesess.action, &gamesess.room)?;

                match result {
                    MoveStatus::Success => {
                        // No longer this player's turn
                        gamesess.active = false;
                        gamesess.action = BoardAction::default();

                        // Update all players on what the new gamestate is
                        let session_id = gamesess.room.to_owned();
                        let game_state = api::get_game_state(&session_id)?;

                        // For debug
                        ctx.text(format!("Board in spiral is {:?}", game_state.board));

                        gamesess.addr.do_send(game_server::UpdateGame {
                            session_id,
                            game_state,
                        });
                    }
                    MoveStatus::Win(_) => {
                        // Grab the winner's id off the game server
                        // update all player gamestates
                        let session_id = gamesess.room.to_owned();
                        let game_state = api::get_game_state(&session_id)?;

                        let winner = game_state.get_winner().unwrap();
                        // send a message to everyone saying who winner is
                        gamesess.addr.do_send(game_server::Winner {
                            team: winner.team,
                            room: gamesess.room.to_owned(),
                            username: Some(winner.username),
                            forfeit: winner.forfeit,
                        });

                        // boot players. Need to figure out how to grab their usernames.
                        let usr1 = game_state.clone().user_1.unwrap();
                        let usr2 = game_state.user_2.unwrap();
                 
                        gamesess.addr.do_send(game_server::Join {
                            id: usr1.parse::<usize>().unwrap(),
                            room: "main".to_string(),
                            username: "player".to_string(),
                        });

                        gamesess.addr.do_send(game_server::Join {
                            id: usr2.parse::<usize>().unwrap(),
                            room: "main".to_string(),
                            username: "player".to_string(),
                        });

                        // set the players as not in a game, remove precursor, reset all
                        // delete the game from the db
                        // needs implementing!
                    }
                    _ => {
                        // Get the client back into the select phase, reset the cmdlist
                        gamesess.action = BoardAction::default();
                        ctx.text("//cmd;select");
                    }
                }

                ctx.text(result.to_string());
            }
            "/abort" if gamesess.active => {
                // Abort the move go back into select phase

                gamesess.action = BoardAction::default();

                // reset the client and local boards
                let session_id = gamesess.room.to_owned();
                let game_state = api::get_game_state(&session_id)?;

                ctx.text(format!("//cmd;upboard;{}", game_state.board.unwrap()));
                ctx.text("//cmd;select");
                ctx.text("Move aborted.");
                ctx.text("//cmd;msg;Select a chip from the board or your hand to move.");
            }
            "/main" => {
                // Can't do it this way or the user can select main themselves. Could have a flag, or figure out how to dump both server/local clients back in main properly.
                gamesess.room = "main".to_string();
            }
            _ => ctx.text(format!("Invalid command.")),
        }
    } else {
        // Default chat is off in game. Need to use /t
        ctx.text("Normal chat is off during games. Use \x1b[31;1m/tell\x1b[0m or \x1b[31;1m/t\x1b[0m to talk to the other player");
    }

    Ok(())
}

/// User help
fn helpme() -> &'static str {
"
----------------------------------------------------------------\n
= Main lobby chat =\n
Type into your terminal to start chatting to other players in the main lobby.\n
You can also use the following commands:\n
\x1b[31;1m/create\x1b[0m:\tcreate a new game of Hoive;
\x1b[31;1m/join\x1b[0m:\t\tsearch for and join a game of Hoive;
\x1b[31;1m/id\x1b[0m:\t\tget your player id and the game you are in;
\x1b[31;1m/who\x1b[0m:\t\tdisplay who is online (alternatively hit enter/return);
\x1b[31;1m/help\x1b[0m:\t\tdisplay this help message.
----------------------------------------------------------------
"

}