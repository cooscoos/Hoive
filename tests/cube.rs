// Tests that use cube co-ordinates: cargo test cube
use hoive::game::board::*;
use hoive::game::comps::Team;
use hoive::maths::{coord::Coord, coord::Cube, morphops};
use hoive::pmoore;

// basic tests that work with all co-ordinate systems
mod basic;

#[test]
fn cube_first_turn() {
    basic::first_turn(&mut Board::default(Cube));
}

#[test]
fn cube_occupied() {
    basic::occupied(&mut Board::default(Cube));
}

#[test]
fn cube_to_the_moon() {
    basic::to_the_moon(&mut Board::default(Cube));
}

// These tests are hecs specific
#[test]
fn cube_second_turn_neighbour() {
    // Place a white chip next to a black chip but on the second turn (should be okay)
    let mut board = Board::default(Cube);
    pmoore::try_move(&mut board, "s1", Team::Black, (0, 0, 0));
    assert_eq!(
        MoveStatus::Success,
        pmoore::try_move(&mut board, "s1", Team::White, (1, 0, -1))
    );
}

#[test]
fn cube_third_turn_badneighbour() {
    // Place a white chip next to a black chip on the third turn (that's illegal)
    let mut board = Board::default(Cube);
    pmoore::try_move(&mut board, "s1", Team::Black, (0, 0, 0));
    pmoore::try_move(&mut board, "s1", Team::White, (1, 0, -1));
    assert_eq!(
        MoveStatus::BadNeighbour,
        pmoore::try_move(&mut board, "s2", Team::White, (-1, 0, 1))
    );
}

#[test]
fn cube_fifth_turn_badneighbour() {
    // Do a bunch of legal stuff with a BadNeighbour move at the end
    let mut board = Board::default(Cube);
    pmoore::try_move(&mut board, "s1", Team::Black, (0, 0, 0));
    pmoore::try_move(&mut board, "s1", Team::White, (0, -1, 1));
    pmoore::try_move(&mut board, "s2", Team::Black, (0, 1, -1));
    pmoore::try_move(&mut board, "s2", Team::White, (0, -2, 2));

    assert_eq!(
        MoveStatus::BadNeighbour,
        pmoore::try_move(&mut board, "s3", Team::Black, (1, -3, 2))
    );
}

#[test]
fn cube_split_hive() {
    // Put down four chips and then split the hive by moving a white spider from the middle
    let mut board = Board::default(Cube);
    pmoore::try_move(&mut board, "s1", Team::Black, (0, 0, 0));
    pmoore::try_move(&mut board, "s1", Team::White, (0, -1, 1));
    pmoore::try_move(&mut board, "s2", Team::Black, (0, 1, -1));
    pmoore::try_move(&mut board, "s2", Team::White, (0, 2, -2));

    assert_eq!(
        MoveStatus::HiveSplit,
        pmoore::try_move(&mut board, "s1", Team::Black, (0, 2, -2))
    );
}




#[test]
fn cube_attack() {
    // Put down lots of chips and then relocate a black next to black after turn 6
    // We haven't coded logic for bee allowing move yet, so we'll need to rewrite this test then
    let mut board = Board::default(Cube);
    pmoore::try_move(&mut board, "s1", Team::Black, (0, 0, 0));
    pmoore::try_move(&mut board, "s1", Team::White, (0, -1, 1));
    pmoore::try_move(&mut board, "s1", Team::Black, (0, 1, -1));
    pmoore::try_move(&mut board, "s2", Team::White, (1, -2, 1));
    pmoore::try_move(&mut board, "s1", Team::Black, (1, 1, -2));
    pmoore::try_move(&mut board, "s3", Team::White, (0, -2, 2));

    assert_eq!(
        MoveStatus::Success,
        pmoore::try_move(&mut board, "s1", Team::Black, (1, -3, 2))
    );
}

#[test]
fn cube_nosplit_hive() {
    // Put down lots of chips and then do a move that doesn't split hive and is legal
    let mut board = Board::default(Cube);
    pmoore::try_move(&mut board, "s1", Team::Black, (0, 0, 0));
    pmoore::try_move(&mut board, "s1", Team::White, (0, -1, 1));
    pmoore::try_move(&mut board, "s1", Team::Black, (0, 1, -1));
    pmoore::try_move(&mut board, "s2", Team::White, (1, -2, 1));
    pmoore::try_move(&mut board, "s1", Team::Black, (1, 1, -2));
    pmoore::try_move(&mut board, "s3", Team::White, (0, -2, 2));
    pmoore::try_move(&mut board, "s1", Team::Black, (1, -3, 2));

    assert_eq!(
        MoveStatus::Success,
        pmoore::try_move(&mut board, "s3", Team::White, (-1, -1, 2))
    );
}


#[test]
fn cube_no_split_hive2() {
    // Put down chips in doubleheight co-ords and then do a move that doesn't split hive and is legal
    // This emulates a bug (hehe) that was found when doing the following:
    // ws1 to 0,0
    // bs1 to 0,-2
    // ws2 to 1,1
    // bs2 to 1,-3
    // ws3 to -1,1
    // bs3 to -1,-3
    // ws4 to 0,2
    // bs4 to 0,-4
    // ws3 to 2,-4
    // bs3 to 0,4
    // check ws2 to 3,-5
    
    let mut board = Board::default(Cube);

    let moves_list: [(i8,i8);11] = [
        (0,0),
        (0,-2),
        (1,1),
        (1,-3),
        (-1,1),
        (-1,-3),
        (0,2),
        (0,-4),
        (2,-4),
        (0,4),
        (3,-5),
    ];



    // map all of these to hex
    let hex_moves = moves_list.iter().map(|xy| board.coord.mapfrom_doubleheight(*xy)).collect::<Vec<(i8,i8,i8)>>();

    // do the moves on the board

    pmoore::try_move(&mut board, "s1", Team::White, hex_moves[0]);
    pmoore::try_move(&mut board, "s1", Team::Black, hex_moves[1]);

    pmoore::try_move(&mut board, "s2", Team::White, hex_moves[2]);
    pmoore::try_move(&mut board, "s2", Team::Black, hex_moves[3]);
    
    pmoore::try_move(&mut board, "s3", Team::White, hex_moves[4]);
    pmoore::try_move(&mut board, "s3", Team::Black, hex_moves[5]);

    pmoore::try_move(&mut board, "s4", Team::White, hex_moves[6]);
    pmoore::try_move(&mut board, "s4", Team::Black, hex_moves[7]);

    pmoore::try_move(&mut board, "s3", Team::White, hex_moves[8]);
    pmoore::try_move(&mut board, "s3", Team::Black, hex_moves[9]);


    assert_eq!(
        MoveStatus::Success,
        pmoore::try_move(&mut board, "s2", Team::White, hex_moves[10])
    );
}




#[test]
fn centroid_calc() {
    let coord_sys = Cube;
    assert_eq!(2.0, coord_sys.centroid_distance((0, 0, 0), (2, -2, -0)));
}

#[test]
fn test_to_doubleheight() {
    // Test conversion from cube to doubleheight
    let coord_sys = Cube;
    let hex = (1, -1, 0); // up and right from the origin in cube coords

    assert_eq!((1, -1), coord_sys.mapto_doubleheight(hex));
}

#[test]
fn test_from_doubleheight() {
    // Test conversion from doubleheight to cube

    let coord_sys = Cube;
    let hex = (-1, 1); // down and left from the origin in doubleheight coords

    assert_eq!((-1, 1, 0), coord_sys.mapfrom_doubleheight(hex));
}
