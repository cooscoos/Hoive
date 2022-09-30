/// Patrick Moore is the GamesMaster. He:
/// - provides a human-readable interface between players and the game logic;
/// - orchestrates normal/special moves in a way that tries to comply with game rules.
/// Pmoore functions are used by
use crate::draw;
use crate::game::comps::{convert_static_basic, Chip, Team};
use crate::game::{actions::BoardAction, board::Board, movestatus::MoveStatus, specials};
use crate::maths::coord::{Coord, DoubleHeight};
use std::collections::BTreeSet;
use std::{error::Error, io};

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

/// For the team who are playing, take guided actions and request those actions from the board.
pub fn action_prompts<T: Coord>(
    board: &mut Board<T>,
    active_team: Team,
) -> Result<MoveStatus, Box<dyn Error>> {
    println!("Team {}, it's your turn!", draw::team_string(active_team));

    // Keep asking player to select chip until Some(value) happens
    let mut chip_selection = None;
    while chip_selection == None {
        chip_selection = chip_select(board, active_team)
    }

    // The user's entry decides what chip to select
    // Safe to unwrap because of loop above
    let base_chip_name = match chip_selection.unwrap() {
        "w" => return Ok(MoveStatus::SkipTurn),   // try and skip turn
        "quit" => return Ok(MoveStatus::Forfeit), // try and forfeit
        valid_name => valid_name,
    };

    // Make a mutable copy of the chip name
    let mut chip_name = base_chip_name;

    // Check if selected chip is on the board already
    let on_board = board.get_position_byname(active_team, chip_name);

    // Create a string to store info on special moves
    let mut special = String::new();

    // If it's a mosquito, on the board, on layer 0, then it must suck from another chip
    let mosquito_suck =
        chip_name == "m1" && on_board.is_some() && on_board.unwrap().get_layer() == 0;

    if mosquito_suck {
        let victim_pos;
        // Change mosquito's name now so that we can catch a pillbug prompt later
        (victim_pos, chip_name) = match mosquito_prompts(board, chip_name, active_team) {
            Some((new_name, vic_pos)) => (vic_pos, new_name), // mosquito morphs into another chip
            None => return Ok(MoveStatus::Nothing),           // aborted suck
        };

        // Add to special string to signify mosquito sucking victim at row,col
        special.push_str(&format!("m,{},{},", victim_pos.col, victim_pos.row));
    }

    // Is this chip a pillbug (or a mosquito acting like one?) and on the board?
    let is_pillbug = chip_name.contains('p') && on_board.is_some();
    if is_pillbug {
        println!("Hit m to sumo a neighbour, or select co-ordinate to move to. If moving, input column then row, separated by comma, e.g.: 0, 0. Hit enter to abort the move.");
    } else {
        println!("Select co-ordinate to move to. Input column then row, separated by comma, e.g.: 0, 0. Hit enter to abort the move.");
    };

    // Ask the user for input. If they hit m, try execute pillbug special, otherwise normal move
    let textin = get_usr_input();
    if textin == "m" {
        // Only pillbugs can do specials
        if !is_pillbug {
            return Ok(MoveStatus::NoSpecial);
        }

        let (victim_source, victim_dest) = match pillbug_prompts(board, chip_name, active_team) {
            Some(value) => value,
            None => return Ok(MoveStatus::Nothing),
        };

        special.push_str(&format!("p,{},{}", victim_source.col, victim_source.row));

        // Generate and return a BoardAction based on the special
        let sumo_action = BoardAction::do_move(
            base_chip_name,
            active_team,
            DoubleHeight::from((victim_dest.col, victim_dest.row)),
            special,
        );
        Ok(MoveStatus::Action(sumo_action))
    } else {
        match coord_prompts(textin) {
            Some((row, col)) => {
                let move_action = BoardAction::do_move(
                    base_chip_name,
                    active_team,
                    DoubleHeight::from((row, col)),
                    special,
                );
                Ok(MoveStatus::Action(move_action))
            }
            None => Ok(MoveStatus::Nothing),
        }
    }
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

/// Request user input into terminal, return a trimmed string
pub fn get_usr_input() -> String {
    let mut textin = String::new();

    io::stdin()
        .read_line(&mut textin)
        .expect("Failed to read line");

    textin.trim().to_string()
}

/// Ask user on active team to select chip. Returns None if user input invalid.
fn chip_select<T: Coord>(board: &mut Board<T>, active_team: Team) -> Option<&'static str> {
    println!("Hit enter to see the board and your hand, h (help), w (skip turn), 'quit' (forfeit).\nSelect a tile from the board or your hand to move.");
    #[cfg(feature = "debug")]
    println!("Or hit s to save");

    let textin = get_usr_input();

    match textin {
        _ if textin.is_empty() => {
            println!(
                "{}\n\n-------------------- PLAYER HAND --------------------\n\n{}\n\n-----------------------------------------------------\n",
                draw::show_board(board),
                draw::list_chips(board, active_team)
            );
            None
        }
        _ if textin == "xylophone" => {
            xylophone();
            None
        }
        _ if textin == "quit" => Some("quit"),
        _ if textin == "h" => {
            println!("{}", help_me());
            None
        }
        _ if textin == "s" => {
            #[cfg(feature = "debug")]
            {
                println!("Enter a filename:");
                let filename = get_usr_input();
                match board.history.save(filename) {
                    Ok(()) => println!("Successfully saved to ./saved_games/"),
                    Err(err) => println!("Could not save because: {}", err),
                }
            }
            None
        }
        _ if textin == "w" => Some("w"), // skip turn
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
                _ => panic!("unreachable"),
            };

            convert_static_basic(proper_str.to_string())
        }
        c => {
            // Try and match a chip by this name
            let chip_str = convert_static_basic(c);

            match chip_str.is_some() {
                true => chip_str,
                false => {
                    println!("You don't have this tile in your hand.");
                    None
                }
            }
        }
    }
}

/// Ask user to select a coordinate or hit enter to return None so that we can
/// abort the parent function.
fn coord_prompts(mut textin: String) -> Option<(i8, i8)> {
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
                    println!("Invalid co-ordinates, try again. Enter to abort.");
                    textin = get_usr_input();
                    coord_prompts(textin)
                }
            }
        }
        _ => {
            println!("Try again: enter two numbers separated by a comma. Enter to abort.");
            textin = get_usr_input();
            coord_prompts(textin)
        }
    }
}

/// Leads the player through executing a pillbug's sumo special move.
fn pillbug_prompts<T: Coord>(
    board: &mut Board<T>,
    chip_name: &'static str,
    active_team: Team,
) -> Option<(DoubleHeight, DoubleHeight)> {
    // Get pillbug's position and prompt the user to select a neighbouring chip to sumo, returning the coords of the victim
    let position = board.get_position_byname(active_team, chip_name).unwrap();
    let source = match neighbour_prompts(board, position, "sumo".to_string()) {
        Some(value) => value,
        None => return None, // abort special move
    };

    // Ask player to select a co-ordinate to sumo to
    println!("Select a co-ordinate to sumo this chip to. Input column then row, separated by a comma, e.g.: 0, 0. Hit enter to abort the sumo.");
    let textin = get_usr_input();
    let coord = match coord_prompts(textin) {
        None => return None, // abort move
        Some((row, col)) => (row, col),
    };

    // Convert from doubleheight to the game's co-ordinate system
    let dest = board.coord.mapfrom_doubleheight(DoubleHeight::from(coord));

    Some((source.to_doubleheight(source), dest.to_doubleheight(dest)))
}

/// Leads the player through executing a mosquito's suck
fn mosquito_prompts<T: Coord>(
    board: &mut Board<T>,
    chip_name: &'static str,
    active_team: Team,
) -> Option<(&'static str, DoubleHeight)> {
    // Get mosquitos's position and prompt the user to select a neighbouring chip to suck, returning the coords of the victim
    let position = board.get_position_byname(active_team, chip_name).unwrap();
    let source = match neighbour_prompts(board, position, "suck".to_string()) {
        Some(value) => value,
        None => return None, // abort special move
    };

    println!("You selected {:?}", source);

    // Execute the special move to become the victim for this turn
    match specials::mosquito_suck(board, source, position) {
        Some(value) => Some((value, source.to_doubleheight(source))),
        None => {
            println!("Cannot suck from another mosquito!");
            None
        }
    }
}

/// Ask the player to select neighbouring chips from a list (will present colour-coded options 0-5)
fn neighbour_prompts<T: Coord>(board: &mut Board<T>, position: T, movename: String) -> Option<T> {
    let neighbours = board.get_neighbour_chips(position);

    // stick them into a BTree to preserve order.
    let neighbours = neighbours.into_iter().collect::<BTreeSet<Chip>>();

    // Ask player to select neighbouring chips from a list (presenting options 0-6 for white and black team chips)
    println!(
        "Select which chip to {} by entering a number up to {}. Hit enter to abort.\n {}",
        movename,
        neighbours.len() - 1,
        draw::list_these_chips(neighbours.clone())
    );

    let textin = get_usr_input();

    // Returning none will abort the special move
    if textin.is_empty() {
        return None;
    }

    // Match to the player's selection
    let selection = match textin.parse::<usize>() {
        Ok(value) if value < neighbours.len() + 1 => value,
        _ => {
            println!("Use a number from the list");
            return None;
        }
    };
    let selected = neighbours.into_iter().nth(selection).unwrap();

    // get the co-ordinate of the selected chip and return them
    let source = board.chips.get(&selected).unwrap().unwrap();
    Some(source)
}

/// Parse comma separated values input by a user to a doubleheight co-ordinate
fn coord_from_string(str: String) -> Vec<Option<i8>> {
    str.trim()
        .split(',')
        .map(|c| match c.parse::<i8>() {
            Ok(value) => Some(value),
            Err(_) => None,
        })
        .collect::<Vec<Option<i8>>>()
}

/// ~~~ OooOoooh ~~~
fn xylophone() {
    let egg = "                                     \n    ....,,..                        \n    .......',;cllllooll:'.                     \n  ..;cccccodxxkkkOkkkxxxol;.                   \n .:ooooooddxkkkkOOOOOOOkxxdl,                  \n.cdddoooddxxkkkkOO0000OOOOkkx:.                \n'loddolooddxkkOOO00000000OOOO0x.                \n.;ldxdlllodxxkOO000KKKK0000OOO0x'                \n,codo::clodxkOO00KKKKKKKK00Okkk:                \n.,::;,;cldxkkOO00000KKK0000OkkOl.               \n.','.';:ldxxkOOO0000OO000O0OOkxo,               \n....',;:loxxkkkkOOkkOO00O0OOkxxd'              \n.....';cclodkkOkkkkOOO00OOOOkxxd'              \n .  ...,:looodkkkxxxkkkkkkkkxxxo.              \n .   .'';ldoodoolloddddddddoxxxo.              \n     ....,,',,;;::ldollccldxOkxo.              \n    .....'',::c::ox0OkdxxkOO0Oxl.              \n    ..'';:cllddc:lx0OOOOOO0Okxdl.              \n   ....';clcldl,;lx0OkxkOOOkdddc.              \n  ..   ..,cool'.;ld0Okdoddxxxxdl.              \n  .. ....':c:,...,:ldxkkdlodxxxo'              \n  .......',,,,....':dkOOkxdxxdxl.              \n  ......,::,.''';:coooddddddddd,.              \n  .......,cc,',,,,;:ccldxdllll;.               \n............','.,;::cloodddoc:c:.                \n.;;;'..''........',::clcclllooxo'                .\n.oxddl,.','''.......''''',:ldOKO:                 .\n.d0000ko;',:c;.    .....,ldOKNWNd.                 .\n.oKKXKX0kkcco;. ......;:;,oXWWMNk;.                 .\n.o00KKXX0O00Od,...''cxKNNXkldXWWO:'.                  \n'd00000KXKKXKkc.....c0NWWMWNK0X0x:,;.                  \n,d000KKKXXXXOl.',. .oXNNWWMWXXXKd'':;.                  \n.:OKKXXXXXNX0l.  ....:KWNWWWWWNX0d;.;:,.                  \n.o0XKXXXXXXKo'      .;ONNNWWWWNXKx:.,;;'.                  \n,xXXXXXXXKxl'   ..   .dNNNNNWNXXKkc'',''.                   \nOXXXXXXXk, ...   .    ;kKXNNNNXXKx,......                   \n0KKKXNXKd.   ...     ..cdxk0KNNN0l. ..',.                   \nkkxkkO00Ol.       ...  .lkxddddl,....:k0l.                  \n..';ldxx0k, .      .'.  :0Ol'. ..'. ;OXXx.";
    println!("{egg}");
}

/// Returns the game manual.
fn help_me() -> &'static str {
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
