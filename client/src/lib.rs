const VERSION: &str = env!("CARGO_PKG_VERSION");

pub mod local;
pub mod online;
pub mod echoer;

use std::error::Error;

/// Play games of Hoive online or locally
pub async fn play_games() -> Result<(), Box<dyn Error>> {
    // Welcome user with sweet ascii graphics
    hoive::pmoore::welcome();

    println!("Choose to play:\n1) Online (default),\n2) Local,\n3) Echo service (debug)");
    let textin = hoive::pmoore::get_usr_input();

    match textin {
        _ if textin.contains('2') => local::play_offline(),
        _ if textin.contains('3') => echoer::echo_service().await,
        _ => online::play_online().await,
    }
}
