use crate::game::actions::BoardAction;
use crate::game::actions::Command;
use crate::game::board::Board;
use crate::game::comps::convert_static_basic;
use crate::game::comps::Chip;
use crate::game::comps::Team;
use crate::maths::coord::Coord;
use std::collections::BTreeSet;
use std::error::Error;
use crate::game::comps::get_team_from_chip;
use crate::maths::coord::DoubleHeight;
use crate::draw;



/// Uses a select chip input string (textin) from a given active_team to update a BoardAction
pub fn select_chip<T: Coord>(
    action: &mut BoardAction,
    textin: &str,
    board: &Board<T>,
    active_team: Team,
) -> Result<(), Box<dyn Error>> {

    // At this stage, the text input will define what our chip is
    let chip_select = match textin {
        _ if textin.is_empty() => {
            action.message = format!(
                "{}\n\n-------------------- PLAYER HAND --------------------\n\n{}\n\n-----------------------------------------------------\n",
                draw::show_board(board),
                draw::list_chips(board, active_team)
            );
            return Ok(());
        }
        _ if textin == "w" => {
            // Atempt to skip turn, return db response
            action.command = Command::SkipTurn;
            return Ok(());
        }
        #[cfg(feature = "debug")]
        _ if textin == "s" => {
            action.command = Command::Save;
            action.message = "Enter a filename".to_string();
            return Ok(())
        }
        _ if textin == "mb" => {
            // The player is probably trying to select their mosquito acting like a beetle
            convert_static_basic("m1".to_string())
        }
        _ if textin.contains('*') => {
            // The player is probably trying to select a beetle (or a mosquito acting like one).
            // Grab the first 2 chars of the string
            let (mut first, _) = textin.split_at(2);

            // If the first two chars are mosquito, convert to m1
            if first.contains('m') {
                first = "m1";
            }
            convert_static_basic(first.to_string())
        }
        _ if textin.starts_with(|c| c == 'l' || c == 'p' || c == 'q' || c == 'm') => {
            let proper_str = match textin.chars().next().unwrap() {
                'l' => "l1",
                'p' => "p1",
                'q' => "q1",
                'm' => "m1",
                _ => unreachable!(),
            };
            convert_static_basic(proper_str.to_string())
        }
        c => {
            // Try and match a chip by this name
            convert_static_basic(c.to_owned())
        }
    };

    match chip_select {
        None => {
            // Player tried to select a chip that doesn't exist.
            action.message = "You don't have this tile in your hand. Select a chip.".to_string();
        }
        Some(chip_name) => {
            // Default params
            action.name = chip_name.to_string();
            action.message = "Select co-ordinate to move to. Input column then row, separated by comma, e.g.: 0, 0. Hit x to abort the move.".to_string();
            action.command = Command::Move;

            let on_board = board.get_position_byname(active_team, chip_name);
            let can_special = on_board.is_some() && on_board.unwrap().get_layer() == 0;

            match chip_name {
            _ if chip_name == "p1" && can_special => {
                // Player selected pillbug on the board
                action.message = "Hit m to sumo a neighbour, or anything else to do move.".to_string();
                action.command = Command::Pillbug;




                    // Get pillbug's position, save to rowcol
                    let position = board
                        .get_position_byname(active_team, chip_name)
                        .unwrap();
                    action.rowcol = Some(position.to_doubleheight(position));

                    // Get the neighbours
                    let neighbours = board.get_neighbour_chips(position);

                    // stick them into a BTree to preserve order.
                    // Probably want to store these later for retrieval
                    // This here is wonk. but works. It's converting back and forth from chip to string dozens of times
                    let neighbours = neighbours.into_iter().collect::<BTreeSet<Chip>>();

                    // need to map to upper/lowercase string
                    let neighbours = neighbours
                        .into_iter()
                        .map(|c| c.to_string())
                        .collect::<BTreeSet<String>>();
                    // Store the neighbours for later
                    action.neighbours = Some(neighbours);


            }
            _ if chip_name == "m1" && can_special => {
                // Player selected mosquito on the board
  
                    // Get Mosquito's position, save to rowcol
                    let position = board
                        .get_position_byname(active_team, chip_name)
                        .unwrap();
                    action.rowcol = Some(position.to_doubleheight(position));

                    // Get the neighbours
                    let neighbours = board.get_neighbour_chips(position);

                    // stick them into a BTree to preserve order.
                    // Probably want to store these later for retrieval
                    // This here is wonk. but works. It's converting back and forth from chip to string dozens of times
                    let neighbours = neighbours.into_iter().collect::<BTreeSet<Chip>>();

                    action.message = format!(
                        "Select a neighbour to suck from...\n{}",
                        crate::draw::list_these_chips(neighbours.clone())
                    );
                    action.command = Command::Mosquito;

                    // need to map to upper/lowercase string
                    let neighbours = neighbours
                        .into_iter()
                        .map(|c| c.to_string())
                        .collect::<BTreeSet<String>>();
                    // Store the neighbours for later
                    action.neighbours = Some(neighbours);
                
            },
            _ => {}, // nothing needs changing
        }
    }
    }
   
    Ok(())
}

/// Parse user inputs into a set of coordinates and update board action
pub fn make_move (
    action: &mut BoardAction,
    textin: &str,
) -> Result<(), Box<dyn Error>>{

    //attempt to parse a move
    let usr_hex = crate::pmoore::coord_from_string(textin.to_owned());


    if let [Some(x), Some(y)] = usr_hex[..] {
        if (x + y) % 2 == 0 {
            action.rowcol = Some(DoubleHeight::from((x, y)));
            action.message = "Attemptig to executing move on the game board".to_string();
            action.command = Command::Execute;
        }
    } else {
        action.message = "Invalid co-ordinates, enter coordinates again or hit x to abort.".to_string();
        action.command = Command::Move;
    }

    Ok(())
}

/// Converts an input number str (textin) into a mosquito action for sucking
pub fn mosquito_prompts<T:Coord> (
    action: &mut BoardAction,
    textin: &str,
    board: &Board<T>,
) -> Result<(), Box<dyn Error>>{

    let selection = textin
    .parse::<usize>()
    .expect("Couldn't parse input into usize");

    let neighbours = action.neighbours.as_ref().unwrap();
    let selected = neighbours.into_iter().nth(selection).unwrap();

    // Get the coordinates of that selected chip
    let chipselect = Chip {
        name: convert_static_basic(selected.to_lowercase()).expect("Invalid chip"),
        team: get_team_from_chip(&selected),
    };
    let source = board.chips.get(&chipselect).unwrap().unwrap();
    let victim_pos = source.to_doubleheight(source);

    // Add to the action's special string to signify mosquito sucking victim at row,col
    let special = format!("m,{},{},", victim_pos.col, victim_pos.row);
    action.special = Some(special);
    action.message = "And where would you like to move to?".to_string();
    action.command = Command::Move;

    Ok(())
}

pub fn pillbug_prompts(
    action: &mut BoardAction,
    textin: &str,
) -> Result<(), Box<dyn Error>>{

    match textin == "m" {
        true => {
            action.message = format!(
                "Select a neighbour to sumo from...\n{}",
                crate::draw::list_these_chips_str(action.neighbours.clone().unwrap())
            );
            action.command = Command::Sumo;

        },
        false => {
            action.message = "Select co-ordinate to move to. Input column then row, separated by comma, e.g.: 0, 0. Hit x to abort the move.".to_string();
            action.command = Command::Move;
        },
    }
        Ok(())
    
}

pub fn sumo_prompts<T:Coord> (
    action: &mut BoardAction,
    textin: &str,
    board: &Board<T>,
) -> Result<(), Box<dyn Error>>{

    let selection = textin
    .parse::<usize>()
    .expect("Couldn't parse input into usize");

    let neighbours = action.neighbours.as_ref().unwrap();
    let selected = neighbours.into_iter().nth(selection).unwrap();

    // Get the coordinates of that selected chip
    let chipselect = Chip {
        name: convert_static_basic(selected.to_lowercase()).expect("Invalid chip"),
        team: get_team_from_chip(&selected),
    };
    let source = board.chips.get(&chipselect).unwrap().unwrap();
    let victim_pos = source.to_doubleheight(source);

    // Add to the action's special string to signify mosquito sucking victim at row,col
    let special = format!("p,{},{},", victim_pos.col, victim_pos.row);
    action.special = Some(special);
    action.message = "Select a co-ordinate to sumo this chip to. Input column then row, separated by a comma, e.g.: 0, 0. Hit enter to abort the sumo.".to_string();

    action.command = Command::SumoTo;

    Ok(())
}

pub fn sumo_to_prompts(
    action: &mut BoardAction,
    textin: &str,

) -> Result<(), Box<dyn Error>>{

    let coord = match crate::pmoore::coord_prompts(textin.to_string()) {
        None => {
            action.message = "Invalid coordinates".to_string();
            return Ok(())
        }, // abort move
        Some((row, col)) => (row, col),
    };

    action.rowcol = Some(DoubleHeight::from(coord));
    action.command = Command::Execute;


    Ok(())
}
