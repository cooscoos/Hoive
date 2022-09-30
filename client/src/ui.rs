pub mod local;
pub mod online;
/// Terminal UI for playing games of Hoive on a server
pub mod play;
pub mod setup;

use std::error::Error;

/// Set up connection to Hoive server, set user id, and play some games
pub async fn play_games() -> Result<(), Box<dyn Error>> {
    // Welcome user with sweet ascii graphics
    hoive::pmoore::welcome();

    println!("Choose to play: 1) Online, 2) Local");
    let textin = hoive::pmoore::get_usr_input();

    match textin.contains('2') {
        true => local::play_offline(),
        false => online::play_online().await,
    }
}
