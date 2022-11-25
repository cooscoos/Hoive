// Tests for the spider
use hoive::game::animals;
use hoive::game::board::Board;
use hoive::game::comps::Team;
use hoive::game::movestatus::MoveStatus;
use hoive::maths::coord::Coord;
use hoive::maths::coord::Cube;
use hoive::maths::coord::DoubleHeight;

mod common;
use common::games::{game_snapshot_2, game_snapshot_3};

/// Helper function to move spider from initial position (snapshot_23) to a given cube coordinate
fn move_spider_test(m: Cube) -> MoveStatus {
    let mut board = common::emulate::load_board("snapshot_23".to_string());
    board.move_chip("s1", Team::White, m)
}

#[test]
fn spider_move_ok() {
    // Try move a spider 3 spaces (okay).
    let mut board = game_snapshot_2();

    // Place a spider down at (0,2)
    let placement = board.coord.mapfrom_doubleheight(DoubleHeight::from((0, 2)));
    board.move_chip("s1", Team::White, placement);

    // Then try and move it 3 spaces away
    let legal_move = board
        .coord
        .mapfrom_doubleheight(DoubleHeight::from((1, -3)));

    assert_eq!(
        MoveStatus::Success,
        board.move_chip("s1", Team::White, legal_move)
    );
}

#[test]
fn spider_move_toofar() {
    // Try move a spider 4 spaces (too far).
    let mut board = game_snapshot_2();

    // Place a spider down at (0,2)
    let placement = board.coord.mapfrom_doubleheight(DoubleHeight::from((0, 2)));
    board.move_chip("s1", Team::White, placement);

    // Then try and move it 4 spaces away
    let illegal_move = board
        .coord
        .mapfrom_doubleheight(DoubleHeight::from((0, -6)));

    assert_eq!(
        MoveStatus::BadDistance(3),
        board.move_chip("s1", Team::White, illegal_move)
    );
}

#[test]
fn spider_distlim_floodfill() {
    // See how far a spider can travel given a barrier
    let board = game_snapshot_3();

    // Spider is already at (0,2) in doubleheight, which is this in cube co-ordinates:
    let cube_pos = board.coord.mapfrom_doubleheight(DoubleHeight::from((0, 2)));

    // Movement rules for a spider are "can it go on top of other obstacles this move?" The answer is always false.
    let move_rules = vec![false, false, false];
    // Find the hexes within 3 spaces
    let cube_withinrange = animals::mod_dist_lim_floodfill(&board, &cube_pos, move_rules);

    // Convert back to doubleheight for easier inperpretation
    let d_withinrange = cube_withinrange
        .into_iter()
        .map(|p| board.coord.to_doubleheight(p))
        .collect::<Vec<DoubleHeight>>();

    println!("Hexes within range 3 are {:?}", d_withinrange);
}

#[test]
fn spider_through_barrier() {
    // Try move a spider 2 spaces as the crow flies
    // but through a barrier that means it needs to travel 7 spaces (too far).
    let mut board = game_snapshot_3();

    // Spider is already at (0,2)

    // Then try and move it 2 spaces up but through a barrier of other chips
    let illegal_move = board
        .coord
        .mapfrom_doubleheight(DoubleHeight::from((0, -2)));

    assert_eq!(
        MoveStatus::BadDistance(3),
        board.move_chip("s1", Team::White, illegal_move)
    );
}

#[test]
fn spider_move_ok2() {
    // Make a legal move using snapshot_23
    let mut board = common::emulate::load_board("snapshot_23".to_string());

    let legal_move = board.coord.mapfrom_doubleheight(DoubleHeight::from((1, 3)));

    assert_eq!(
        MoveStatus::Success,
        board.move_chip("s1", Team::White, legal_move)
    );
}

#[test]
fn spider_move_ok3() {
    // Make another legal move using snapshot_23
    let mut board = common::emulate::load_board("snapshot_23".to_string());

    let legal_move = board
        .coord
        .mapfrom_doubleheight(DoubleHeight::from((-1, 3)));

    assert_eq!(
        MoveStatus::Success,
        board.move_chip("s1", Team::White, legal_move)
    );
}

#[test]
fn spider_illegal_move() {
    // Make a bunch of illegal moves, check they all return BadDistance(3)

    // All of these doubleheight moves are illegal from snapshot_23 because the distance is !=3.
    let illegal_moves = [(1, -1), (1, 1), (-1, -1), (-1, 1), (0, 4)];

    // Convert all dheight to cube coordinates
    let board = Board::<Cube>::default();
    let moves = illegal_moves
        .into_iter()
        .map(|m| board.coord.mapfrom_doubleheight(DoubleHeight::from(m)))
        .collect::<Vec<Cube>>();

    // Do all of those moves (not sequentially, but from initial position) and get the movestatuses back
    let move_statuses = moves
        .into_iter()
        .map(|m| move_spider_test(m))
        .collect::<Vec<MoveStatus>>();

    println!("Move statuses of illegal moves: {:?}", move_statuses);

    // Ensure they're all returning BadDistance(3)
    assert_eq!(
        move_statuses
            .into_iter()
            .all(|f| f == MoveStatus::BadDistance(3)),
        true
    );
}
