///! Client for playing games of Hoive on a websocket server
use crate::get_usr_input;
use crate::websock::local_store::LGameSession;
use actix_web::body::MessageBody;
use actix_web::web::Bytes;
use awc::ws;
use futures_util::{SinkExt as _, StreamExt as _};
use std::error::Error;
use std::{io, thread};
use tokio::{select, sync::mpsc};
use tokio_stream::wrappers::UnboundedReceiverStream;

use std::str::FromStr;

use hoive::draw;
use hoive::game::comps::Team;
pub mod local_store;

/// Play games of Hoive online on a websocket server.
pub async fn play_websock(def_setup: bool) -> Result<(), Box<dyn Error>> {
    //env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
    //log::info!("starting WebSocket client");

    // Define the server to connect to: user can choose address/port if def_setup is false.
    let url = match def_setup {
        true => "ws://localhost:8080/api/ws".to_string(),
        false => match websock_setup().await {
            Ok(value) => format!("ws://{}ws", value),
            Err(err) => panic!("Err: {}", err),
        },
    };

    // Set up comms
    let (cmd_tx, cmd_rx) = mpsc::unbounded_channel();
    let mut cmd_rx = UnboundedReceiverStream::new(cmd_rx);

    // Run blocking terminal input reader on separate thread
    let input_thread = thread::spawn(move || loop {
        let mut cmd = String::with_capacity(32);

        if io::stdin().read_line(&mut cmd).is_err() {
            log::error!("error reading line");
            return;
        }

        cmd_tx.send(cmd).unwrap();
    });

    let (_res, mut ws) = match awc::Client::new().ws(url).connect().await {
        Ok(values) => values,
        Err(err) => {
            log::error!("error: {}", err);
            panic!("problem")
        }
    };

    //log::debug!("response: {_res:?}");
    println!("Connected! Welcome. Enter a username:");

    // Store information about game sessions locally
    let mut local = LGameSession::default();

    // Start knowing that we're going to be dumped in room and need to select a name
    local.precursor = "/name ".to_string();
    local.room = "main".to_string();

    loop {
        select! {

            // We recieved an input from the websocket server
            Some(msg) = ws.next() => {
                match msg {
                    Ok(ws::Frame::Text(txt)) => {
                        // Get the msg from the server
                        let msg = crate::bytes_to_str(&txt).unwrap();

                        // msgs that start with //cmd need handlers
                        if msg.starts_with("//cmd") {
                            let v: Vec<&str> = msg.split(';').collect();
                            let cmd = v[1];
                            match cmd {
                                "default" => {
                                    // Reset precursor
                                    local.precursor = String::new();
                                }
                                "room" => {
                                    // Update room
                                    local.room = v[2].to_owned();
                                }
                                "goback" => {
                                    // Reset local game session info
                                    local = LGameSession::default();
                                    local.room = "main".to_string();

                                    // tell the chat server to bring the session back to main
                                    ws.send(ws::Message::Text("/main".into())).await.unwrap();
                                }
                                "newgame" => {
                                    // Grab the gamestate and update local copy of board based on this
                                    let gamestate = serde_json::from_str(v[2])?;
                                    local.update(gamestate);

                                    match local.active{
                                        true => {
                                            println!("You take your turn first!");
                                            ws.send(ws::Message::Text("/play".into())).await.unwrap(); // tell server you're ready to play
                                        },
                                        false => println!("Other player goes first."),
                                    }
                                },
                                "gamestate" => {
                                    // Gamestate updates are recieved upon some change to gamestate at the server end

                                    // Parse the recieved txt and update local info
                                    let gamestate = serde_json::from_str(v[2])?;
                                    local.update(gamestate);

                                    match local.active {
                                        true => ws.send(ws::Message::Text("/play".into())).await.unwrap(),
                                        false => {
                                            local.precursor = String::new();
                                            local.game_message = String::new();
                                        },
                                    }

                                    // Show the board, etc
                                    println!("{}", show_game_info(&local));
                                }
                                "yourid" => {
                                    // Update player id
                                    local.id = v[2].to_owned();
                                }
                                "moveto" | "select" | "mosquito" | "pillbug" | "sumo" => {
                                    // Echo the cmd back to the websocket server in response to show that the client is synced up
                                    local.precursor = format!("/{cmd} ");
                                }
                                "execute" => {
                                    // Server says it's ready, so send an execute cmd to the server to do the move
                                    // This isn't really needed, could just execute within the server's code
                                    ws.send(ws::Message::Text("/execute".into())).await.unwrap();
                                }
                                "upboard" => {
                                    // Update the board
                                    local.board = local.board.decode_spiral(v[2].to_owned());

                                }
                                "team" => {
                                    //update player team
                                    local.team = Team::from_str(v[2])?;
                                }
                                "msg" => {
                                    // Update the message to guide players on what to do
                                    local.game_message = v[2].to_string();
                                    println!("{}",v[2]);
                                }
                                _ => {},
                            }
                        } else {
                            // Anything that starts without //cmd will be chat from other players
                            println!("{msg}");
                        }
                    }
                    Ok(ws::Frame::Ping(_)) => {
                        // respond to ping probes
                        ws.send(ws::Message::Pong(Bytes::new())).await.unwrap();
                    }
                    _ => {}
                }
            }

            // We typed something in locally that we want to send to the server
            Some(cmd) = cmd_rx.next() => {
                // Ignore empty
                if cmd.is_empty() {
                    continue;
                }

                // Behaviour in game and in main lobby is different
                match local.in_game(){
                    false => {
                        // We're in the main lobby so...
                        if cmd == "\n" {
                            // Default to sending /who if user hits return
                            ws.send(ws::Message::Text("/who".into())).await.unwrap();
                        } else {
                            // Send the message with the precursor
                            let sendme = format!("{}{}", local.precursor, cmd);
                            ws.send(ws::Message::Text(sendme.into())).await.unwrap();
                        }
                    },
                    true => {
                        // We're in game so ...
                        let cmd = cmd.trim();
                        //println!("Sending: {}{}", local.precursor, thing);
                        match cmd {
                            _ if cmd.is_empty() => {
                                // User hit return. Show board and other game info.
                                println!("{}", show_game_info(&local));
                            }
                            _ if cmd == "x" => {
                                // x is means the player wants to abort their move
                                ws.send(ws::Message::Text("/abort".into())).await.unwrap();
                            }
                            _ if cmd == "w" => {
                                // w means they want to skip turn
                                ws.send(ws::Message::Text("/skip".into())).await.unwrap();
                            }
                            _ if cmd == "quit" => {
                                // quit: forfeit the game
                                ws.send(ws::Message::Text("/forfeit".into())).await.unwrap();
                            }
                            _ if cmd.starts_with("/t") || cmd.starts_with("/tell") => {
                                // t and tell should always work too
                                ws.send(ws::Message::Text(cmd.into())).await.unwrap();
                            }
                            _ => {
                                // Otherwise, send their input with precursor
                                let sendme = format!("{}{}", local.precursor, cmd);
                                ws.send(ws::Message::Text(sendme.into())).await.unwrap()
                            },
                        }

                    }
                }
            }
            else => break
        }
    }

    input_thread.join().unwrap();
    Ok(())
}

/// Run user through prompts to attempt to join a Hoive server
async fn websock_setup() -> Result<String, Box<dyn Error>> {
    println!("Select a server address (leave blank for default localhost):");
    let textin = get_usr_input();
    let address = match textin {
        _ if textin.is_empty() => "localhost".to_string(), // default
        _ => textin,
    };

    println!("Select a port (leave blank for default 8080):");
    let textin = get_usr_input();

    let port = match textin {
        _ if textin.is_empty() => "8080".to_string(), // default
        _ => textin,
    };

    // Create a base url that points to the Hoive server
    let base_url = format!("{address}:{port}/api/");

    // Create a client and check the server is up and running
    let client = awc::Client::default();

    // Test the base url connects to a valid Hoive server of same version.
    // The Hoive client version (converted to bytes)
    let client_version = format!("Hoive-server v{}", crate::VERSION)
        .try_into_bytes()
        .unwrap();

    // Try and get a response from the server
    let mut res = client.get(format!("http://{}", base_url)).send().await?;

    // The server version
    let server_version = res.body().await?;

    match client_version == server_version {
        true => Ok(base_url),
        false => Err("server and client versions don't match.".into()),
    }
}

/// Show information about the board, the chips in the player's hand, whose turn it is, what they need to do next.
fn show_game_info(local: &LGameSession) -> String {
    // Show the board
    format!(
        "{}\n\n-------------------- PLAYER HAND --------------------\n\n{}\n\n-----------------------------------------------------\n{}\n{}\n",
        draw::show_board(&local.board),
        draw::list_chips(&local.board, local.team),
        local.turn_string(),
        local.game_message,
    )
}
