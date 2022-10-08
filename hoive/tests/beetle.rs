use hoive::game::comps::Team;
use hoive::game::{board::Board, movestatus::MoveStatus};
use hoive::maths::coord::{Coord, Cube, DoubleHeight};
mod common;

fn beetle_test_setup(filename: String) -> Board<Cube> {
    // Some set up used by most tests for beetle

    // Create and emulate a board from a named snapshots file
    let mut board = Board::new(Cube::default());
    common::emulate::emulate(&mut board, filename, true);
    board
}

#[test]
fn beetle_bad_neigbour() {
    // Black ant tries to go next to white beetle which is on top of its black bee
    let mut board = beetle_test_setup("snapshot_11".to_string());

    let bad_move = board.coord.mapfrom_doubleheight(DoubleHeight::from((1, 3)));

    assert_eq!(
        MoveStatus::BadNeighbour,
        board.move_chip("a2", Team::Black, bad_move)
    );
}

#[test]
fn beetle_layer_1() {
    // Check white beetle layer is 1
    let board = beetle_test_setup("snapshot_11".to_string());

    let position = board.get_position_byname(Team::White, "b1").unwrap();

    assert_eq!(1, position.get_layer());
}

#[test]
fn beetle_layer_2() {
    // Put black beetle on white beetle and check if layer 2
    let mut board = beetle_test_setup("snapshot_11".to_string());

    let stack_up = board.coord.mapfrom_doubleheight(DoubleHeight::from((0, 2)));

    // execute move
    board.move_chip("b1", Team::Black, stack_up);

    let position = board.get_position_byname(Team::Black, "b1").unwrap();

    assert_eq!(2, position.get_layer());
}

#[test]
fn beetle_layer_0() {
    // Put black beetle on white beetle, then move it to an empty position and check layer is 0
    let mut board = beetle_test_setup("snapshot_11".to_string());

    let stack_up = board.coord.mapfrom_doubleheight(DoubleHeight::from((0, 2)));
    board.move_chip("b1", Team::Black, stack_up);

    // now move to empty
    let empty_move = board.coord.mapfrom_doubleheight(DoubleHeight::from((1, 3)));
    board.move_chip("b1", Team::Black, empty_move);

    let position = board.get_position_byname(Team::Black, "b1").unwrap();

    assert_eq!(0, position.get_layer());
}

#[test]
fn beetle_stop_move() {
    // Put black beetle on white beetle and ensure white beetle can't move
    let mut board = beetle_test_setup("snapshot_11".to_string());

    let stack_up = board.coord.mapfrom_doubleheight(DoubleHeight::from((0, 2)));
    board.move_chip("b1", Team::Black, stack_up);

    let bad_move = board.coord.mapfrom_doubleheight(DoubleHeight::from((1, 3)));

    assert_eq!(
        MoveStatus::BeetleBlock,
        board.move_chip("b1", Team::White, bad_move)
    );
}

#[test]
fn beetle_small_gap() {
    // Try fit wb1 through a gap that's too small
    let mut board = beetle_test_setup("snapshot_12".to_string());

    let bad_move = board
        .coord
        .mapfrom_doubleheight(DoubleHeight::from((-1, -1)));

    assert_eq!(
        MoveStatus::SmallGap,
        board.move_chip("b1", Team::White, bad_move)
    );
}

#[test]
fn beetle_gate() {
    // Try move bb2 to 0,-2 (fine) then through a beetle gate to 0,0 (not fine)
    let mut board = beetle_test_setup("snapshot_15".to_string());

    let okay_move = board
        .coord
        .mapfrom_doubleheight(DoubleHeight::from((0, -2)));

    board.move_chip("b2", Team::Black, okay_move);

    let bad_move = board.coord.mapfrom_doubleheight(DoubleHeight::from((0, 0)));

    assert_eq!(
        MoveStatus::SmallGap,
        board.move_chip("b2", Team::Black, bad_move)
    );
}


#[test]
fn beetle_too_far_same_layer() {
    // Try move wb1 from 0,-2 to 0,4 (too far)
    let mut board = beetle_test_setup("snapshot_20".to_string());

    let too_far = board
        .coord
        .mapfrom_doubleheight(DoubleHeight::from((0, 4)));


    assert_eq!(
        MoveStatus::BadDistance(1),
        board.move_chip("b1", Team::White, too_far)
    );
}

#[test]
fn beetle_too_far_up_layer() {
    // Try move wb1 from 0,-2 to 0,2 (too far and up one layer)
    let mut board = beetle_test_setup("snapshot_20".to_string());

    let too_far = board
        .coord
        .mapfrom_doubleheight(DoubleHeight::from((0, 2)));


    assert_eq!(
        MoveStatus::BadDistance(1),
        board.move_chip("b1", Team::White, too_far)
    );
}
