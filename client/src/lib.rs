const VERSION: &str = env!("CARGO_PKG_VERSION");

pub mod echoer;
pub mod local;
pub mod online;

use std::error::Error;

use bytes::Bytes;
use std::str;

/// Play games of Hoive online or locally
pub async fn play_games() -> Result<(), Box<dyn Error>> {
    // Welcome user with sweet ascii graphics
    hoive::pmoore::welcome();

    println!("Choose to play:\n1) Online (default),\n2) Local,\n3) Echo service (debug)");
    let textin = local::get_usr_input();

    match textin {
        _ if textin.contains('2') => local::play_offline(),
        _ if textin.contains('3') => echoer::echo_service().await,
        _ => online::play_online().await,
    }
}

// Convert bytes to str
pub fn bytes_to_str(b: &Bytes) -> Result<&str, str::Utf8Error> {
    str::from_utf8(b)
}
