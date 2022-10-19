use std::time::{Duration, Instant};
use std::usize;

use actix::prelude::*;
use actix_web_actors::ws;
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
                act.addr.do_send(chat_server::Disconnect { id: act.id });

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

        // register self in chat server. `AsyncContext::wait` register
        // future within context, but context waits until this future resolves
        // before processing any other events.
        // HttpContext::state() is instance of WsChatSessionState, state is shared
        // across all routes within application
        let addr = ctx.address();
        self.addr
            .send(chat_server::Connect {
                addr: addr.recipient(),
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
        self.addr.do_send(chat_server::Disconnect { id: self.id });
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
                // Detect hitting enter
                if text == "\n" {
                    println!("Newline");
                }

                let m = text.trim();
                // we check for /sss type of messages
                if m.starts_with('/') {
                    let v: Vec<&str> = m.splitn(2, ' ').collect();
                    match v[0] {
                        "/join" => {
                            if v.len() == 2 {
                                let session_id = v[1].to_owned();
                                // Check the db to see if there's a session with this id
                                // no function to do this yet, create one later

                                // If there's a match, then join the session, and join the chat for that room
                            } else {
                                // Join an empty game if there is one available
                                match api::find() {
                                    Ok(Some(game_state)) => {
                                        // Join the game

                                        let session_id = game_state.id.to_owned();
                                        match api::join(&session_id, &self.id) {
                                            Ok(()) => {
                                                // and now join its chat room
                                                self.game_room = game_state.id.to_owned();

                                                self.addr.do_send(chat_server::Join {
                                                    id: self.id,
                                                    name: self.game_room.clone(),
                                                });

                                                ctx.text(format!(
                                                    "joined game room {}",
                                                    session_id
                                                ));
                                            }
                                            Err(err) => panic!("Err {}", err),
                                        };
                                    }
                                    Ok(None) => {
                                        ctx.text("No empty games available. Try create one!")
                                    }
                                    Err(err) => panic!("Error {}", err),
                                }
                            }
                        }
                        "/name" => {
                            if v.len() == 2 {
                                let user_name = v[1];
                                // Profanity filter
                                if user_name.is_inappropriate() {
                                    ctx.text("Invalid username");
                                } else {
                                    // Register username on the game db.
                                    let _result = api::register_user(user_name, self.id);

                                    // Assign username in the chat
                                    self.name = Some(user_name.to_owned());

                                    ctx.text(format!(
                                        "Successfully changed name to: {}",
                                        user_name
                                    ));
                                }
                            } else {
                                ctx.text("!!! name is required");
                            }
                        }
                        "/wipe" => {
                            match api::delete_all() {
                                Ok(_) => ctx.text("Database wiped"),
                                Err(err) => panic!("Error {}", err),
                            };
                        }
                        "/getid" => {
                            let return_string = format!(
                                "Your user id is: {}, and username is {:?}. You're in game_session: {}",
                                self.id, self.name, self.game_room
                            );
                            ctx.text(return_string)
                        }
                        "/who" => {
                            // Display who is in this room
                        }
                        "/create" => {
                            // Create a new game on the db, register self as user_1
                            let session_id = match api::new_game(&self.id) {
                                Ok(value) => value,
                                Err(err) => panic!("Error: {}", err),
                            };

                            // and now join its chat room
                            self.game_room = session_id.to_owned();

                            self.addr.do_send(chat_server::Join {
                                id: self.id,
                                name: self.game_room.clone(),
                            });

                            ctx.text(format!("joined game room {}", session_id));
                        }
                        "/gamestate" => {
                            // Get and return the game state (as long as we're in a game)
                            if self.game_room != "main" {
                                let gamestate = match api::get_game_state(&self.game_room) {
                                    Ok(value) => value,
                                    Err(err) => panic!("Error {err}"),
                                };

                                match serde_json::to_string(&gamestate) {
                                    Ok(value) => ctx.text(value),
                                    Err(err) => panic!("Error {}", err),
                                };
                            } else {
                                ctx.text("You're not in a game. There is no game state");
                            }
                        }
                        "/do" => {
                            // Do an action (as long as we're in a game)
                            if self.game_room != "main" {
                                if v.len() == 2 {
                                    let action_string = v[1].to_owned();
                                } else {
                                    ctx.text("No action requested");
                                }
                            } else {
                                ctx.text("You're not in a game. There is no game state");
                            }
                        }
                        _ => ctx.text(format!("!!! unknown command: {m:?}")),
                    }
                } else {
                    if self.name.is_none() {
                        ctx.text("Define a username using /name before chatting");
                    } else {
                        let msg = if let Some(ref name) = self.name {
                            format!("{name}: {m}")
                        } else {
                            m.to_owned()
                        };
                        // send message to chat server
                        self.addr.do_send(chat_server::ClientMessage {
                            id: self.id,
                            msg,
                            room: self.game_room.clone(),
                        })
                    }
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
