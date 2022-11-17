const VERSION: &str = env!("CARGO_PKG_VERSION");

pub mod htmlserv;
pub mod local;
pub mod websock;

use std::error::Error;

use bytes::Bytes;
use std::io;
use std::str;

/// Play games of Hoive online or locally
pub async fn play_games() -> Result<(), Box<dyn Error>> {
    // Welcome user with sweet ascii graphics
    hoive::pmoore::welcome();

    println!("Choose to play:\n1) Online (default),\n2) Local,\n3) Online with advanced set up");
    let textin = get_usr_input();

    match textin {
        _ if textin.contains('2') => local::play_offline(),
        _ if textin.contains('3') => websock::play_websock(false).await,
        _ => websock::play_websock(true).await,
        //_ => htmlserv::play_online().await,
    }
}

// Convert bytes to str
pub fn bytes_to_str(b: &Bytes) -> Result<&str, str::Utf8Error> {
    str::from_utf8(b)
}

/// Request user input into terminal, return a trimmed string
pub fn get_usr_input() -> String {
    let mut textin = String::new();

    io::stdin()
        .read_line(&mut textin)
        .expect("Failed to read line");

    textin.trim().to_string()
}
