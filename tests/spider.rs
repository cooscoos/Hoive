// Tests for the spider
use hoive::game::board::*;
use hoive::game::comps::Team;
use hoive::maths::{coord::Coord, coord::Cube};
use hoive::pmoore;

mod readyboards;
use readyboards::snapshot_2;


#[test]
fn spider_move_ok() {
    // Try move a spider 3 spaces (okay).
    let mut board = snapshot_2();

    // Place a spider down at (0,2)
    let placement = board.coord.mapfrom_doubleheight((0,2));
    pmoore::try_move(&mut board, "s1", Team::White, placement);
    

    // Then try and move it 3 spaces away
    let legal_move = board.coord.mapfrom_doubleheight((1,-3));

    assert_eq!(
        MoveStatus::Success,
        pmoore::try_move(&mut board, "s1", Team::White, legal_move)
    );

}



#[test]
fn spider_move_toofar() {
    // Try move a spider 4 spaces (too far).
    let mut board = snapshot_2();

    // Place a spider down at (0,2)
    let placement = board.coord.mapfrom_doubleheight((0,2));
    pmoore::try_move(&mut board, "s1", Team::White, placement);

    // Then try and move it 4 spaces away
    let illegal_move = board.coord.mapfrom_doubleheight((0,-6));

    assert_eq!(
        MoveStatus::TooFar(3),
        pmoore::try_move(&mut board, "s1", Team::White, illegal_move)
    );
    
}
