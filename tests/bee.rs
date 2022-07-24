// Tests for the bee
use hoive::game::board::*;
use hoive::game::comps::Team;
use hoive::maths::coord::Coord;
use hoive::pmoore;

mod game_snapshots;
use game_snapshots::game_snapshot_2;


#[test]
fn bee_move_ok() {
    // Try move a bee 1 space (okay).
    let mut board = game_snapshot_2();

    // There's a white bee at 0,0 in this snapshot already

    // Then try and move it 1 space away
    let legal_move = board.coord.mapfrom_doubleheight((1,-1));

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
    let illegal_move = board.coord.mapfrom_doubleheight((1,-3));

    assert_eq!(
        MoveStatus::TooFar(1),
        pmoore::try_move(&mut board, "q1", Team::White, illegal_move)
    );
    
}