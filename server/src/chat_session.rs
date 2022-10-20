use std::error::Error;
use std::time::{Duration, Instant};
use std::usize;

use actix::prelude::*;
use actix_web_actors::ws;
use actix_web_actors::ws::WebsocketContext;
use hoive::game;

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
                    Ok(()) => {},
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
    // Don't do anything if user hits enter. Can possibly catch these at the client end?
    if text == "\n" {
        return Ok(());
    }

    let m = text.trim();
    // we check for /sss type of messages
    if m.starts_with('/') {
        let v: Vec<&str> = m.splitn(2, ' ').collect();

        if chatsess.name.is_none() && v[0] != "/name" {
            ctx.text("Define a username using /name before chatting");
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
                } else if v[1].is_inappropriate() {
                    // Username profanity filter
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

                            ctx.text(format!("Welcome {}. Begin typing to chat.", user_name));
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
        let msg = if let Some(ref name) = chatsess.name {
            format!("\x1b[36;2m{name}:\x1b[0m {m}")
        } else {
            m.to_owned()
        };
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
    // Enter shows board
    if text == "\n" {
        return Ok(());
    }

    let m = text.trim();
    // we check for /sss type of messages
    if m.starts_with('/') {
        let v: Vec<&str> = m.splitn(2, ' ').collect();

        if chatsess.name.is_none() && v[0] != "/name" {
            ctx.text("Define a username using /name before chatting");
            return Ok(());
        }

        match v[0] {
            "/join" => {
                if v.len() == 2 {
                    let session_id = v[1].to_owned();
                    // Check the db to see if there's a session with this id
                    // no function to do this yet, create one later

                    // If there's a match, then join the session, and join the chat for that room
                    return Ok(());
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
                        }
                        None => ctx.text("No empty games available. Try /create one!"),
                    }
                }
            }
            // "/leave" => {
            // }
            "/name" => {
                if let Some(name) = &chatsess.name {
                    ctx.text(format!("You already have the name {name}!"));
                } else if v.len() != 2 {
                    ctx.text("You need to input a name!");
                } else if v[1].is_inappropriate() {
                    // Username profanity filter
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

                            ctx.text(format!("Welcome {}. Begin typing to chat.", user_name));
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
            // "/gamestate" => {
            //     // Get and return the game state (as long as we're in a game)
            //     if chatsess.game_room != "main" {
            //         let gamestate = match api::get_game_state(&chatsess.game_room) {
            //             Ok(value) => value,
            //             Err(err) => panic!("Error {err}"),
            //         };

            //         match serde_json::to_string(&gamestate) {
            //             Ok(value) => ctx.text(value),
            //             Err(err) => panic!("Error {}", err),
            //         };
            //     } else {
            //         ctx.text("You're not in a game. There is no game state");
            //     }
            // }
            // "/do" => {
            //     // Do an action (as long as we're in a game)
            //     if chatsess.game_room != "main" {
            //         if v.len() == 2 {
            //             let action_string = v[1].to_owned();
            //         } else {
            //             ctx.text("No action requested");
            //         }
            //     } else {
            //         ctx.text("You're not in a game. There is no game state");
            //     }
            // }
            _ => ctx.text(format!("!!! unknown command: {m:?}")),
        }
    } else {
        let msg = if let Some(ref name) = chatsess.name {
            format!("\x1b[36;2m{name}:\x1b[0m {m}")
        } else {
            m.to_owned()
        };
        // send message to chat server
        chatsess.addr.do_send(chat_server::ClientMessage {
            id: chatsess.id,
            msg,
            room: chatsess.game_room.clone(),
        })
    }

    Ok(())
}
