// Tests for the ladybird
use hoive::game::comps::Team;
use hoive::game::movestatus::MoveStatus;
use hoive::maths::coord::Coord;
use hoive::maths::coord::DoubleHeight;

mod common;
use common::games::game_snapshot_6;

#[test]
fn ladybird_backtrack() {
    // Try move a ladybird over 2 hexes then back on itself (ok).
    let mut board = game_snapshot_6();

    // There's a white ladybird at 0,-4 in this snapshot already

    let legal_move = board.coord.from_doubleheight(DoubleHeight::from((1, -5)));

    assert_eq!(
        MoveStatus::Success,
        board.move_chip("l1", Team::White, legal_move)
    );
}

#[test]
fn ladybird_advance() {
    // Try move a ladybird over 2 hexes then back on itself (ok).
    let mut board = game_snapshot_6();

    let legal_move = board.coord.from_doubleheight(DoubleHeight::from((0, 2)));

    assert_eq!(
        MoveStatus::Success,
        board.move_chip("l1", Team::White, legal_move)
    );
}

#[test]
fn ladybird_illegal() {
    // Try move a ladybird over to an illegal spot that's too far away
    let mut board = game_snapshot_6();

    let illegal_move = board.coord.from_doubleheight(DoubleHeight::from((4, 0)));

    assert_eq!(
        MoveStatus::BadDistance(3),
        board.move_chip("l1", Team::White, illegal_move)
    );
}
