// Tests for the ladybird
use hoive::game::board::*;
use hoive::game::comps::Team;
use hoive::maths::coord::{Coord, Cube};
use hoive::pmoore;

mod common;
use common::games::game_snapshot_6;

#[test]
fn ladybird_backtrack() {
    // Try move a ladybird over 2 hexes then back on itself (ok).
    let mut board = game_snapshot_6();

    // There's a white ladybird at 0,-4 in this snapshot already

    let legal_move = board.coord.mapfrom_doubleheight((1, -5));

    assert_eq!(
        MoveStatus::Success,
        pmoore::try_move(&mut board, "l1", Team::White, legal_move)
    );
}

#[test]
fn ladybird_advance() {
    // Try move a ladybird over 2 hexes then back on itself (ok).
    let mut board = game_snapshot_6();

    let legal_move = board.coord.mapfrom_doubleheight((0, 2));

    assert_eq!(
        MoveStatus::Success,
        pmoore::try_move(&mut board, "l1", Team::White, legal_move)
    );
}

#[test]
fn ladybird_illegal() {
    // Try move a ladybird over to an illegal spot that's too far away
    let mut board = game_snapshot_6();

    let illegal_move = board.coord.mapfrom_doubleheight((4, 0));

    assert_eq!(
        MoveStatus::BadDistance(3),
        pmoore::try_move(&mut board, "l1", Team::White, illegal_move)
    );
}
