// Patrick Moore is the GamesMaster: the human-readable interface between players and the game logic

use rand::Rng; // To randomise which player goes first
use std::io; // For parsing player inputs

use crate::draw;
use crate::game::{animals, specials};
use crate::game::board::{Board, MoveStatus};
use crate::game::comps::{other_team, Team};
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
        _ => other_team(first),
    };

    println!("{} team's turn.\n", draw::team_string(active_team));

    // Ask player to select chip
    let mut chip_selection = None;
    while chip_selection == None {
        chip_selection = chip_select(board, active_team)
    }

    let chip_name = chip_selection.unwrap();


    // If we selected a pillbug or mosquito, and it's already on the board, then we need to have the option to do a special move
    if (chip_name == "p1" || chip_name == "m1") && board.get_position_byname(active_team, chip_name).is_some() {
        match ask_special_move() {  // ask the player if they want to do the special move
            // This will either return success and end the turn (by incrementing board.turn), or it'll return a useful error and start this turn again
            true => {println!("SPECIAL");return try_special(board, chip_name);}, 
            false => (), // do nothing, proceed to movement
        }
    }

    let mut coord = None;
    while coord == None {
        coord = coord_select();
    }

    // If the user wants to abort coord selecton and switch pieces, go back to the start
    let select_hex = match coord.unwrap().0 {
        true => return MoveStatus::Nothing,
        false => (coord.unwrap().1, coord.unwrap().2),
    };

    // Convert from doubleheight to the game's co-ordinate system
    let game_hex = board.coord.mapfrom_doubleheight(select_hex);

    // Try execute the move, if it works then show the board. The function try_move will increment the turn itself if move=success
    let return_status = try_move(board, chip_name, active_team, game_hex);

    match return_status {
        MoveStatus::Success => println!("{}\n", draw::show_board(board, 5)),
        MoveStatus::Win(teamopt) => {
            println!("{}\n", draw::show_board(board, 5));

            match teamopt {
                Some(team) => {
                    let team_str = draw::team_string(team);
                    println!("\n << {team_str} team wins. Well done!  >> \n");
                }
                None => {
                    println!("\n << Draw. Both teams have suffered defeat! >> \n");
                }
            }
        }
        _ => (),
    };

    return_status
}

// Return the str of the chip if it matches the query
fn match_chip<T: Coord>(board: &Board<T>, team: Team, name: String) -> Option<&'static str> {
    // Filter out the chips that belong to a given
    let list = board
        .chips
        .clone()
        .into_iter()
        .filter(|(c, _)| c.team == team)
        .map(|(c, _)| c.name)
        .collect::<Vec<&str>>();

    for item in list {
        if item == name {
            return Some(item);
        }
    }
    None
}

// Select a chip and return its static str. Returns None if user input invalid.
fn chip_select<T: Coord>(board: &Board<T>, active_team: Team) -> Option<&'static str> {
    println!("Select a tile from the board or your hand to move. Hit enter to see the board and your hand, or h for help.");

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
        c => {
            let list = match_chip(board, active_team, c); // get the available chips

            match list {
                Some(value) => Some(value),
                None => {
                    println!("You don't have this tile in your hand.");
                    None
                }
            }
        }
    }
}

// Return a co-ordinate (i8,i8). Returns (true, 0, 0) if we want to abort co-ordinate select. None if user input invalid.
fn coord_select() -> Option<(bool, i8, i8)> {
    println!("Select a co-ordinate to move to. Input column then row, separated by a comma, e.g.: 0, 0. Type enter to abort this move.");
    let textin = get_usr_input();
    if textin.is_empty() {
        return Some((true, 0, 0));
    }; // escape this function and start again

    let usr_hex = coord_from_string(textin);

    match usr_hex[..] {
        [Some(x), Some(y)] => {
            match (x+y)%2 { // The sum of doubleheight coords should always be an even no.
                0 => {return Some((false, x, y));},
                _ => {println!("Enter valid co-ordinates."); return None;},
                }},
        _ => {
            println!("Enter two numbers separated by a comma for co-ordinates.");
            None
        }
    }
}

// Try execute a move and provide printscreen feedback
pub fn try_move<T: Coord>(
    board: &mut Board<T>,
    name: &'static str,
    team: Team,
    position: (i8, i8, i8),
) -> MoveStatus {
    let move_status = board.move_chip(name, team, position);

    match move_status {
        MoveStatus::Success => {
            println!("Chip move was successful.");
            board.turns += 1;
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
        MoveStatus::Win(_) => {},
        MoveStatus::Nothing => {},
    }
    move_status
}

// Ask player if they want to do a special move
fn ask_special_move() -> bool {
    println!("Type s to try and execute this chip's special move, or enter to move the chip.");
    let textin = get_usr_input();
    !textin.is_empty() // try execute a special if the user inputs anything
}

// Handles the attempt at doing a special move.
pub fn try_special<T: Coord>(board: &mut Board<T>, chip_name: &'static str) -> MoveStatus {
    // Find out if we're dealing with a mosquito or pillbug and lead the player through the prompts


    // Pillbug, p1
    // Select a neighbouring chip from this list (present options 1-6 for white and black team chips), return source to fn
    // Select a location to sumo to (write to dest)
    // let move_status = pillbug_toss(board, source, dest);
    // if it's success, increment board turns by 1
    // return the movestatus directly
        

    let move_status = specials::pillbug_toss(board, source, dest);

    board.turns += 1;
    MoveStatus::Success

    // mosquito todo, m1

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
Game rules: https://en.wikipedia.org/wiki/Hive_(game)\n
----------------------------------------------------------------
"
}