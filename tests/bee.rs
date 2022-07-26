// Tests for the bee
use hoive::game::board::*;
use hoive::game::comps::Team;
use hoive::maths::coord::{Coord, Cube};
use hoive::pmoore;

mod common;
use common::games::{game_snapshot_2, game_snapshot_4, game_snapshot_5};

#[test]
fn bee_move_ok() {
    // Try move a bee 1 space (okay).
    let mut board = game_snapshot_2();

    // There's a white bee at 0,0 in this snapshot already

    // Then try and move it 1 space away
    let legal_move = board.coord.mapfrom_doubleheight((1, -1));

    assert_eq!(
        MoveStatus::Success,
        pmoore::try_move(&mut board, "q1", Team::White, legal_move)
    );
}

#[test]
fn bee_move_toofar() {
    // Try move a bee 2 spaces (too far).
    let mut board = game_snapshot_2();

    // There's a white bee at 0,0 in this snapshot already

    // Then try and move it 2 spaces away
    let illegal_move = board.coord.mapfrom_doubleheight((1, -3));

    assert_eq!(
        MoveStatus::BadDistance(1),
        pmoore::try_move(&mut board, "q1", Team::White, illegal_move)
    );
}

#[test]
fn bee_need() {
    // Place no bees and then continue to have no bee on white player's 3rd turn.
    let mut board = Board::default(Cube);

    let moves_list = vec![
        (0, 0),   // wa1
        (1, -1),  // bq1
        (-1, -1), // wa2
        (2, -2),  // ba1
        (-2, -2), // wa3
    ];

    // Convert to cube
    let hex_moves = moves_list
        .iter()
        .map(|xy| board.coord.mapfrom_doubleheight(*xy))
        .collect::<Vec<(i8, i8, i8)>>();

    pmoore::try_move(&mut board, "a1", Team::White, hex_moves[0]);
    pmoore::try_move(&mut board, "q1", Team::Black, hex_moves[1]);
    pmoore::try_move(&mut board, "a2", Team::White, hex_moves[2]);
    pmoore::try_move(&mut board, "a1", Team::Black, hex_moves[3]);

    assert_eq!(
        MoveStatus::BeeNeed,
        pmoore::try_move(&mut board, "a3", Team::White, hex_moves[4])
    );
}

#[test]
fn bee_missing() {
    // Place no bees and then try move existing chip
    let mut board = Board::default(Cube);

    let moves_list = vec![
        (0, 0),   // wa1
        (1, -1),  // bq1
        (-1, -1), // wa1
    ];

    // Convert to cube
    let hex_moves = moves_list
        .iter()
        .map(|xy| board.coord.mapfrom_doubleheight(*xy))
        .collect::<Vec<(i8, i8, i8)>>();

    pmoore::try_move(&mut board, "a1", Team::White, hex_moves[0]);
    pmoore::try_move(&mut board, "q1", Team::Black, hex_moves[1]);

    assert_eq!(
        MoveStatus::NoBee,
        pmoore::try_move(&mut board, "a1", Team::White, hex_moves[2])
    );
}

#[test]
fn bee_defeat() {
    // Let the black team defeat white team.
    // This game set up so that we can now move ba2 or wa3 to (1,1) to defeat white team
    let mut board = game_snapshot_4();

    let defeat_move = board.coord.mapfrom_doubleheight((1, 1));

    assert_eq!(
        MoveStatus::Win(Some(Team::Black)),
        pmoore::try_move(&mut board, "a2", Team::Black, defeat_move)
    );
}

#[test]
fn bee_seppuku() {
    // Let the white team defeat itself.
    // This game set up so that we can now move ba2 or wa3 to (1,1) to defeat white team
    let mut board = game_snapshot_4();

    let defeat_move = board.coord.mapfrom_doubleheight((1, 1));

    assert_eq!(
        MoveStatus::Win(Some(Team::Black)),
        pmoore::try_move(&mut board, "a3", Team::White, defeat_move)
    );
}

#[test]
fn bee_spinning_seppuku() {
    // Let the white team defeat itself and opponent simultaneously.
    // This game set up so that we can now move wa3 to (1,1) to defeat both teams at once
    let mut board = game_snapshot_5();

    let defeat_move = board.coord.mapfrom_doubleheight((1, 1));

    assert_eq!(
        MoveStatus::Win(None),
        pmoore::try_move(&mut board, "a3", Team::White, defeat_move)
    );
}
