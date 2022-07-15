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




// These tests are hecs specific
#[test]
fn cube_second_turn_neighbour() {
    // Place a white chip next to a black chip but on the second turn (should be okay)
    let mut board = Board::default(Cube);
    board.try_move("s1", Team::Black, (0, 0, 0));
    assert_eq!(
        MoveStatus::Success,
        board.try_move("s1", Team::White, (1, 0, -1))
    );
}

#[test]
fn cube_third_turn_badneighbour() {
    // Place a white chip next to a black chip on the third turn (that's illegal)
    let mut board = Board::default(Cube);
    board.try_move("s1", Team::Black, (0, 0, 0));
    board.try_move("s1", Team::White, (1, 0, -1));
    assert_eq!(
        MoveStatus::BadNeighbour,
        board.try_move("s2", Team::White, (-1, 0, 1))
    );
}

#[test]
fn cube_fifth_turn_badneighbour() {
    // Do a bunch of legal stuff with a BadNeighbour move at the end
    let mut board = Board::default(Cube);
    board.try_move("s1", Team::Black, (0, 0, 0));
    board.try_move("s1", Team::White, (0, -1, 1));
    board.try_move("s2", Team::Black, (0, 1, -1));
    board.try_move("s2", Team::White, (0, -2, 2));

    assert_eq!(
        MoveStatus::BadNeighbour,
        board.try_move("s3", Team::Black, (1, -3, 2))
    );
}

#[test]
fn cube_split_hive() {
    // Put down four chips and then split the hive by moving a white spider from the middle
    let mut board = Board::default(Cube);
    board.try_move("s1", Team::Black, (0, 0, 0));
    board.try_move("s1", Team::White, (0, -1, 1));
    board.try_move("s2", Team::Black, (0, 1, -1));
    board.try_move("s2", Team::White, (0, 2, -2));


    assert_eq!(
        MoveStatus::HiveSplit,
        board.try_move("s1", Team::Black, (0, 2, -2))
    );
}

#[test]
fn cube_attack() {
    // Put down lots of chips and then relocate a black next to black after turn 6
    // We haven't coded logic for bee allowing move yet, so we'll need to rewrite this test then
    let mut board = Board::default(Cube);
    board.try_move("s1", Team::Black, (0, 0, 0));
    board.try_move("s1", Team::White, (0, -1, 1));
    board.try_move("s1", Team::Black, (0, 1, -1));
    board.try_move("s2", Team::White, (1, -2, 1));
    board.try_move("s1", Team::Black, (1, 1, -2));
    board.try_move("s3", Team::White, (0, -2, 2));


    assert_eq!(
        MoveStatus::Success,
        board.try_move("s1", Team::Black, (1, -3, 2)));
        
}


#[test]
fn cube_nosplit_hive() {
    // Put down lots of chips and then do a move that doesn't split hive and is legal
    let mut board = Board::default(Cube);
    board.try_move("s1", Team::Black, (0, 0, 0));
    board.try_move("s1", Team::White, (0, -1, 1));
    board.try_move("s1", Team::Black, (0, 1, -1));
    board.try_move("s2", Team::White, (1, -2, 1));
    board.try_move("s1", Team::Black, (1, 1, -2));
    board.try_move("s3", Team::White, (0, -2, 2));
    board.try_move("s1", Team::Black, (1, -3, 2));


    assert_eq!(
        MoveStatus::Success,
        board.try_move("s3", Team::White, (-1, -1, 2))
    );
}

