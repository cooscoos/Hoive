use actix_web::body::MessageBody;
use actix_web::web::Bytes;
use awc::ws;
use futures_util::{SinkExt as _, StreamExt as _};
use hoive::pmoore::get_usr_input;
use std::error::Error;
use std::str::FromStr;
use std::{io, thread};
use tokio::{select, sync::mpsc};
use tokio_stream::wrappers::UnboundedReceiverStream;


use hoive::game::board::Board;
use hoive::game::{comps::Team, movestatus::MoveStatus};
use hoive::maths::coord::{Coord, Cube};
use server::models::{Winner, GameState};
use hoive::draw;
use hoive::pmoore;

pub async fn echo_service() -> Result<(), Box<dyn Error>> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    // Define the server to connect to
    let url = match websock_setup().await {
        Ok(value) => format!("ws://{}ws", value),
        Err(err) => panic!("Err: {}", err),
    };

    //log::info!("starting echo WebSocket client");

    let (cmd_tx, cmd_rx) = mpsc::unbounded_channel();
    let mut cmd_rx = UnboundedReceiverStream::new(cmd_rx);

    // run blocking terminal input reader on separate thread
    let input_thread = thread::spawn(move || loop {
        let mut cmd = String::with_capacity(32);

        if io::stdin().read_line(&mut cmd).is_err() {
            log::error!("error reading line");
            return;
        }

        cmd_tx.send(cmd).unwrap();
    });

    let (res, mut ws) = match awc::Client::new().ws(url).connect().await {
        Ok(values) => values,
        Err(err) => {
            log::error!("error: {}", err);
            panic!("problem")
        }
    };

    //log::debug!("response: {res:?}");
    log::info!("Connected! Welcome. Type /name to set your name.");

    // Initialise a new board, and new struct to store info locally on who won and why
    let mut board = Board::new(Cube::default());
    let mut winner = Winner::default();
    let mut game_state = GameState::default();
    let mut in_game = false;
    let mut my_id = String::new(); // player id
    let mut my_turn = false;
    let mut my_team = Team::White;
    let mut precursor = String::new();


    loop {
        select! {
            Some(msg) = ws.next() => {
                match msg {
                    Ok(ws::Frame::Text(txt)) => {
                        // Display messages from server
                        let msg = crate::bytes_to_str(&txt).unwrap();

                        //println!("{}",msg);
                        if msg.starts_with("//cmd") {
                            // If the msg with //cmd then we need to do things.
                            let v: Vec<&str> = msg.split(' ').collect();
                            let cmd = v[1];

                            match cmd {
                                "newgame" => {
                                    // grab the gamestate and decode it into a local copy of the board
                                    let gamestate_txt = v[2];
                                    game_state = serde_json::from_str(&gamestate_txt)?;
                                    board = board.decode_spiral(game_state.board.unwrap());

                                    // Find out who goves first
                                    my_turn = my_id != game_state.last_user_id.unwrap();

                                    my_team = match my_turn{
                                        true => {
                                            println!("You take your turn first!");
                                            ws.send(ws::Message::Text("/play".into())).await.unwrap(); // tell server you're ready to play
                                            precursor = "/select".to_string(); // get into a select state
                                            Team::Black
                                        },
                                        false => {println!("Other player goes first.");Team::White}, 
                                    };
                                    // reset the local copy of the board, winner .. may no longer be needed
                                    winner = Winner::default();

                                    in_game = true;

                                    println!("New board");
                                },
                                "gamestate" => {
                                    // Gamestate updates are recieved upon some change, so also
                                    // double as notifying the player that it is their turn.

                                    // Parse the recieved txt into a Gamestate struct
                                    let gamestate_txt = v[2];
                                    game_state = serde_json::from_str(&gamestate_txt)?;
                                    println!("Gamestate is: {:?}", game_state);

                                    // Decode the game_state into a board
                                    board = board.decode_spiral(game_state.board.unwrap());

                                    // Figure out if it's your turn
                                    my_turn = my_id != game_state.last_user_id.unwrap();
                                    println!("Your turn = {}",my_turn);

                                    if my_turn {
                                        ws.send(ws::Message::Text("/play".into())).await.unwrap(); // tell server you're ready to play
                                        precursor = "/select".to_string(); // get into a select state
                                    } else {
                                        precursor = String::new(); // wipe your precursor
                                    }


                                }
                                "yourid" => {
                                    // Update player id
                                    my_id = v[2].to_owned();
                                }
                                "moveto" => {
                                    // Get into a moveto state
                                    precursor = "/moveto".to_string();
                                }
                                "execute" => {
                                    // Get into a moveto state
                                    precursor = "/execute".to_string();
                                }
                                "winner" => {}, // and so on
                                _ => {},
                            }

                        } else {
                            // It'll just be some chat from other players, so print it
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

            Some(cmd) = cmd_rx.next() => {
                if cmd.is_empty() {
                    continue;
                }
                
                if !in_game{
                    if cmd == "\n" {
                        // Default to sending /who
                        ws.send(ws::Message::Text("/who".into())).await.unwrap();
                    } else {
                        ws.send(ws::Message::Text(cmd.into())).await.unwrap();
                    }
                } else {
                    // Otherwise we're in game so keyboard should behave appropriately

                    let thing = cmd.trim();
                    match thing {
                        _ if thing.is_empty() => {
                            // Dont't send anything
                            let turn_string = match my_turn {
                                true => format!("It's your turn!"),
                                false => format!("Waiting for other player to take turn..."),
                            };

                            // show the board
                            println!(
                                "{}\n\n-------------------- PLAYER HAND --------------------\n\n{}\n\n-----------------------------------------------------\n{turn_string}\n",
                                draw::show_board(&board),
                                draw::list_chips(&board, my_team)
                            );
                        }
                        _ => {
                            let sendme = format!("{} {}", precursor, cmd);
                            println!("sending {sendme}");
                            ws.send(ws::Message::Text(sendme.into())).await.unwrap()
                        }, // send it to the server
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
