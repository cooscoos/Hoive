use hoive::draw;
use hoive::game::{board::Board, history};
use hoive::maths::coord::Cube;

mod common;
use common::games::game_snapshot_7;

#[test]
fn history_load() {
    // Load up snapshot_7.csv from reference/tests and check that it plays
    // out the same as /tests/common/games.rs pub fn game_snapshot_7

    // load in game from file
    let mut board1 = Board::default(Cube);
    let filename = "snapshot_7".to_string();
    history::emulate(&mut board1, filename, true);

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
    let mut board1 = Board::default(Cube);
    let filename = "badsnapshot_7".to_string();
    history::emulate(&mut board1, filename, true);

    // now load in the game from the fn
    let board2 = game_snapshot_7();

    // Check they're different
    assert_ne!(board1, board2);
}
