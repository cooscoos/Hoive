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

    //println!("Before:  {:?}", board.chips);

    //board.place_chip("s1", Team::Black, (0,0,0));
    println!("turn 1");
    board.move_chip("s1", Team::Black, (1, 0, 0));

    // try place a white chip next to it
    println!("turn 2");
    board.move_chip("s1", Team::White, (0, 1, 0));

    // place black chip next to black chip (okay)
    println!("turn 3");
    board.move_chip("s2", Team::Black, (0, 0, 0));

    // place black chip next to black chip (okay)
    println!("turn 4");
    board.move_chip("s2", Team::White, (1, 1, 0));

    println!("turn 5");
    board.move_chip("s3", Team::White, (1, 1, 1));

    // that's all the chips down, let's try move s3
    board.move_chip("s3", Team::White, (1,1,1));

    // place white chip above black (not ok)
    //board.move_chip("s2", Team::White, (0,0,-2));

    // try do it twice
    //board.move_chip("s2", Team::Black, (0, 0, 0));
}
