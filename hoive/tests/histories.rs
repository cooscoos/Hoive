use hoive::game::board::Board;
use hoive::maths::coord::{Coord, Cube};

mod common;
use common::games::game_snapshot_7;
use hoive::game::{movestatus::MoveStatus, specials};
use hoive::maths::coord::DoubleHeight;

#[test]
fn history_load() {
    // Load up snapshot_7.csv from reference/tests and check that it plays
    // out the same as /tests/common/games.rs pub fn game_snapshot_7

    // load in game from file
    let mut board1 = Board::new(Cube::default());
    let filename = "snapshot_7".to_string();
    common::emulate::emulate(&mut board1, filename, true);

    // now load in the game from the fn
    let board2 = game_snapshot_7();

    // Check they're the same
    assert_eq!(board1, board2);
}

#[test]
fn history_wrong_load() {
    // Load up badsnapshot_7.csv from reference/tests and check that it plays
    // out differently to /tests/common/games.rs pub fn game_snapshot_7

    // load in game from file
    let mut board1 = Board::new(Cube::default());
    let filename = "badsnapshot_7".to_string();
    common::emulate::emulate(&mut board1, filename, true);

    // now load in the game from the fn
    let board2 = game_snapshot_7();

    // Check they're different
    assert_ne!(board1, board2);
}

#[test]
fn history_skipturn() {
    // Check skipping turn works on history. If it does then pillbug sumo will work after a few skipped turns

    // load in game from file
    let mut board = Board::new(Cube::default());
    let filename = "snapshot_13".to_string();
    common::emulate::emulate(&mut board, filename, true);

    // wp1 at 1,1 to try sumo wa1 at 0,2 to destination 2,0
    let position = board.coord.mapfrom_doubleheight(DoubleHeight::from((1, 1)));
    let source = board.coord.mapfrom_doubleheight(DoubleHeight::from((0, 2)));
    let dest = board.coord.mapfrom_doubleheight(DoubleHeight::from((2, 0)));

    assert_eq!(
        MoveStatus::Success,
        specials::pillbug_sumo(&mut board, source, dest, position)
    );
}
