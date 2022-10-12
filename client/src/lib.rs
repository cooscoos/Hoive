const VERSION: &str = env!("CARGO_PKG_VERSION");

pub mod local;
pub mod online;

use std::error::Error;

/// Play games of Hoive online or locally
pub async fn play_games() -> Result<(), Box<dyn Error>> {
    // Welcome user with sweet ascii graphics
    hoive::pmoore::welcome();

    println!("Choose to play:\n1) Online (default),\n2) Local");
    let textin = hoive::pmoore::get_usr_input();

    match textin.contains('2') {
        true => local::play_offline(),
        false => online::play_online().await,
    }
}
