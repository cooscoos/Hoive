/// Play games of Hoive locally (couch co-op)
use hoive::game::{
    actions::{BoardAction, Command},
    board::Board,
    comps::Team,
    movestatus::MoveStatus,
    specials,
};
use hoive::maths::coord::Coord;
use hoive::{draw, pmoore};

use rand::Rng;
use std::{error::Error, io};

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
        println!("Team {}, it's your turn!", draw::team_string(active_team));
        println!(
            "Hit enter to see the board and your hand, h (help), w (skip turn), 'quit' (forfeit)."
        );

        let mut action = BoardAction::default();

        // Loop until action prompts returns false to exit
        while action_prompts(&mut action, &mut board.clone(), active_team)?{};
        
        let move_status = match action.command {
            Command::SkipTurn => board.try_skip_turn(active_team),
            Command::Forfeit => MoveStatus::Win(Some(!active_team)),
            Command::Execute => try_execute_action(&mut board, action, active_team),
            _ => !unreachable!(),
        };

        println!("{}", move_status.to_string());
        // Refresh all mosquito names back to m1
        specials::mosquito_desuck(&mut board);
        println!("{}", draw::show_board(&board));
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

/// For the team who are playing, take guided actions and request those actions from the board.
pub fn action_prompts<T: Coord>(
    action: &mut BoardAction,
    board: &mut Board<T>,
    active_team: Team,
) -> Result<bool, Box<dyn Error>> {

    println!("{}", action.message);
    let textin = get_usr_input();

    match action.command {
        Command::Select => pmoore::select_chip(action, &textin, &board, active_team)?,
        Command::Mosquito => {
            pmoore::mosquito_prompts(action, &textin, board)?;
            // Have a check to see if we're a pillbug and correct the prompts
            // either here or in websocket pmoore
        }
        Command::Pillbug => pmoore::pillbug_prompts(action, &textin)?,
        Command::Sumo => pmoore::sumo_prompts(action, &textin, &board)?,
        Command::SumoTo => pmoore::sumo_to_prompts(action, &textin)?,
        Command::Move => pmoore::make_move(action, &textin)?,
        _ => return Ok(false),
    }

    Ok(true)
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
        Some(special) => {
            pmoore::decode_specials(board, &special, active_team, chip_name, d_dest.unwrap())
        }
        None => board.move_chip(chip_name, active_team, d_dest.unwrap().mapto(board.coord)),
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
