use hoive::game::{board::Board, movestatus::MoveStatus};
use hoive::game::{comps::Team, specials};
use hoive::maths::coord::{Coord, Cube, DoubleHeight};
mod common;

fn mosquito_tests_setup(filename: String) -> Board<Cube> {
    // Some set up used by most tests for mosquito

    // Create and emulate a board from a named reference/tests/snapshots file
    let mut board = Board::new(Cube::default());
    common::emulate::emulate(&mut board, filename, true);
    board
}

// used by a few mosquito tests
fn mosquito_beetlemove_one() -> (Board<Cube>, &'static str, Cube) {
    let mut board = mosquito_tests_setup("snapshot_17".to_string());

    // get the black mosquito
    let position = board.get_position_byname(Team::Black, "m1").unwrap();

    // ask it to suck the white beetle
    let source = board.get_position_byname(Team::Black, "b1").unwrap();

    // do the suck
    let newname = specials::mosquito_suck(&mut board, source, position).unwrap();

    (board, newname, source)
}

#[test]
fn mosquito_beetle_top() {
    let (mut board, newname, source) = mosquito_beetlemove_one();

    // now move on top of beetle
    assert_eq!(
        MoveStatus::Success,
        board.move_chip(newname, Team::Black, source)
    );
}

#[test]
fn mosquito_beetle_backdown() {
    let (mut board, newname, source) = mosquito_beetlemove_one();

    // move on top of beetle
    board.move_chip(newname, Team::Black, source);

    // Revert to a mosquito
    specials::mosquito_desuck(&mut board);

    // move to -1,-1
    let legal_move = board
        .coord
        .mapfrom_doubleheight(DoubleHeight::from((-1, -1)));

    assert_eq!(
        MoveStatus::Success,
        board.move_chip("m1", Team::Black, legal_move)
    );
}

#[test]
fn mosquito_ant() {
    let mut board = mosquito_tests_setup("snapshot_18".to_string());

    // get the white mosquito
    let position = board.get_position_byname(Team::White, "m1").unwrap();

    // ask it to suck the white ant 1
    let source = board.get_position_byname(Team::White, "a1").unwrap();

    // do the suck
    let newname = specials::mosquito_suck(&mut board, source, position).unwrap();

    // move to -1,-1
    let legal_move = board
        .coord
        .mapfrom_doubleheight(DoubleHeight::from((-1, -1)));

    assert_eq!(
        MoveStatus::Success,
        board.move_chip(newname, Team::White, legal_move)
    );
}

#[test]
fn mosquito_on_mosquito() {
    let mut board = mosquito_tests_setup("snapshot_19".to_string());

    // get the white mosquito
    let position = board.get_position_byname(Team::White, "m1").unwrap();

    // ask it to suck the black mosquito
    let source = board.get_position_byname(Team::Black, "m1").unwrap();

    // Make sure we return no &str to show the suck failed
    assert_eq!(None, specials::mosquito_suck(&mut board, source, position));
}
