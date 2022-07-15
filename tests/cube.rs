// Tests that use cube co-ordinates: cargo test cube

use hoive::coord::Cube;
use hoive::{Board, MoveStatus, Team};

// basic tests that work with all co-ordinate systems
mod basic;


#[test]
fn cube_first_turn() {
    basic::first_turn(&mut Board::default(Cube));
}

#[test]
fn cube_occupied() {
    basic::occupied(&mut Board::default(Cube));
}

#[test]
fn cube_to_the_moon() {
    basic::to_the_moon(&mut Board::default(Cube));
}

//TODO: Finish writing cube tests