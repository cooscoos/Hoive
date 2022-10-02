use hoive::game::{board::Board, movestatus::MoveStatus};
use hoive::game::{comps::Chip, comps::Team, specials};
use hoive::maths::coord::DoubleHeight;
use hoive::maths::coord::{Coord, Cube};
mod common;

fn pillbug_tests_setup(filename: String) -> Board<Cube> {
    // Some set up used by most tests for pillbug

    // Create and emulate a board from a named reference/tests/snapshots file
    let mut board = Board::new(Cube::default());
    common::emulate::emulate(&mut board, filename, true);
    board
}

#[test]
fn pillbug_me_too_soon() {
    // White pillbug p1 tries to sumo too soon after moving itself
    let mut board = pillbug_tests_setup("snapshot_8".to_string());

    let pillchip = Chip {
        name: "p1",
        team: Team::White,
    };

    // p1 (0,0) to try sumo q1 (0,-2) to 0,2 should cause RecentMove(p1)
    // Map everything to board's co-ordinate system
    let position = board.coord.mapfrom_doubleheight(DoubleHeight::from((0, 0)));
    let source = board
        .coord
        .mapfrom_doubleheight(DoubleHeight::from((0, -2)));
    let dest = board.coord.mapfrom_doubleheight(DoubleHeight::from((0, 2)));

    assert_eq!(
        MoveStatus::RecentMove(pillchip.name.to_string()),
        specials::pillbug_sumo(&mut board, source, dest, position)
    );
}

#[test]
fn pillbug_you_too_soon() {
    // Pillbug tries to sumo a chip too soon after it has moved
    let mut board = pillbug_tests_setup("snapshot_9".to_string());

    let antchip = Chip {
        name: "a1",
        team: Team::Black,
    };

    // p1 (0,0) to try sumo ba1 (1,-1) to -1,-1 should cause RecentMove(ba1)
    let position = board.coord.mapfrom_doubleheight(DoubleHeight::from((0, 0)));
    let source = board
        .coord
        .mapfrom_doubleheight(DoubleHeight::from((1, -1)));
    let dest = board
        .coord
        .mapfrom_doubleheight(DoubleHeight::from((-1, -1)));

    assert_eq!(
        MoveStatus::RecentMove(antchip.name.to_string()),
        specials::pillbug_sumo(&mut board, source, dest, position)
    );
}

#[test]
fn pillbug_hivebreak() {
    // Pillbug sumo breaks hive
    let mut board = pillbug_tests_setup("snapshot_9".to_string());

    // p1 (0,0) to try sumo wa1 (-1,1) to 1,1 should cause hive break
    let position = board.coord.mapfrom_doubleheight(DoubleHeight::from((0, 0)));
    let source = board
        .coord
        .mapfrom_doubleheight(DoubleHeight::from((-1, 1)));
    let dest = board.coord.mapfrom_doubleheight(DoubleHeight::from((1, 1)));

    assert_eq!(
        MoveStatus::HiveSplit,
        specials::pillbug_sumo(&mut board, source, dest, position)
    );
}

#[test]
fn pillbug_no_hivebreak() {
    // Pillbug sumo doesn't break hive and is okay
    let mut board = pillbug_tests_setup("snapshot_9".to_string());

    // p1 (0,0) to try sumo wa1 (-1,1) to 0,2 should be successful
    let position = board.coord.mapfrom_doubleheight(DoubleHeight::from((0, 0)));
    let source = board
        .coord
        .mapfrom_doubleheight(DoubleHeight::from((-1, 1)));
    let dest = board.coord.mapfrom_doubleheight(DoubleHeight::from((0, 2)));

    assert_eq!(
        MoveStatus::Success,
        specials::pillbug_sumo(&mut board, source, dest, position)
    );
}

#[test]
fn pillbug_non_neighbouring() {
    // Pillbug attempts to sumo into a non-neighbouring hex
    let mut board = pillbug_tests_setup("snapshot_10".to_string());

    // p1 (0,0) to try sumo wa1 (-1,1) to 2,0 should return "not neighbour"
    let position = board.coord.mapfrom_doubleheight(DoubleHeight::from((0, 0)));
    let source = board
        .coord
        .mapfrom_doubleheight(DoubleHeight::from((-1, 1)));
    let dest = board.coord.mapfrom_doubleheight(DoubleHeight::from((2, 0)));

    assert_eq!(
        MoveStatus::NotNeighbour,
        specials::pillbug_sumo(&mut board, source, dest, position)
    );
}

#[test]
fn pillbug_stacksumo() {
    // Pillbug attempts to sumo a beetle from one layer above (on a stack)
    let mut board = pillbug_tests_setup("snapshot_14".to_string());

    // use wp1 at 0,-4 to sumo bb1 at 0,-2 (layer 1) to 1,-5: should return not-neighbour
    let position = board
        .coord
        .mapfrom_doubleheight(DoubleHeight::from((0, -4)));
    let source = board
        .coord
        .mapfrom_doubleheight(DoubleHeight::new(0, -2, 1));
    let dest = board
        .coord
        .mapfrom_doubleheight(DoubleHeight::from((1, -5)));

    assert_eq!(
        MoveStatus::NotNeighbour,
        specials::pillbug_sumo(&mut board, source, dest, position)
    );
}

#[test]
fn pillbug_under_stacksumo() {
    // Pillbug attempts to sumo a qeen from the bottom of a beetle stack
    let mut board = pillbug_tests_setup("snapshot_14".to_string());

    // use wp1 at 0,-4 to sumo wq1 at 0,-2 (layer 0): should return beetle-stack
    let position = board
        .coord
        .mapfrom_doubleheight(DoubleHeight::from((0, -4)));
    let source = board
        .coord
        .mapfrom_doubleheight(DoubleHeight::from((0, -2)));
    let dest = board
        .coord
        .mapfrom_doubleheight(DoubleHeight::from((1, -5)));

    assert_eq!(
        MoveStatus::BeetleBlock,
        specials::pillbug_sumo(&mut board, source, dest, position)
    );
}

#[test]
fn sumo_through_beetlegate() {
    // Pillbug attempts to sumo an ant through a beetle gate
    let mut board = pillbug_tests_setup("snapshot_15".to_string());

    // use wp1 at 0,-2 to sumo ba1 at 0,0 to 0,-4 should not be allowed because of beetle gate
    let position = board
        .coord
        .mapfrom_doubleheight(DoubleHeight::from((0, -2)));
    let source = board.coord.mapfrom_doubleheight(DoubleHeight::from((0, 0)));
    let dest = board
        .coord
        .mapfrom_doubleheight(DoubleHeight::from((0, -4)));

    assert_eq!(
        MoveStatus::BeetleGate,
        specials::pillbug_sumo(&mut board, source, dest, position)
    );
}
