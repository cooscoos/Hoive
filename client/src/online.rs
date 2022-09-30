pub mod comms;
/// Play games of Hoive online
pub mod play;
pub mod setup;

use hoive::game::board::Board;
use hoive::maths::coord::{Coord, Cube};
use server::models::Winner;
use std::error::Error;

/// Play games of Hoive online on a server
pub async fn play_online() -> Result<(), Box<dyn Error>> {
    // Run user through prompts to join a Hoive server
    let (client, base_url) = setup::join_server().await?;

    // For development, option to wipe the server clean
    println!("Dev wipe db? Enter nothing to do so.");
    if hoive::pmoore::get_usr_input().is_empty() {
        comms::wipe_db(&client, &base_url).await?;
    }

    // Ask user to register themselves on server's db
    setup::register_user(&client, &base_url).await?;

    // Keep playing the game?
    let mut play_games = true;

    // Play games for as long as user stays connected to the session
    while play_games {
        // Create or join a new game
        let (mut game_state, my_team, mut active_team) =
            setup::join_game(&client, &base_url).await?;

        // Initialise a new board, and new struct to store info on who won and why
        let mut board = Board::new(Cube::default());
        let mut winner = Winner::default();

        // Play this session while there's no winner (need to detect ctrl+c or ctrl+z terminal or timeout (2 mins))
        while !winner.happened(&game_state.winner) {
            // Take a turn, or wait and watch if it's not your turn.
            match my_team == active_team {
                true => {
                    game_state = play::take_turn(&mut board, my_team, &client, &base_url).await?
                }
                false => {
                    game_state = play::observe(&mut board, my_team, &client, &base_url).await?
                }
            }
            // Update our local copy of the active team and board
            active_team = game_state.which_team()?;
            board = board.decode_spiral(game_state.board.unwrap());
        }

        // The game ended. Tell user who won game, and ask them if they want to play again
        play_games = play::endgame(winner, my_team);
    }
    Ok(())
}
