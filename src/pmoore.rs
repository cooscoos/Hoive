// Patrick Moore is the GamesMaster: the human-readable interface between players and the game

use rand::Rng; // To randomise which player goes first
use std::io; // For parsing player inputs

use crate::draw;
use crate::game::board::{Board, MoveStatus};
use crate::game::comps::{other_team, Team};
use crate::maths::coord::{self, Coord};

// Say hello and define who goes first
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
pub fn turn<T: Coord>(board: &mut Board<T>, first: Team) {
    let active_team = match board.turns % 2 {
        0 => first,
        _ => other_team(first),
    };

    println!("{} team's turn.\n", draw::team_string(active_team));

    println!("Select a tile to place or move. Type enter to see the board and your hand.");

    let textin = get_usr_input();

    match textin {
        _ if textin == "" => println!("{}\n Hand:{}\n", draw::show_board(&board, 5),draw::list_chips(&board, active_team)), // hard-coded 5 here but can adapt based on game extremeties later
        _ if textin == "xylophone" => xylophone(),
        c => {
            let list = match_chip(&board, active_team, c); // get the available chips

            match list {
                Some(value) => {
                    // move on
                    println!("Select a co-ordinate to move to. Input column then row, separated by a comma, e.g.: 0, 0. Type enter to abort this move.");

                    let textin2 = get_usr_input();
                    if textin2 == "" {
                        return ();
                    }; // escape this function and start again

                    let usr_hex = coord_from_string(textin2);

                    // use doubleheight to cubic converter
                    let game_hex = board.coord.mapfrom_doubleheight(usr_hex);

                    // Try execute the move, if it works then it's the end of the turn
                    try_move(board, value, active_team, game_hex);
                    println!("{}\n", draw::show_board(&board, 5));
                }
                None => println!("You don't have this tile in your hand."),
            }
        }
    }
}

// Return the str of the chip if it matches the query
pub fn match_chip<T: Coord>(board: &Board<T>, team: Team, name: String) -> Option<&'static str> {
    // Filter out the chips that are hand of given team (in hand  position = None)
    let list = board
        .chips
        .clone()
        .into_iter()
        .filter(|(c, p)| (p.is_none()) & (c.team == team))
        .map(|(c, _)| c.name)
        .collect::<Vec<&str>>();

    for item in list {
        if item == name {
            return Some(item);
        }
    }
    None
}

// For now, this guy handles the MoveStatus enum and provides some printscreen feedback
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
        MoveStatus::BadNeighbour => println!("Can't place a new chip next to other team."),
        MoveStatus::HiveSplit => println!("No: this move would split the hive in two."),
        MoveStatus::Occupied => println!("Can't move this chip to an occupied position."),
        MoveStatus::Unconnected => println!("Can't move your chip to an unconnected position."),
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

// Parse comma separated values in a str to a doubleheight co-ordinate
fn coord_from_string(str: String) -> (i8, i8) {
    let coord_vec: Vec<i8> = str
        .trim()
        .split(',')
        .map(|c| c.parse::<i8>().expect("Input numbers!"))
        .collect();
    (coord_vec[0], coord_vec[1])
}

fn xylophone() {
    let egg = "                                     \n    ....,,..                        \n    .......',;cllllooll:'.                     \n  ..;cccccodxxkkkOkkkxxxol;.                   \n .:ooooooddxkkkkOOOOOOOkxxdl,                  \n.cdddoooddxxkkkkOO0000OOOOkkx:.                \n'loddolooddxkkOOO00000000OOOO0x.                \n.;ldxdlllodxxkOO000KKKK0000OOO0x'                \n,codo::clodxkOO00KKKKKKKK00Okkk:                \n.,::;,;cldxkkOO00000KKK0000OkkOl.               \n.','.';:ldxxkOOO0000OO000O0OOkxo,               \n....',;:loxxkkkkOOkkOO00O0OOkxxd'              \n.....';cclodkkOkkkkOOO00OOOOkxxd'              \n .  ...,:looodkkkxxxkkkkkkkkxxxo.              \n .   .'';ldoodoolloddddddddoxxxo.              \n     ....,,',,;;::ldollccldxOkxo.              \n    .....'',::c::ox0OkdxxkOO0Oxl.              \n    ..'';:cllddc:lx0OOOOOO0Okxdl.              \n   ....';clcldl,;lx0OkxkOOOkdddc.              \n  ..   ..,cool'.;ld0Okdoddxxxxdl.              \n  .. ....':c:,...,:ldxkkdlodxxxo'              \n  .......',,,,....':dkOOkxdxxdxl.              \n  ......,::,.''';:coooddddddddd,.              \n  .......,cc,',,,,;:ccldxdllll;.               \n............','.,;::cloodddoc:c:.                \n.;;;'..''........',::clcclllooxo'                .\n.oxddl,.','''.......''''',:ldOKO:                 .\n.d0000ko;',:c;.    .....,ldOKNWNd.                 .\n.oKKXKX0kkcco;. ......;:;,oXWWMNk;.                 .\n.o00KKXX0O00Od,...''cxKNNXkldXWWO:'.                  \n'd00000KXKKXKkc.....c0NWWMWNK0X0x:,;.                  \n,d000KKKXXXXOl.',. .oXNNWWMWXXXKd'':;.                  \n.:OKKXXXXXNX0l.  ....:KWNWWWWWNX0d;.;:,.                  \n.o0XKXXXXXXKo'      .;ONNNWWWWNXKx:.,;;'.                  \n,xXXXXXXXKxl'   ..   .dNNNNNWNXXKkc'',''.                   \nOXXXXXXXk, ...   .    ;kKXNNNNXXKx,......                   \n0KKKXNXKd.   ...     ..cdxk0KNNN0l. ..',.                   \nkkxkkO00Ol.       ...  .lkxddddl,....:k0l.                  \n..';ldxx0k, .      .'.  :0Ol'. ..'. ;OXXx.";
    println!("{egg}");
}
