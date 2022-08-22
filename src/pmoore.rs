/// Patrick Moore is the GamesMaster, who:
/// - provides a human-readable interface between players and the game logic;
/// - orchestrates normal/special moves in a way that tries to comply with game rules.
///
use rand::Rng;
// To randomise which player goes first
use std::io; // For parsing player inputs

use crate::draw;
use crate::game::comps::{convert_static_basic, Team};
use crate::game::{board::Board, movestatus::MoveStatus, specials};
use crate::maths::coord::{Coord, DoubleHeight};

/// Introduction: say hello and define which team goes first
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

/// The game loop. Pass the team which goes first and this will handle the rest.
pub fn take_turn<T: Coord>(board: &mut Board<T>, first: Team) -> MoveStatus {
    let active_team = match board.turns % 2 {
        0 => first,
        _ => !first,
    };

    println!("{} team's turn.\n", draw::team_string(active_team));

    // Keep asking player to select chip until Some(value) happens
    let mut chip_selection = None;
    while chip_selection == None {
        chip_selection = chip_select(board, active_team)
    }

    // The user's entry decides what chip to select
    let temp_chip_name = match chip_selection {
        Some("w") | None => return MoveStatus::Nothing, // w means turn was skipped
        Some("quit") => {
            message(board, &MoveStatus::Win(Some(!active_team)));
            return MoveStatus::Win(Some(!active_team))
         } // the team forfeited
        Some(value) => value,
    };

    // Try and find a chip on the board with this name
    let on_board = board.get_position_byname(active_team, temp_chip_name);

    // If it's a mosquito, on the board, on layer 0, then it needs to suck a power
    let mosquito_suck =
        temp_chip_name == "m1" && on_board.is_some() && on_board.unwrap().get_layer() == 0;

    // Update the chip name if it's a mosquito
    let chip_name = match mosquito_suck {
        true => {
            match mosquito_prompts(board, temp_chip_name, active_team) {
                Some(morphed_name) => morphed_name, // mosquito morphs into another piece
                None => return MoveStatus::Nothing, // aborted suck
            }
        }
        false => temp_chip_name,
    };

    // Is this chip a pillbug (or a mosquito acting like one?) and on the board?
    let is_pillbug = chip_name.contains('p') && on_board.is_some();

    let textin = if is_pillbug {
        println!("Hit m to sumo a neighbour, or select co-ordinate to move to. If moving, input column then row, separated by comma, e.g.: 0, 0. Hit enter to abort the move.");
        get_usr_input()
    } else {
        println!("Select co-ordinate to move to. Input column then row, separated by comma, e.g.: 0, 0. Hit enter to abort the move.");
        get_usr_input()
    };

    // If the user hits m then try execute a pillbug's special move
    let return_status = if textin == "m" && is_pillbug {
        pillbug_prompts(board, chip_name, active_team)
    } else if textin == "m" && !is_pillbug {
        println!("This chip doesn't have special moves!");
        MoveStatus::Nothing
    } else {
        match movement_prompts(board, textin) {
            // Otherwise try move the chip if the movement prompts are valid
            Some(value) => board.move_chip(chip_name, active_team, value),
            None => MoveStatus::Nothing,
        }
    };

    // The board will handle itself. Patrick just needs to print messages for player
    message(board, &return_status);

    // Refresh all mosquito names back to m1
    specials::mosquito_desuck(board);

    return_status
}

/// Ask user on active team to select chip. Returns None if user input invalid.
fn chip_select<T: Coord>(board: &mut Board<T>, active_team: Team) -> Option<&'static str> {
    println!("Select a tile from the board or your hand to move. Hit enter to see the board and your hand, h for help, s to save, w to skip turn, 'quit' to forfeit.");

    let textin = get_usr_input();

    match textin {
        _ if textin.is_empty() => {
            // hard-coded 5 for show_board here but can adapt based on game extremeties later.
            println!(
                "{}\n Hand:{}\n",
                draw::show_board(board),
                draw::list_chips(board, active_team)
            );
            None
        }
        _ if textin == "xylophone" => {
            xylophone();
            None
        }
        _ if textin == "quit" => {
            Some("quit")
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
        _ if textin == "w" => {
            // skip turn, only if both bees have been placed
            if board.bee_placed(active_team) && board.bee_placed(!active_team) {
                println!(
                    "\n{} team skipped their turn.\n",
                    draw::team_string(active_team)
                );
                board.turns += 1;
            } else {
                println!("Can't skip turns until both bees placed");
            }
            Some("w") // return this so that the main loop can return MoveStatus::Nothing
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
        _ if textin.starts_with(|c| c == 'l' || c == 'p' || c == 'q') => {
            let proper_str = match textin.chars().next().unwrap() {
                'l' => "l1",
                'p' => "p1",
                'q' => "q1",
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

/// Run the player through prompts to execute a chip movement
/// Returns the movestatus and the coordinate the player moved to
fn movement_prompts<T: Coord>(board: &mut Board<T>, textin: String) -> Option<T> {
    // Ask user to input dheight co-ordinates
    let coord = match coord_prompts(textin) {
        None => return None, // abort move
        Some((row, col)) => (row, col),
    };

    let moveto = DoubleHeight::from(coord);

    // Convert from doubleheight to the board's co-ordinate system
    let game_hex = board.coord.mapfrom_doubleheight(moveto);

    // Try execute the move, return the hex
    Some(game_hex)
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

/// Print feedback for the player based on how un/successful an attempted action was.
fn message<T: Coord>(board: &mut Board<T>, move_status: &MoveStatus) {
    match move_status {
        MoveStatus::Success => {
            println!("{}\n", draw::show_board(board));
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
        MoveStatus::BeetleBlock => {
            println!(
                "\n\x1b[31;1m<< A beetle on top of you prevents you from taking action >>\x1b[0m\n"
            )
        }
        MoveStatus::BeetleGate => {
            println!("\n\x1b[31;1m<< A beetle gate prevents this move >>\x1b[0m\n")
        }
        MoveStatus::NoJump => {
            println!("\n\x1b[31;1m<< Grasshopper can't make this jump >>\x1b[0m\n")
        }
        MoveStatus::Win(teamopt) => {
            println!("{}\n", draw::show_board(board));
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

/// Leads the player through executing a pillbug's sumo special move.
fn pillbug_prompts<T: Coord>(
    board: &mut Board<T>,
    chip_name: &'static str,
    active_team: Team,
) -> MoveStatus {
    // Get pillbug's position and prompt the user to select a neighbouring chip to sumo, returning the coords of the victim
    let position = board.get_position_byname(active_team, chip_name).unwrap();
    let source = match neighbour_prompts(board, position, "sumo".to_string()) {
        Some(value) => value,
        None => return MoveStatus::Nothing, // abort special move
    };

    // Ask player to select a co-ordinate to sumo to
    println!("Select a co-ordinate to sumo this chip to. Input column then row, separated by a comma, e.g.: 0, 0. Hit enter to abort the sumo.");
    let textin = get_usr_input();
    let coord = match coord_prompts(textin) {
        None => return MoveStatus::Nothing, // abort move
        Some((row, col)) => (row, col),
    };

    // Convert from doubleheight to the game's co-ordinate system
    let dest = board.coord.mapfrom_doubleheight(DoubleHeight::from(coord));

    // Try execute the move and show the game's messages.
    specials::pillbug_sumo(board, source, dest, position)
}

/// Leads the player through executing a mosquito's suck
fn mosquito_prompts<T: Coord>(
    board: &mut Board<T>,
    chip_name: &'static str,
    active_team: Team,
) -> Option<&'static str> {
    // Get mosquitos's position and prompt the user to select a neighbouring chip to suck, returning the coords of the victim
    let position = board.get_position_byname(active_team, chip_name).unwrap();
    let source = match neighbour_prompts(board, position, "suck".to_string()) {
        Some(value) => value,
        None => return None, // abort special move
    };

    // Execute the special move to become the victim for this turn
    match specials::mosquito_suck(board, source, position) {
        Some(value) => Some(value),
        None => {
            println!("Cannot suck from another mosquito!");
            None
        }
    }
}

/// Ask the player to select neighbouring chips from a list (will present colour-coded options 0-5)
fn neighbour_prompts<T: Coord>(board: &mut Board<T>, position: T, movename: String) -> Option<T> {
    let neighbours = board.get_neighbour_chips(position);

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

    // get the co-ordinate of the selected chip and return them
    let source = board.chips.get(&neighbours[selection]).unwrap().unwrap();
    Some(source)
}

/// Request user input into terminal, return a trimmed string
pub fn get_usr_input() -> String {
    let mut textin = String::new();

    io::stdin()
        .read_line(&mut textin)
        .expect("Failed to read line");

    textin.trim().to_string()
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
