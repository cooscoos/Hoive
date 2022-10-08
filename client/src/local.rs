/// Play games of Hoive locally (couch co-op)
use rand::Rng;
use std::error::Error;

use hoive::game::{
    actions::BoardAction, board::Board, comps::Team, movestatus::MoveStatus, specials,
};
use hoive::maths::coord::Coord;
use hoive::{draw, pmoore};

/// Set up connection to Hoive server, set user id, and play some games
pub fn play_offline() -> Result<(), Box<dyn Error>> {
    // Initialise game board in cube co-ordinates
    let coord = hoive::maths::coord::Cube::default();
    let mut board = Board::new(coord);

    // Say hello, tell players who goes first
    let first = pick_team();

    // Loop game until someone wins
    loop {
        let active_team = match board.turns % 2 {
            0 => first,
            _ => !first,
        };

        let temp_move_status = pmoore::action_prompts(&mut board.clone(), active_team)?;

        let move_status = match temp_move_status {
            MoveStatus::SkipTurn => board.try_skip_turn(active_team),
            MoveStatus::Forfeit => MoveStatus::Win(Some(!active_team)),
            MoveStatus::Action(action) => try_execute_action(&mut board, action, active_team),
            _ => temp_move_status,
        };

        println!("{}", move_status.to_string());
        // Refresh all mosquito names back to m1
        specials::mosquito_desuck(&mut board);
        println!("{}",draw::show_board(&board));
        if let MoveStatus::Win(_) = move_status {
            println!("Play again? y/n");
            let textin = pmoore::get_usr_input();
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

/// Try and execute a player action using the board. This emulates how the server decodes and then does actions.
fn try_execute_action<T: Coord>(
    board: &mut Board<T>,
    action: BoardAction,
    active_team: Team,
) -> MoveStatus {
    // Unwrap the action struct to get chip name, destination and special string
    let chip_name = action.get_chip_name();
    let d_dest = action.rowcol;
    let special_str = action.special;

    // Try execute a special if one is requested, otherwise normal move
    match special_str {
        Some(special) => pmoore::decode_specials(board, &special, active_team, chip_name, d_dest),
        None => board.move_chip(chip_name, active_team, d_dest.mapto(board.coord)),
    }
}
