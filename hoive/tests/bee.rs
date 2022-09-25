// Tests for the bee
use hoive::game::comps::Team;
use hoive::game::{board::Board, movestatus::MoveStatus};
use hoive::maths::coord::{Coord, Cube, DoubleHeight};

mod common;
use common::games::{game_snapshot_2, game_snapshot_4, game_snapshot_5};

#[test]
fn bee_move_ok() {
    // Try move a bee 1 space (okay).
    let mut board = game_snapshot_2();

    // There's a white bee at 0,0 in this snapshot already

    // Then try and move it 1 space away
    let legal_move = board
        .coord
        .mapfrom_doubleheight(DoubleHeight::from((1, -1)));

    assert_eq!(
        MoveStatus::Success,
        board.move_chip("q1", Team::White, legal_move)
    );
}

#[test]
fn bee_move_toofar() {
    // Try move a bee 2 spaces (too far).
    let mut board = game_snapshot_2();

    // There's a white bee at 0,0 in this snapshot already

    // Then try and move it 2 spaces away
    let illegal_move = board
        .coord
        .mapfrom_doubleheight(DoubleHeight::from((1, -3)));

    assert_eq!(
        MoveStatus::BadDistance(1),
        board.move_chip("q1", Team::White, illegal_move)
    );
}

#[test]
fn bee_need() {
    // Place no bees and then continue to have no bee on white player's 4th turn.
    let mut board = Board::new(Cube::default());

    let moves_list = vec![
        (0, 0),   // wa1 turn 1
        (1, -1),  // bq1
        (-1, -1), // wa2 turn 2
        (2, -2),  // ba1
        (-2, -2), // wa3  turn 3
        (2, -4),  // bs1
        (-2, -4), // ws1 turn 4
    ];

    // Convert to cube
    let hex_moves = moves_list
        .iter()
        .map(|xy| board.coord.mapfrom_doubleheight(DoubleHeight::from(*xy)))
        .collect::<Vec<Cube>>();

    board.move_chip("a1", Team::White, hex_moves[0]);
    board.move_chip("q1", Team::Black, hex_moves[1]);
    board.move_chip("a2", Team::White, hex_moves[2]);
    board.move_chip("a1", Team::Black, hex_moves[3]);
    board.move_chip("a3", Team::White, hex_moves[4]);
    board.move_chip("s1", Team::Black, hex_moves[5]);

    assert_eq!(
        MoveStatus::BeeNeed,
        board.move_chip("s1", Team::White, hex_moves[6])
    );
}

#[test]
fn bee_missing() {
    // Place no bees and then try move existing chip
    let mut board = Board::new(Cube::default());

    let moves_list = vec![
        (0, 0),   // wa1
        (1, -1),  // bq1
        (-1, -1), // wa1
    ];

    // Convert to cube
    let hex_moves = moves_list
        .iter()
        .map(|xy| board.coord.mapfrom_doubleheight(DoubleHeight::from(*xy)))
        .collect::<Vec<Cube>>();

    board.move_chip("a1", Team::White, hex_moves[0]);
    board.move_chip("q1", Team::Black, hex_moves[1]);

    assert_eq!(
        MoveStatus::NoBee,
        board.move_chip("a1", Team::White, hex_moves[2])
    );
}

#[test]
fn bee_defeat() {
    // Let the black team defeat white team.
    // This game set up so that we can now move ba2 or wa3 to (1,1) to defeat white team
    let mut board = game_snapshot_4();

    let defeat_move = board.coord.mapfrom_doubleheight(DoubleHeight::from((1, 1)));

    assert_eq!(
        MoveStatus::Win(Some(Team::Black)),
        board.move_chip("a2", Team::Black, defeat_move)
    );
}

#[test]
fn bee_seppuku() {
    // Let the white team defeat itself.
    // This game set up so that we can now move ba2 or wa3 to (1,1) to defeat white team
    let mut board = game_snapshot_4();

    let defeat_move = board.coord.mapfrom_doubleheight(DoubleHeight::from((1, 1)));

    assert_eq!(
        MoveStatus::Win(Some(Team::Black)),
        board.move_chip("a3", Team::White, defeat_move)
    );
}

#[test]
fn bee_spinning_seppuku() {
    // Let the white team defeat itself and opponent simultaneously.
    // This game set up so that we can now move wa3 to (1,1) to defeat both teams at once
    let mut board = game_snapshot_5();

    let defeat_move = board.coord.mapfrom_doubleheight(DoubleHeight::from((1, 1)));

    assert_eq!(
        MoveStatus::Win(None),
        board.move_chip("a3", Team::White, defeat_move)
    );
}
