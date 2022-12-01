use crate::get_usr_input;
/// Play games of Hoive locally (couch co-op)
use hoive::game::{
    actions::BoardAction, ask::Req, board::Board, comps::Team, movestatus::MoveStatus, specials,
};
use hoive::maths::coord::{Coord, Cube};
use hoive::{draw, pmoore};

use rand::Rng;
use std::error::Error;

/// Play games of Hoive on the same computer offline
pub fn play_offline() -> Result<(), Box<dyn Error>> {
    // Initialise game board in cube co-ordinates
    let mut board = Board::<Cube>::default();

    // Say hello, tell players who goes first
    let first_team = pick_team();

    // Loop the game until someone wins
    loop {
        let active_team = match board.turns % 2 {
            0 => first_team,
            _ => !first_team,
        };
        println!("Team {}, it's your turn!", draw::team_string(active_team));

        // BoardActions store information on the move a player wants to make
        let mut action = BoardAction::default();

        // Ask the player to build up an action until there's a request to execute it
        while action.request != Req::Execute {
            action_prompts(&mut action, &board, active_team)?;
        }

        // Try and execute action the player has generated
        let move_status = try_execute_action(&mut board, action, active_team);

        // Refresh all mosquito names back to m1
        specials::mosquito_desuck(&mut board);

        // Display the move status to user
        println!("{}", move_status.to_string());

        if move_status.is_success() {
            println!("{}", draw::show_board(&board));
        }

        if move_status.is_winner() {
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

/// Select a random team (Black or White) to go first
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

/// Guides the player through building and then requesting an action to be taken on the board.
pub fn action_prompts<T: Coord>(
    action: &mut BoardAction,
    board: &Board<T>,
    active_team: Team,
) -> Result<(), Box<dyn Error>> {
    // Display guidance to the user and ask for their input
    println!("{}", action.message);
    let textin = get_usr_input();

    // Inputs like x, enter, quit should always be caught
    match textin {
        _ if textin.starts_with('x') => {
            // Abort whatever action is being built
            *action = BoardAction::default();
        }
        _ if textin.is_empty() => {
            // User hit return/enter: display the board
            action.message = format!(
                "{}\n\n-------------------- PLAYER HAND --------------------\n\n{}\n\n-----------------------------------------------------\n{}\n",
                draw::show_board(board),
                draw::list_chips(board, active_team),
                action.message
            );
        }
        _ if textin == "w" => {
            // Request skip turn
            pmoore::skip_turn(action);
        }
        _ if textin == "quit" => {
            // Forfeit the game
            pmoore::forfeit(action,&0);
        }
        _ if textin == "h" => {
            // Display help, abort action
            *action = BoardAction::default();
            println!("{}", pmoore::help_me());
        }
        #[cfg(feature = "debug")]
        _ if textin == "s" => pmoore::save_game(board),
        _ => {
            // Otherwise select an appropriate path based on request being made
            match action.request {
                Req::Select => pmoore::select_chip_prompts(action, &textin, board, active_team)?,
                Req::Mosquito => pmoore::mosquito_prompts(action, &textin, board)?,
                Req::Pillbug => pmoore::pillbug_prompts(action, &textin)?,
                Req::Sumo => pmoore::sumo_victim_prompts(action, &textin, board)?,
                Req::Move => pmoore::move_chip_prompts(action, &textin)?,
                _ => {}
            }
        }
    }

    Ok(())
}

/// Try and execute a player action using the board. Just like a Hoive game webserver: we decode actions, try them out on a board, and return how successful the move was.
/// If the move is successful then the mut Board passed to this function is updated automatically.
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
        Some(special) if special.starts_with("forfeit") => MoveStatus::Win(Some(!active_team)),
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
