/// Patrick Moore is the GamesMaster. He:
/// - provides a human-readable interface between players and the game logic;
/// - orchestrates normal/special moves in a way that tries to comply with game rules.
/// Pmoore functions are used by
use crate::{draw};
use crate::game::comps::{convert_static_basic, Chip, Team};
use crate::game::{actions::BoardAction, actions::Command,board::Board, movestatus::MoveStatus, specials};
use crate::maths::coord::{Coord, DoubleHeight};
use std::collections::BTreeSet;
use std::{error::Error, io};
use crate::game::comps::get_team_from_chip;


/// Say hello to the player
pub fn welcome() {
    println!(
        "
░█░█░█▀█░▀█▀░█░█░█▀▀
░█▀█░█░█░░█░░▀▄▀░█▀▀
░▀░▀░▀▀▀░▀▀▀░░▀░░▀▀▀

The boardgame Hive, in Rust.
"
    );
}

/// Decode a special string into a series of mosquito and/or pillbug actions
pub fn decode_specials<T: Coord>(
    board: &mut Board<T>,
    special: &str,
    active_team: Team,
    mut chip_name: &'static str,
    d_dest: DoubleHeight,
) -> MoveStatus {
    // Separate out the special's instructions using commas
    let items = special.split(',').collect::<Vec<&str>>();

    for (i, item) in items.clone().into_iter().enumerate() {
        // If we come across an m or a p, we need to read in the next 2 items to find col/row of victim
        if item == "m" || item == "p" {
            // Parse the victim coordinates into the board's coordinates
            let d_vic = DoubleHeight::from((
                items[i + 1].parse::<i8>().unwrap(),
                items[i + 2].parse::<i8>().unwrap(),
            ));
            let vic_coord = d_vic.mapto(board.coord);

            match item {
                "m" => {
                    // Get the mosquito's current position and ask it to absorb power from the victim
                    let position = board.get_position_byname(active_team, "m1").unwrap();
                    let newname = match specials::mosquito_suck(board, vic_coord, position) {
                        Some(value) => value,
                        None => return MoveStatus::NoSuck,
                    };
                    // Change the mosquito's name
                    chip_name = newname;
                }
                "p" => {
                    // Get the sumo-ing chip's position, parse destination and do the sumo
                    let position = board.get_position_byname(active_team, chip_name).unwrap();
                    let dest = d_dest.mapto(board.coord);
                    return specials::pillbug_sumo(board, vic_coord, dest, position);
                }
                _ => (), // ignore other entries
            }
        }
    }

    // if we get to this point without returning anything then we must be moving a mosquito, so do so
    board.move_chip(chip_name, active_team, d_dest.mapto(board.coord))
}







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


/// Ask user to select a coordinate or hit enter to return None so that we can
/// abort the parent function.
pub fn coord_prompts(mut textin: String) -> Option<(i8, i8)> {
    if textin.is_empty() {
        return None;
    }; // escape this function and start again

    let usr_hex = coord_from_string(textin);

    match usr_hex[..] {
        [Some(x), Some(y)] => {
            match (x + y) % 2 {
                // The sum of doubleheight coords should always be an even no.
                0 => Some((x, y)),
                _ => {
                    None
                }
            }
        }
        _ => {
            None
        }
    }
}

/// Parse comma separated values input by a user to a doubleheight co-ordinate
pub fn coord_from_string(str: String) -> Vec<Option<i8>> {
    str.trim()
        .split(',')
        .map(|c| match c.parse::<i8>() {
            Ok(value) => Some(value),
            Err(_) => None,
        })
        .collect::<Vec<Option<i8>>>()
}

/// ~~~ OooOoooh ~~~
pub fn xylophone() -> &'static str {
    "                                     \n    ....,,..                        \n    .......',;cllllooll:'.                     \n  ..;cccccodxxkkkOkkkxxxol;.                   \n .:ooooooddxkkkkOOOOOOOkxxdl,                  \n.cdddoooddxxkkkkOO0000OOOOkkx:.                \n'loddolooddxkkOOO00000000OOOO0x.                \n.;ldxdlllodxxkOO000KKKK0000OOO0x'                \n,codo::clodxkOO00KKKKKKKK00Okkk:                \n.,::;,;cldxkkOO00000KKK0000OkkOl.               \n.','.';:ldxxkOOO0000OO000O0OOkxo,               \n....',;:loxxkkkkOOkkOO00O0OOkxxd'              \n.....';cclodkkOkkkkOOO00OOOOkxxd'              \n .  ...,:looodkkkxxxkkkkkkkkxxxo.              \n .   .'';ldoodoolloddddddddoxxxo.              \n     ....,,',,;;::ldollccldxOkxo.              \n    .....'',::c::ox0OkdxxkOO0Oxl.              \n    ..'';:cllddc:lx0OOOOOO0Okxdl.              \n   ....';clcldl,;lx0OkxkOOOkdddc.              \n  ..   ..,cool'.;ld0Okdoddxxxxdl.              \n  .. ....':c:,...,:ldxkkdlodxxxo'              \n  .......',,,,....':dkOOkxdxxdxl.              \n  ......,::,.''';:coooddddddddd,.              \n  .......,cc,',,,,;:ccldxdllll;.               \n............','.,;::cloodddoc:c:.                \n.;;;'..''........',::clcclllooxo'                .\n.oxddl,.','''.......''''',:ldOKO:                 .\n.d0000ko;',:c;.    .....,ldOKNWNd.                 .\n.oKKXKX0kkcco;. ......;:;,oXWWMNk;.                 .\n.o00KKXX0O00Od,...''cxKNNXkldXWWO:'.                  \n'd00000KXKKXKkc.....c0NWWMWNK0X0x:,;.                  \n,d000KKKXXXXOl.',. .oXNNWWMWXXXKd'':;.                  \n.:OKKXXXXXNX0l.  ....:KWNWWWWWNX0d;.;:,.                  \n.o0XKXXXXXXKo'      .;ONNNWWWWNXKx:.,;;'.                  \n,xXXXXXXXKxl'   ..   .dNNNNNWNXXKkc'',''.                   \nOXXXXXXXk, ...   .    ;kKXNNNNXXKx,......                   \n0KKKXNXKd.   ...     ..cdxk0KNNN0l. ..',.                   \nkkxkkO00Ol.       ...  .lkxddddl,....:k0l.                  \n..';ldxx0k, .      .'.  :0Ol'. ..'. ;OXXx."
}

/// Returns the game manual.
pub fn help_me() -> &'static str {
    "
----------------------------------------------------------------\n
= How to play =\n
Each player starts the game with the following peices in their hand:\n
- 1 bee (q1 or q)
- 2 spiders (s1, s2)
- 3 ants (a1, a2, a3)
- 2 beetles (b1, b2)
- 3 grasshoppers (g1, g2, g3)
- a mosquito (m1 or m)
- a ladybird (l1 or l)
- a pill bug (p1 or p).\n
Select one of the peices above using the codes given in brackets,
and then enter a location to move the peice to on the board using
comma separated values e.g. 1,-3.\n
The dots in the terminal represent the centres of hexagons: positions
where each hexagonal peice can be placed. Each hex is surrounded by 6 neighbours.\n
Press enter at any time to abort moves, or to see the peices
on the board and in your hand.\n
You can attempt to move any peice you own that is in your hand
or on the board. The game won't hint about whether a move is
legal, but it will tell you if an attempted action is illegal.\n
Beetles can move on top of the hive. When they are on
top of the hive, they will be have an asterix (*) next
to their name. They can be reselected later by typing their code
with or without this asterix.\n
Pillbugs can sumo pieces next to them. To sumo, hit
the m key when prompted.\n
Mosquitos need to absorb the power of a neighbour before
they take any action.\n
If a mosquito absorbs power from a beetle and
ends up on top of the hive, it will be represented as
mb*. It can be selected using: m1, mb, or mb*.\n
Game rules: https://en.wikipedia.org/wiki/Hive_(game)\n
----------------------------------------------------------------
"
}
