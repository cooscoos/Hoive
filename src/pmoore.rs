// Patrick Moore is the GamesMaster: the human-readable interface between players and the game

use rand::Rng; // To randomise which player goes first
use std::io; // For parsing player inputs

use crate::draw;
use crate::game::board::{Board, MoveStatus};
use crate::game::comps::{other_team, Team};
use crate::maths::coord::{Coord, self};

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
pub fn turn<T: Coord>(mut board: Board<T>, first: Team) -> bool{

    let mut turn_end = false;      // Tells the calling function if this turn has ended

    let active_team = match board.turns % 2 {
        0 => first,
        _ => other_team(first),
    };

    println!("Select a tile to place or move. Type h to see your hand, or t to see the table.");


    let textin = get_usr_input();

    match textin {
        _ if textin == "h" => println!("{}", draw::list_chips(&board, active_team)),
        _ if textin == "t" => println!("{}", draw::show_board(&board, 5)),             // hard-coded 5 here but can adapt based on game extremeties later
        c => {
            let list = match_chip(&board, active_team, c);    // get the available chips
            
            match list {
                Some(value) => {

                // move on
                println!("{}", draw::show_board(&board, 5));
                println!("Select a co-ordinate to move to. Input column then row, separated by a comma, e.g.: 0, 0. Type h to select a different chip.");

                let textin2 = get_usr_input();
                if textin2 == "h" {return false};    // escape this function and start again

                let usr_hex = coord_from_string(textin2);

                // use doubleheight to cubic converter
                let game_hex = board.coord.mapfrom_doubleheight(usr_hex);





                // Try execute the move, if it works then it's the end of the turn
                match try_move(&mut board, value, active_team, game_hex) {
                    MoveStatus::Success => {turn_end = true},
                    _ => (),
                }
                
            },
            None => println!("You don't have this tile in your hand."),
            }
        }

    }
    
    turn_end

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
            return Some(item)
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

    let coord_vec: Vec<i8> = str.trim().split(',').map(|c| c.parse::<i8>().expect("Input numbers!")).collect();
    (coord_vec[0], coord_vec[1])
}

fn xylophone() {

}