use std::collections::HashMap;

use Hoive::{Animal, Board, Chip, Player, Team};

fn main() {
    // Start with a 1 player game where we just place pieces correctly
    // Add in a second player to place pieces correctly
    // Then do movement

    // initialise a player
    let mut p1 = Player::default(Team::Black);

    // show the player's hand
    //println!("{:?}", p1.show_hand());

    // show all peices
    //println!("{:?}", p1.show_all());

    // let them place a piece at 0,0,0, use a command to show hand
    //p1.place("s1", (0, 0, 0));

    //println!("{:?}", p1.show_all());

    // pieces should belong to a hashmap that belongs to nobody

    // place another piece, but only at designated locations

    // need a function (hashmap?) that returns the co-ords of all existing b & w pieces

    let mut board = Board::default();

    println!("Before:  {:?}", board.chips);

    //board.place_chip("s1", Team::Black, (0,0,0));
    board.move_chip("s1", Team::Black, (0, 0, 0));

    println!("After move:  {:?}", board.chips);
}
