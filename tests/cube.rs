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
fn what_cube_split_hive() {
    // Put down four chips and then split the hive by moving a white spider from the middle
    let mut board = Board::default(Cube);
    pmoore::try_move(&mut board, "s1", Team::Black, (0, 0, 0));
    pmoore::try_move(&mut board, "s1", Team::White, (0, -1, 1));
    pmoore::try_move(&mut board, "s2", Team::Black, (0, 1, -1));
    pmoore::try_move(&mut board, "s2", Team::White, (0, -2, 2));

    assert_eq!(
        MoveStatus::HiveSplit,
        pmoore::try_move(&mut board, "s1", Team::Black, (0, -3, 3))
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
fn test_simple_dheight_to_cube() {
    let board = Board::default(Cube);
    let dheight = (0,-4);

    let cubehex = board.coord.mapfrom_doubleheight(dheight);

    assert_eq!((0,-2,2), cubehex);

}


#[test]
fn test_dheight_to_cube_conversion() {
    let board = Board::default(Cube);

    // generate a bunch of co-ords in dheight to simulate bug1.png
    let moves_list: [(i8,i8);8] = [
        (0,0),
        (0,-2),
        (1,1),
        (1,-3),
        (0,2),
        (0,-4),
        (2,-4),
        (0,4),
    ];

    // map all of these to hex
    let mut hex_moves = moves_list.iter().map(|xy| board.coord.mapfrom_doubleheight(*xy)).collect::<Vec<(i8,i8,i8)>>();

    // raster scan them
    board.coord.raster_scan(&mut hex_moves);

    // we'd expect the output to be
    let mut expected = vec![
        (0,0,0),
        (0,-1,1),
        (0,-2,2),
        (1,-2,1),
        (2,-3,1),
        (1,0,-1),
        (0,1,-1),
        (0,2,-2),
    ];


    board.coord.raster_scan(&mut expected);


    assert_eq!(expected, hex_moves);

}

// This fn gets run by some of the next tests
fn doubleheight_to_cube<T: Coord>(board: &mut Board<T>) -> Vec<(i8,i8,i8)> {

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

    hex_moves
}


#[test]
fn test_cube_no_split_hive2() {
    // Put down chips in doubleheight co-ords and then do a move that doesn't split hive and is legal
    // This emulates a bug that was found when doing the following:
    
    let mut board = Board::default(Cube);

    // Generate a bunch of moves
    let hex_moves = doubleheight_to_cube(&mut board);
    
    // do these moves on the board
    pmoore::try_move(&mut board, "s1", Team::White, hex_moves[0]);
    pmoore::try_move(&mut board, "s1", Team::Black, hex_moves[1]);

    pmoore::try_move(&mut board, "s2", Team::White, hex_moves[2]);
    pmoore::try_move(&mut board, "s2", Team::Black, hex_moves[3]);
    
    pmoore::try_move(&mut board, "s3", Team::White, hex_moves[4]);
    pmoore::try_move(&mut board, "s3", Team::Black, hex_moves[5]);

    pmoore::try_move(&mut board, "s4", Team::White, hex_moves[6]);
    pmoore::try_move(&mut board, "s4", Team::Black, hex_moves[7]);

    pmoore::try_move(&mut board, "s3", Team::White, hex_moves[8]);
    
    assert_eq!(
        MoveStatus::Success,
        pmoore::try_move(&mut board, "s3", Team::Black, hex_moves[9])
    );


//    assert_eq!(
//        MoveStatus::Success,
//        pmoore::try_move(&mut board, "s2", Team::White, hex_moves[10])
//    );
}


fn big_game_1() -> Board<Cube>{
    let mut board = Board::default(Cube);
    let hex_moves: [(i8,i8,i8);9] = [
        (0,0,0),
        (0,-1,1),
        (1,0,-1),
        (1,-2,1),
        (-1,1,0),
        (-1,-1,2),
        (0,1,-1),
        (0,-2,2),
        (2,-3,1),
        //(0,2,-2),
    ];

    // do these moves on the board
    pmoore::try_move(&mut board, "s1", Team::White, hex_moves[0]);
    pmoore::try_move(&mut board, "s1", Team::Black, hex_moves[1]);

    pmoore::try_move(&mut board, "s2", Team::White, hex_moves[2]);
    pmoore::try_move(&mut board, "s2", Team::Black, hex_moves[3]);
    
    pmoore::try_move(&mut board, "s3", Team::White, hex_moves[4]);
    pmoore::try_move(&mut board, "s3", Team::Black, hex_moves[5]);

    pmoore::try_move(&mut board, "s4", Team::White, hex_moves[6]);
    pmoore::try_move(&mut board, "s4", Team::Black, hex_moves[7]);

    pmoore::try_move(&mut board, "s3", Team::White, hex_moves[8]);
    //let checker = pmoore::try_move(&mut board, "s3", Team::Black, hex_moves[9]);

    //println!("Second bs3 move was:{:?}", checker);

    board
}


#[test]
fn test_check_rasterscan_order(){
    // Make sure raster scan is behaving
    // Do a bug2.png game
    let snapshot = big_game_1();

    // Ask the code to raster scan this
    let code_raster = snapshot.rasterscan_board();


    // expected raster output should be (top to bottom, left to right)
    let expected = vec![
        (2,-3,1),
        (0,-2,2),
        (1,-2,1),
        (-1,-1,2), 
        (0,-1,1),
        (0,0,0),
        (1,0,-1),
        (0,1,-1),
    ];


    println!("Code's rasterscan:\n{:?}\n\nManual rasterscan:\n{:?}",code_raster,expected);
    assert_eq!(expected,code_raster);


}



#[test]
fn pure_test_cube_no_split_hive3(){
    // Run the same game again in pure cube co-ords to see if the bug persists 

    // Get board into bug1.png snapshot
    let mut snapshot = big_game_1();


    assert_eq!(
        MoveStatus::Success,
        pmoore::try_move(&mut snapshot, "s3", Team::Black, (0,2,-2))
    );

    //assert_eq!(
    //    MoveStatus::Success,
    //    pmoore::try_move(&mut snapshot, "s2", Team::White, (2,-3,1))
    //);
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
