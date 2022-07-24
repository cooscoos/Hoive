// Tests for the bee
use hoive::game::board::*;
use hoive::game::comps::Team;
use hoive::maths::coord::Coord;
use hoive::pmoore;

mod readyboards;
use readyboards::snapshot_2;


#[test]
fn bee_move_ok() {
    // Try move a bee 1 space (okay).
    let mut board = snapshot_2();

    // Place a bee down at (0,2)
    let placement = board.coord.mapfrom_doubleheight((0,2));
    pmoore::try_move(&mut board, "q1", Team::White, placement);
    

    // Then try and move it 1 space away
    let legal_move = board.coord.mapfrom_doubleheight((-1,1));

    assert_eq!(
        MoveStatus::Success,
        pmoore::try_move(&mut board, "q1", Team::White, legal_move)
    );

}



#[test]
fn bee_move_toofar() {
    // Try move a bee 2 spaces (too far).
    let mut board = snapshot_2();

    // Place a bee down at (0,2)
    let placement = board.coord.mapfrom_doubleheight((0,2));
    pmoore::try_move(&mut board, "q1", Team::White, placement);

    // Then try and move it 2 spaces away
    let illegal_move = board.coord.mapfrom_doubleheight((-1,-1));

    assert_eq!(
        MoveStatus::TooFar(1),
        pmoore::try_move(&mut board, "q1", Team::White, illegal_move)
    );
    
}