// Patrick Moore is the GamesMaster: the human-readable interface between players and the game

use rand::Rng; // To randomise which player goes first
use std::io; // For parsing player inputs

use crate::draw;
use crate::game::board::Board;
use crate::game::comps::{other_team, Team};
use crate::maths::coord::Coord;

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
pub fn turn<T: Coord>(board: Board<T>, first: Team) {
    let active_team = match board.turns % 2 {
        0 => first,
        _ => other_team(first),
    };

    println!("Select a tile to place or move. Type t to see list of tiles in your hand");

    let mut textin = String::new();

    io::stdin()
        .read_line(&mut textin)
        .expect("Failed to read line");

    let textin = textin.trim();

    match textin {
        "t" => println!("{}", draw::list_chips(&board, active_team)), // Todo, make human readable, split out into on-board / in-hand
        _ => println!("Unrecognised input."),
    }
    println!("You guessed: {textin}");
}
