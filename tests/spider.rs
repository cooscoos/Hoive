// Tests for the spider
use hoive::game::board::*;
use hoive::game::comps::Team;
use hoive::maths::coord::Coord;
use hoive::pmoore;

mod game_snapshots;
use game_snapshots::{game_snapshot_2, game_snapshot_3};

#[test]
fn spider_move_ok() {
    // Try move a spider 3 spaces (okay).
    let mut board = game_snapshot_2();

    // Place a spider down at (0,2)
    let placement = board.coord.mapfrom_doubleheight((0, 2));
    pmoore::try_move(&mut board, "s1", Team::White, placement);

    // Then try and move it 3 spaces away
    let legal_move = board.coord.mapfrom_doubleheight((1, -3));

    assert_eq!(
        MoveStatus::Success,
        pmoore::try_move(&mut board, "s1", Team::White, legal_move)
    );
}

#[test]
fn spider_move_toofar() {
    // Try move a spider 4 spaces (too far).
    let mut board = game_snapshot_2();

    // Place a spider down at (0,2)
    let placement = board.coord.mapfrom_doubleheight((0, 2));
    pmoore::try_move(&mut board, "s1", Team::White, placement);

    // Then try and move it 4 spaces away
    let illegal_move = board.coord.mapfrom_doubleheight((0, -6));

    assert_eq!(
        MoveStatus::TooFar(3),
        pmoore::try_move(&mut board, "s1", Team::White, illegal_move)
    );
}

#[test]
fn spider_distlim_floodfill() {
    // See how far a spider can travel given a barrier
    let mut board = game_snapshot_3();

    // Spider is already at (0,2) in doubleheight, which is this in cube co-ordinates:
    let cube_pos = board.coord.mapfrom_doubleheight((0, 2));

    // Find the hexes within 3 spaces
    let cube_withinrange = board.dist_lim_floodfill(&cube_pos, 3);

    // Convert back to doubleheight for easier inperpretation
    let d_withinrange = cube_withinrange
        .into_iter()
        .map(|p| board.coord.mapto_doubleheight(p))
        .collect::<Vec<(i8, i8)>>();

    println!("Hexes within range 3 are {:?}", d_withinrange);
}

#[test]
fn spider_through_barrier() {
    // Try move a spider 2 spaces as the crow flies
    // but through a barrier that means it needs to travel 7 spaces (too far).
    let mut board = game_snapshot_3();

    // Spider is already at (0,2)

    // Then try and move it 2 spaces up but through a barrier of other chips
    let illegal_move = board.coord.mapfrom_doubleheight((0, -2));

    assert_eq!(
        MoveStatus::TooFar(3),
        pmoore::try_move(&mut board, "s1", Team::White, illegal_move)
    );
}