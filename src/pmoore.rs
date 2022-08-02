// Patrick Moore is the GamesMaster: the human-readable interface between players and the game logic

use rand::Rng;
// To randomise which player goes first
use std::io; // For parsing player inputs

use crate::draw;
use crate::game::board::{Board, MoveStatus};
use crate::game::comps::{convert_static, Team};
use crate::game::specials;
use crate::maths::coord::Coord;

// Introduction: say hello and define who goes first
pub fn intro() -> Team {
    println!(
        "
░█░█░█▀█░▀█▀░█░█░█▀▀
░█▀█░█░█░░█░░▀▄▀░█▀▀
░▀░▀░▀▀▀░▀▀▀░░▀░░▀▀▀

The boardgame Hive, in Rust.
"
    );

    // Select a random team to go first
    let mut rand = rand::thread_rng();
    let first = match rand.gen() {
        true => Team::Black,
        false => Team::White,
    };

    println!("{} team goes first.\n", draw::team_string(first));
    first
}

// The game loop
pub fn take_turn<T: Coord>(board: &mut Board<T>, first: Team) -> MoveStatus {
    let active_team = match board.turns % 2 {
        0 => first,
        _ => !first,
    };

    println!("{} team's turn.\n", draw::team_string(active_team));

    // Ask player to select chip
    let mut chip_selection = None;
    while chip_selection == None {
        chip_selection = chip_select(board, active_team)
    }

    let chip_name = chip_selection.unwrap();

    // Is this chip a mosquito or pillbug and alreaddy on the board?
    let morp = (chip_name == "p1" || chip_name == "m1")
        && board.get_position_byname(active_team, chip_name).is_some();

    if morp {
        println!("Hit m to do special move, or select co-ordinate to move to. If moving, input column then row, separated by comma, e.g.: 0, 0. Hit enter to abort the move.");
    } else {
        println!("Select co-ordinate to move to. Input column then row, separated by comma, e.g.: 0, 0. Hit enter to abort the move.");
    }

    let textin = get_usr_input();

    let return_status;

    if textin == "m" && morp {
        return_status = special_prompts(board, chip_name, active_team);
    } else if textin == "m" && !morp {
        println!("This chip doesn't have special moves!");
        return_status = MoveStatus::Nothing;
    } else {
        return_status = movement_prompts(board, chip_name, active_team, textin);
    }

    // The board will handle itself. Patrick just needs to print messages for player
    message(board, &return_status);

    return_status

}

// Ask user to select chip. Returns None if user input invalid.
fn chip_select<T: Coord>(board: &Board<T>, active_team: Team) -> Option<&'static str> {
    println!("Select a tile from the board or your hand to move. Hit enter to see the board and your hand, h for help, s to save.");

    let textin = get_usr_input();

    match textin {
        _ if textin.is_empty() => {
            println!(
                "{}\n Hand:{}\n",
                draw::show_board(board, 5),
                draw::list_chips(board, active_team)
            );
            None
        } // hard-coded 5 here but can adapt based on game extremeties later
        _ if textin == "xylophone" => {
            xylophone();
            None
        }
        _ if textin == "h" => {
            println!("{}", help_me());
            None
        }
        _ if textin == "s" => {
            println!("Enter a filename:");
            let filename = get_usr_input();
            match board.history.save(filename) {
                Ok(()) => println!("Successfully saved to ./saved_games/"),
                Err(err) => println!("Could not save because: {}", err),
            }
            None
        }
        c => {
            // Try and match a chip by this name
            let chip_str = convert_static(c);

            match chip_str {
                Some(value) => Some(value),
                None => {
                    println!("You don't have this tile in your hand.");
                    None
                }
            }
        }
    }
}

// Run the player through prompts to execute a chip movement
fn movement_prompts<T: Coord>(
    board: &mut Board<T>,
    chip_name: &'static str,
    active_team: Team,
    textin: String,
) -> MoveStatus {

    // Keep asking them for a coordinate until they input something useful or give up and hit enter
    let mut coord = None;
    while coord == None {
        coord = match coord_select(textin.clone()) {
            None => return MoveStatus::Nothing, // abort move
            Some((row,col)) => Some((row,col)),
        };
    }

    // Convert from doubleheight to the board's co-ordinate system
    let game_hex = board.coord.mapfrom_doubleheight(coord.unwrap());

    // Try execute the move.
    board.move_chip(chip_name, active_team, game_hex)

}

// Ask user to select a coordinate. Returns None to abort parent function.
fn coord_select(mut textin: String) -> Option<(i8, i8)> {
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
                    coord_select(textin)
                }
            }
        }
        _ => {
            println!("Try again: enter two numbers separated by a comma. Enter to abort.");
            textin = get_usr_input();
            coord_select(textin)
        }
    }
}


fn message<T: Coord>(board: &mut Board<T>, move_status: &MoveStatus) {
    match move_status {
        MoveStatus::Success => {
            println!("{}\n", draw::show_board(board, 5));
            println!("Successful.");
        }
        MoveStatus::BadNeighbour => {
            println!("\n\x1b[31;1m<< Can't place a new chip next to other team >>\x1b[0m\n")
        }
        MoveStatus::HiveSplit => {
            println!("\n\x1b[31;1m<< No: this move would split the hive in two >>\x1b[0m\n")
        }
        MoveStatus::Occupied => {
            println!("\n\x1b[31;1m<< Can't move this chip to an occupied position >>\x1b[0m\n")
        }
        MoveStatus::Unconnected => {
            println!("\n\x1b[31;1m<< Can't move your chip to an unconnected position  >>\x1b[0m\n")
        }
        MoveStatus::SmallGap => {
            println!("\n\x1b[31;1m<< Gap too small for this piece to move into  >>\x1b[0m\n")
        }
        MoveStatus::BadDistance(value) => {
            println!("\n\x1b[31;1m<<  No: this peice must move {value} space(s)  >>\x1b[0m\n")
        }
        MoveStatus::NoBee => {
            println!("\n\x1b[31;1m<< Can't move existing chips until you've placed your bee  >>\x1b[0m\n")
        }
        MoveStatus::BeeNeed => {
            println!(
                "\n\x1b[31;1m<< It's your third turn, you must place your bee now  >>\x1b[0m\n"
            )
        }
        MoveStatus::RecentMove(chip) => {
            println!("\n\x1b[31;1m<< Can't do that this turn because chip {} moved last turn  >>\x1b[0m\n", chip.name)
        }
        MoveStatus::NotNeighbour => {
            println!("\n\x1b[31;1m<< That is not a neighbouring hex >>\x1b[0m\n")
        }
        MoveStatus::Win(teamopt) => {
            println!("{}\n", draw::show_board(board, 5));
            match teamopt {
                Some(team) => {
                    let team_str = draw::team_string(*team);
                    println!("\n << {team_str} team wins. Well done!  >> \n");
                }
                None => {
                    println!("\n << Draw. Both teams have suffered defeat! >> \n");
                }
            }
        }
        MoveStatus::Nothing => {}
    }
}

// Handles the attempt at doing a special move.
pub fn special_prompts<T: Coord>(
    board: &mut Board<T>,
    chip_name: &'static str,
    active_team: Team,
) -> MoveStatus {
    let move_status;

    // Find out if we're dealing with a mosquito or pillbug, then lead the player through the prompts to execute special
    match chip_name {
        "p1" => {
            // Pillbug

            // Get pillbug's position and neighbour chips
            let position = board.get_position_byname(active_team, chip_name).unwrap();
            let neighbours = board.get_neighbour_chips(position);

            // Ask player to select neighbouring chips from a list (presenting options 1-6 for white and black team chips)
            println!(
                "Select which chip to sumo by entering a number up to {}. Hit enter to abort.\n {}",
                neighbours.len() - 1,
                draw::list_these_chips(neighbours.clone())
            );

            let textin = get_usr_input();

            // This will abort
            if textin.is_empty() {
                return MoveStatus::Nothing;
            }

            let selection = match textin.parse::<usize>() {
                Ok(value) if value < neighbours.len() + 1 => value,
                _ => {
                    println!("Use a number from the list");
                    return MoveStatus::Nothing;
                    }
            };

            // get the co-ordinate of the selected chip
            let source = board.chips.get(&neighbours[selection]).unwrap().unwrap();

            // Ask player to select a co-ordinate to sumo to
            let mut coord = None;
            while coord == None {
                println!("Select a co-ordinate to sumo this chip to. Input column then row, separated by a comma, e.g.: 0, 0. Hit enter to abort the sumo.");
                let textin = get_usr_input();
                coord = match coord_select(textin.clone()) {
                    None => return MoveStatus::Nothing, // abort move
                    Some((row,col)) => Some((row,col)),
                };
            }

            // Convert from doubleheight to the game's co-ordinate system
            let dest = board.coord.mapfrom_doubleheight(coord.unwrap());

            // Try execute the move and show the game's messages.
            move_status = specials::pillbug_sumo(board, &source, dest, position);
            //message(board, &move_status);
        }
        "m1" => {
            move_status = MoveStatus::Success;
        } // mosquito is to do
        _ => panic!("Unrecognised chip"),
    }
    move_status
}

// Request user input into terminal
fn get_usr_input() -> String {
    let mut textin = String::new();

    io::stdin()
        .read_line(&mut textin)
        .expect("Failed to read line");

    textin.trim().to_string()
}

// Parse comma separated values in a string to a doubleheight co-ordinate
fn coord_from_string(str: String) -> Vec<Option<i8>> {
    str.trim()
        .split(',')
        .map(|c| match c.parse::<i8>() {
            Ok(value) => Some(value),
            Err(_) => None,
        })
        .collect::<Vec<Option<i8>>>()
}

// OooOoooh
fn xylophone() {
    let egg = "                                     \n    ....,,..                        \n    .......',;cllllooll:'.                     \n  ..;cccccodxxkkkOkkkxxxol;.                   \n .:ooooooddxkkkkOOOOOOOkxxdl,                  \n.cdddoooddxxkkkkOO0000OOOOkkx:.                \n'loddolooddxkkOOO00000000OOOO0x.                \n.;ldxdlllodxxkOO000KKKK0000OOO0x'                \n,codo::clodxkOO00KKKKKKKK00Okkk:                \n.,::;,;cldxkkOO00000KKK0000OkkOl.               \n.','.';:ldxxkOOO0000OO000O0OOkxo,               \n....',;:loxxkkkkOOkkOO00O0OOkxxd'              \n.....';cclodkkOkkkkOOO00OOOOkxxd'              \n .  ...,:looodkkkxxxkkkkkkkkxxxo.              \n .   .'';ldoodoolloddddddddoxxxo.              \n     ....,,',,;;::ldollccldxOkxo.              \n    .....'',::c::ox0OkdxxkOO0Oxl.              \n    ..'';:cllddc:lx0OOOOOO0Okxdl.              \n   ....';clcldl,;lx0OkxkOOOkdddc.              \n  ..   ..,cool'.;ld0Okdoddxxxxdl.              \n  .. ....':c:,...,:ldxkkdlodxxxo'              \n  .......',,,,....':dkOOkxdxxdxl.              \n  ......,::,.''';:coooddddddddd,.              \n  .......,cc,',,,,;:ccldxdllll;.               \n............','.,;::cloodddoc:c:.                \n.;;;'..''........',::clcclllooxo'                .\n.oxddl,.','''.......''''',:ldOKO:                 .\n.d0000ko;',:c;.    .....,ldOKNWNd.                 .\n.oKKXKX0kkcco;. ......;:;,oXWWMNk;.                 .\n.o00KKXX0O00Od,...''cxKNNXkldXWWO:'.                  \n'd00000KXKKXKkc.....c0NWWMWNK0X0x:,;.                  \n,d000KKKXXXXOl.',. .oXNNWWMWXXXKd'':;.                  \n.:OKKXXXXXNX0l.  ....:KWNWWWWWNX0d;.;:,.                  \n.o0XKXXXXXXKo'      .;ONNNWWWWNXKx:.,;;'.                  \n,xXXXXXXXKxl'   ..   .dNNNNNWNXXKkc'',''.                   \nOXXXXXXXk, ...   .    ;kKXNNNNXXKx,......                   \n0KKKXNXKd.   ...     ..cdxk0KNNN0l. ..',.                   \nkkxkkO00Ol.       ...  .lkxddddl,....:k0l.                  \n..';ldxx0k, .      .'.  :0Ol'. ..'. ;OXXx.";
    println!("{egg}");
}

// Returns info on how to play
fn help_me() -> &'static str {
    "
----------------------------------------------------------------\n
= How to play =\n
Each player starts the game with the following peices in their hand:\n
- 1 bee (q1)
- 2 spiders (s1, s2)
- 3 ants (a1, a2, a3)
- 2 beetles (b1, b2)
- 3 grasshoppers (g1, g2, g3)
- a mosquito (m1)
- a ladybird (l1)
- a pill bug (p1).\n
Select one of the peices above using the codes given in brackets,
and then enter a location to move the peice to on the board using
the board's grid co-ordinate system (e.g. 1,-3).\n
Press return or enter at any time to abort moves, or to see the peices
on the board and in your hand.\n
You can attempt to move any peice in your hand or on the board as
long as it belongs to you.\n
Pillbugs and mosquitos have special moves. To access these hit
the s key when prompted.\n
Game rules: https://en.wikipedia.org/wiki/Hive_(game)\n
----------------------------------------------------------------
"
}
