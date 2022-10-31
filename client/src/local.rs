/// Play games of Hoive locally (couch co-op)
use hoive::game::{
    actions::BoardAction, ask::Ask, board::Board, comps::Team, movestatus::MoveStatus, specials,
};
use hoive::maths::coord::{Coord, Cube};
use hoive::{draw, pmoore};

use rand::Rng;
use std::{error::Error, io};

/// Set up connection to Hoive server, set user id, and play some games
pub fn play_offline() -> Result<(), Box<dyn Error>> {
    // Initialise game board in cube co-ordinates
    let mut board = Board::<Cube>::default();

    // Say hello, tell players who goes first
    let first_team = pick_team();

    // Loop game until someone wins
    loop {
        let active_team = match board.turns % 2 {
            0 => first_team,
            _ => !first_team,
        };
        println!("Team {}, it's your turn!", draw::team_string(active_team));

        let mut action = BoardAction::default();

        // Loop until we're told to execute something
        while action.command != Ask::Execute {
            action_prompts(&mut action, &mut board.clone(), active_team)?;
        }

        // Try and execute action we've been given
        let move_status = try_execute_action(&mut board, action, active_team);

        // Refresh all mosquito names back to m1
        specials::mosquito_desuck(&mut board);

        // Display the move status to user
        println!("{}", move_status.to_string());

        if move_status.is_success() {
            println!("{}", draw::show_board(&board));
        }

        if let MoveStatus::Win(_) = move_status {
            println!("Play again? y/n");
            let textin = get_usr_input();
            match textin {
                _ if textin == "y" => {
                    let _result = play_offline();
                }
                _ => break,
            }
        }
    }
    Ok(())
}

/// Select random team to go first
fn pick_team() -> Team {
    // Select a random team to go first
    let mut rand = rand::thread_rng();
    let first = match rand.gen() {
        true => Team::Black,
        false => Team::White,
    };

    println!("{} team goes first.\n", draw::team_string(first));
    first
}

/// Guides the player through formulating an action request from the board.
pub fn action_prompts<T: Coord>(
    action: &mut BoardAction,
    board: &mut Board<T>,
    active_team: Team,
) -> Result<(), Box<dyn Error>> {
    println!("{}", action.message);
    let textin = get_usr_input();

    // Commands that always need to be caught regardless of what we're doing
    if textin.starts_with('x') {
        // Abort whatever action is being built
        action.reset();
        return Ok(());
    } else if textin.is_empty() {
        // Display the board
        action.message = format!(
            "{}\n\n-------------------- PLAYER HAND --------------------\n\n{}\n\n-----------------------------------------------------\n{}\n",
            draw::show_board(board),
            draw::list_chips(board, active_team),
            action.message
        );
        return Ok(());
    }

    match action.command {
        Ask::Select => pmoore::select_chip(action, &textin, &board, active_team)?,
        Ask::Mosquito => pmoore::mosquito_prompts(action, &textin, board)?,
        Ask::Pillbug => pmoore::pillbug_prompts(action, &textin)?,
        Ask::Sumo => pmoore::sumo_prompts(action, &textin, &board)?,
        Ask::SumoTo => pmoore::sumo_to_prompts(action, &textin)?,
        Ask::Move => pmoore::make_move(action, &textin)?,
        _ => {}
    }

    Ok(())
}

/// Try and execute a player action using the board. This emulates how the server decodes and then does actions.
pub fn try_execute_action<T: Coord>(
    board: &mut Board<T>,
    action: BoardAction,
    active_team: Team,
) -> MoveStatus {
    // Unwrap the action struct to get chip name, destination and special string
    let special_str = action.clone().special;
    let d_dest = action.rowcol;

    // Try execute a special if one is requested, otherwise normal move
    match special_str {
        Some(special) if special == "forfeit" => MoveStatus::Win(Some(!active_team)),
        Some(special) if special == "skip" => board.try_skip_turn(active_team),
        Some(special) => pmoore::decode_specials(
            board,
            &special,
            active_team,
            action.get_chip_name(),
            d_dest.unwrap(),
        ),
        None => board.move_chip(
            action.get_chip_name(),
            active_team,
            d_dest.unwrap().mapto(board.coord),
        ),
    }
}

/// Request user input into terminal, return a trimmed string
pub fn get_usr_input() -> String {
    let mut textin = String::new();

    io::stdin()
        .read_line(&mut textin)
        .expect("Failed to read line");

    textin.trim().to_string()
}
