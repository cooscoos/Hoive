use actix_web::body::MessageBody;
use actix_web::web::Bytes;
use awc::ws;
use futures_util::{SinkExt as _, StreamExt as _};
use hoive::pmoore::get_usr_input;
use std::error::Error;
use std::{io, thread};
use tokio::{select, sync::mpsc};
use tokio_stream::wrappers::UnboundedReceiverStream;

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

    loop {
        select! {
            Some(msg) = ws.next() => {
                match msg {
                    Ok(ws::Frame::Text(txt)) => {
                        // Display messages from server
                        println!("{txt:?}");
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
                ws.send(ws::Message::Text(cmd.into())).await.unwrap();
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
