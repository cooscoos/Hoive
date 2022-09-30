/// Take actions to play live games of Hoive on the server
use reqwest::Client;
use std::{error::Error, thread, time::Duration};

use super::comms;
use server::models::{GameState, Winner};

use hoive::game::{actions::BoardAction, board::Board, comps::Team, movestatus::MoveStatus};
use hoive::maths::coord::Coord;
use hoive::pmoore;
use hoive::{draw, pmoore::get_usr_input};

/// Ask player to take a turn
pub async fn take_turn<T: Coord>(
    board: &Board<T>,
    active_team: Team,
    client: &Client,
    base_url: &String,
) -> Result<GameState, Box<dyn Error>> {
    println!("{}\n", draw::show_board(&board));
    'turn: loop {
        // Ask player to do action, provide them with response message, break loop if move was successful
        let temp_move_status = pmoore::action_prompts(&mut board.clone(), active_team)?;

        let move_status = match temp_move_status {
            MoveStatus::SkipTurn => {
                comms::send_action(BoardAction::skip(), client, base_url).await?
            }
            MoveStatus::Forfeit => {
                comms::send_action(BoardAction::forfeit(), client, base_url).await?
            }
            MoveStatus::Action(action) => comms::send_action(action, &client, &base_url).await?,
            _ => temp_move_status,
        };

        println!("{}", move_status.to_string());
        if move_status == MoveStatus::Success {
            break 'turn;
        }
    }

    // Update the local game state based on server db
    comms::get_gamestate(&client, &base_url).await
}

/// Poll the server every few seconds to check if other player is done with their move.
pub async fn observe<T: Coord>(
    board: &mut Board<T>,
    my_team: Team,
    client: &Client,
    base_url: &String,
) -> Result<GameState, Box<dyn Error>> {
    println!("{}\n", draw::show_board(&board));

    // Update the board based on info on the server
    let mut game_state = comms::get_gamestate(&client, &base_url).await?;

    println!("Waiting for other player to take turn...");

    let my_user_id = match my_team {
        Team::White => game_state.user_2,
        Team::Black => game_state.user_1,
    };

    // If the last person who took turn is you, then we're still waiting for other player
    while game_state.last_user_id.as_ref().unwrap() == my_user_id.as_ref().unwrap() {
        // Wait a few seconds, refresh gamestate
        thread::sleep(Duration::from_secs(5));
        game_state = comms::get_gamestate(&client, &base_url).await?;
    }
    game_state = comms::get_gamestate(&client, &base_url).await?;
    Ok(game_state)
}

/// Tell the player who won, ask them if they want to play again
pub fn endgame(winner: Winner, my_team: Team) -> bool {
    let mut endgame_msg = match winner.team {
        Some(team) if team == my_team => "You win ".to_string(),
        Some(team) if team != my_team => "You lose ".to_string(),
        None => "It's a draw!".to_string(),
        Some(_) => panic!("Unrecognised team has won"),
    };

    match winner.forfeit {
        true => endgame_msg.push_str("by forfeit!"),
        false => endgame_msg.push_str("by destruction of queen bee!"),
    }

    println!("{endgame_msg}");

    println!("Hit y to play again, anything else to quit.");
    get_usr_input() == "y"
}
