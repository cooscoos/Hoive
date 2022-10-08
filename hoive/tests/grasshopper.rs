use hoive::game::comps::Team;
use hoive::game::{board::Board, movestatus::MoveStatus};
use hoive::maths::coord::DoubleHeight;
use hoive::maths::coord::{Coord, Cube};
mod common;

fn ghop_tests_setup(filename: String) -> Board<Cube> {
    // Some set up used by most tests for grasshopper

    // Create and emulate a board from a named reference/tests/snapshots file
    let mut board = Board::new(Cube::default());
    common::emulate::emulate(&mut board, filename, true);
    board
}

#[test]
fn grasshopper_longjump() {
    // Black g1 is going to jump from 0,4 to 0,-4 (a straight line). Should be okay
    let mut board = ghop_tests_setup("snapshot_16".to_string());

    let legal_move = board
        .coord
        .mapfrom_doubleheight(DoubleHeight::from((0, -4)));

    assert_eq!(
        MoveStatus::Success,
        board.move_chip("g1", Team::Black, legal_move)
    );
}

#[test]
fn grasshopper_shortjump() {
    // White g1 is going to jump from -1,-1 to 1,-3 (a short straight line over one chip). Should be okay
    let mut board = ghop_tests_setup("snapshot_16".to_string());

    let legal_move = board
        .coord
        .mapfrom_doubleheight(DoubleHeight::from((1, -3)));

    assert_eq!(
        MoveStatus::Success,
        board.move_chip("g1", Team::White, legal_move)
    );
}

#[test]
fn grasshopper_badjump() {
    // Black g1 is going to jump from 0,4 to 1,3 (a straight line, but an immediate neighbour). Not okay
    let mut board = ghop_tests_setup("snapshot_16".to_string());

    let bad_move = board.coord.mapfrom_doubleheight(DoubleHeight::from((1, 3)));

    assert_eq!(
        MoveStatus::NoJump,
        board.move_chip("g1", Team::Black, bad_move)
    );
}

#[test]
fn grasshopper_noline() {
    // Black g1 is going to jump from 0,4 to 1,-4 (not a straight line). Not okay
    let mut board = ghop_tests_setup("snapshot_16".to_string());

    let bad_move = board
        .coord
        .mapfrom_doubleheight(DoubleHeight::from((1, -1)));

    assert_eq!(
        MoveStatus::NoJump,
        board.move_chip("g1", Team::Black, bad_move)
    );
}
