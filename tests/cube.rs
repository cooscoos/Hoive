// Tests that use cube co-ordinates: cargo test cube
use hoive::game::board::*;
use hoive::game::comps::Team;
use hoive::maths::{coord::Coord, coord::Cube};
use hoive::pmoore;

// basic tests that work with all co-ordinate systems
mod basic;

#[test]
fn cube_first_turn() {
    basic::first_turn(&mut Board::test_board(Cube));
}

#[test]
fn cube_occupied() {
    basic::occupied(&mut Board::test_board(Cube));
}

#[test]
fn cube_to_the_moon() {
    basic::to_the_moon(&mut Board::test_board(Cube));
}

// These tests are hecs specific
#[test]
fn cube_second_turn_neighbour() {
    // Place a white chip next to a black chip but on the second turn (should be okay)
    let mut board = Board::test_board(Cube);
    pmoore::try_move(&mut board, "a1", Team::Black, (0, 0, 0));
    assert_eq!(
        MoveStatus::Success,
        pmoore::try_move(&mut board, "a1", Team::White, (1, 0, -1))
    );
}

#[test]
fn cube_third_turn_badneighbour() {
    // Place a white chip next to a black chip on the third turn (that's illegal)
    let mut board = Board::test_board(Cube);
    pmoore::try_move(&mut board, "a1", Team::Black, (0, 0, 0));
    pmoore::try_move(&mut board, "a1", Team::White, (1, 0, -1));
    assert_eq!(
        MoveStatus::BadNeighbour,
        pmoore::try_move(&mut board, "a2", Team::White, (-1, 0, 1))
    );
}

#[test]
fn cube_fifth_turn_badneighbour() {
    // Do a bunch of legal stuff with a BadNeighbour move at the end
    let mut board = Board::test_board(Cube);
    pmoore::try_move(&mut board, "a1", Team::Black, (0, 0, 0));
    pmoore::try_move(&mut board, "a1", Team::White, (0, -1, 1));
    pmoore::try_move(&mut board, "a2", Team::Black, (0, 1, -1));
    pmoore::try_move(&mut board, "a2", Team::White, (0, -2, 2));

    assert_eq!(
        MoveStatus::BadNeighbour,
        pmoore::try_move(&mut board, "a3", Team::Black, (1, -3, 2))
    );
}

#[test]
fn cube_split_hive() {
    // Put down four chips and then split the hive by moving a black spider from the middle
    let mut board = Board::test_board(Cube);
    pmoore::try_move(&mut board, "a1", Team::Black, (0, 0, 0));
    pmoore::try_move(&mut board, "a1", Team::White, (0, -1, 1));
    pmoore::try_move(&mut board, "a2", Team::Black, (0, 1, -1));
    pmoore::try_move(&mut board, "a2", Team::White, (0, -2, 2));

    assert_eq!(
        MoveStatus::HiveSplit,
        pmoore::try_move(&mut board, "a1", Team::Black, (0, -3, 3))
    );
}

#[test]
fn cube_attack() {
    // Put down lots of chips and then relocate a black next to black after turn 6
    // We haven't coded logic for first bee placement allowing moves yet, so we'll need to rewrite this test then
    let mut board = Board::test_board(Cube);
    pmoore::try_move(&mut board, "a1", Team::Black, (0, 0, 0));
    pmoore::try_move(&mut board, "a1", Team::White, (0, -1, 1));
    pmoore::try_move(&mut board, "a1", Team::Black, (0, 1, -1));
    pmoore::try_move(&mut board, "a2", Team::White, (1, -2, 1));
    pmoore::try_move(&mut board, "a1", Team::Black, (1, 1, -2));
    pmoore::try_move(&mut board, "a3", Team::White, (0, -2, 2));

    assert_eq!(
        MoveStatus::Success,
        pmoore::try_move(&mut board, "a1", Team::Black, (1, -3, 2))
    );
}

#[test]
fn cube_nosplit_hive() {
    // Put down lots of chips and then do a move that doesn't split hive and is legal
    let mut board = Board::test_board(Cube);
    pmoore::try_move(&mut board, "a1", Team::Black, (0, 0, 0));
    pmoore::try_move(&mut board, "a1", Team::White, (0, -1, 1));
    pmoore::try_move(&mut board, "a1", Team::Black, (0, 1, -1));
    pmoore::try_move(&mut board, "a2", Team::White, (1, -2, 1));
    pmoore::try_move(&mut board, "a1", Team::Black, (1, 1, -2));
    pmoore::try_move(&mut board, "a3", Team::White, (0, -2, 2));
    pmoore::try_move(&mut board, "a1", Team::Black, (1, -3, 2));

    assert_eq!(
        MoveStatus::Success,
        pmoore::try_move(&mut board, "a3", Team::White, (-1, -1, 2))
    );
}

#[test]
fn cube_from_dheight_once() {
    // Convert from doubleheight to cube co-ordinates once
    let board = Board::test_board(Cube);
    let dheight = (0, -4);

    let cubehex = board.coord.mapfrom_doubleheight(dheight);

    assert_eq!((0, -2, 2), cubehex);
}

#[test]
fn cube_from_dheight_complicated() {
    // Convert from doubleheight to cube co-ordinates a lot

    // Do a bunch of moves in dheight co-ords to simulate the game shownn in /reference/tests/bug1.png
    let board = Board::test_board(Cube);
    let moves_list: [(i8, i8); 8] = [
        (0, 0),
        (0, -2),
        (1, 1),
        (1, -3),
        (0, 2),
        (0, -4),
        (2, -4),
        (0, 4),
    ];

    // Map all of these moves to cube co-ords
    let mut hex_moves = moves_list
        .iter()
        .map(|xy| board.coord.mapfrom_doubleheight(*xy))
        .collect::<Vec<(i8, i8, i8)>>();

    // We'd expect the output to be this
    let mut expected = vec![
        (0, 0, 0),
        (0, -1, 1),
        (0, -2, 2),
        (1, -2, 1),
        (2, -3, 1),
        (1, 0, -1),
        (0, 1, -1),
        (0, 2, -2),
    ];

    // Raster scan both the result and the expected vectors to make sure they're in the same order
    board.coord.raster_scan(&mut hex_moves);
    board.coord.raster_scan(&mut expected);

    assert_eq!(expected, hex_moves);
}

// This fn gets run by some of the next tests
fn doubleheight_to_cube<T: Coord>(board: &mut Board<T>) -> Vec<(i8, i8, i8)> {
    // Here's a big list of moves that simulates the game shown in /reference/tests/bug2.png using doubleheight co-ords
    let moves_list: [(i8, i8); 10] = [
        (0, 0),
        (0, -2),
        (1, 1),
        (1, -3),
        (-1, 1),
        (-1, -3),
        (0, 2),
        (0, -4),
        (2, -4),
        (0,4),
    ];

    // Map all of these to hex
    let hex_moves = moves_list
        .iter()
        .map(|xy| board.coord.mapfrom_doubleheight(*xy))
        .collect::<Vec<(i8, i8, i8)>>();

    hex_moves
}

#[test]
fn cube_no_split_hive2() {
    // Put down chips in doubleheight co-ords and then do a move that doesn't split hive and is legal
    let mut board = Board::test_board(Cube);
    let hex_moves = doubleheight_to_cube(&mut board);

    // Apply these moves on the board
    pmoore::try_move(&mut board, "a1", Team::White, hex_moves[0]);
    pmoore::try_move(&mut board, "a1", Team::Black, hex_moves[1]);

    pmoore::try_move(&mut board, "a2", Team::White, hex_moves[2]);
    pmoore::try_move(&mut board, "a2", Team::Black, hex_moves[3]);

    pmoore::try_move(&mut board, "a3", Team::White, hex_moves[4]);
    pmoore::try_move(&mut board, "a3", Team::Black, hex_moves[5]);

    pmoore::try_move(&mut board, "a4", Team::White, hex_moves[6]);
    pmoore::try_move(&mut board, "a4", Team::Black, hex_moves[7]);

    pmoore::try_move(&mut board, "a3", Team::White, hex_moves[8]);

    assert_eq!(
        MoveStatus::Success,
        pmoore::try_move(&mut board, "a3", Team::Black, hex_moves[9])
    );
}

fn bug2_game_cube() -> Board<Cube> {
    // This function is called by a few subsequent tests
    // Run the game shown in /referenece/tests/bug2.png using cube co-ordinates
    let mut board = Board::test_board(Cube);
    let hex_moves: [(i8, i8, i8); 9] = [
        (0, 0, 0),
        (0, -1, 1),
        (1, 0, -1),
        (1, -2, 1),
        (-1, 1, 0),
        (-1, -1, 2),
        (0, 1, -1),
        (0, -2, 2),
        (2, -3, 1),
    ];

    // Do these moves on the board
    pmoore::try_move(&mut board, "a1", Team::White, hex_moves[0]);
    pmoore::try_move(&mut board, "a1", Team::Black, hex_moves[1]);

    pmoore::try_move(&mut board, "a2", Team::White, hex_moves[2]);
    pmoore::try_move(&mut board, "a2", Team::Black, hex_moves[3]);

    pmoore::try_move(&mut board, "a3", Team::White, hex_moves[4]);
    pmoore::try_move(&mut board, "a3", Team::Black, hex_moves[5]);

    pmoore::try_move(&mut board, "a4", Team::White, hex_moves[6]);
    pmoore::try_move(&mut board, "a4", Team::Black, hex_moves[7]);

    pmoore::try_move(&mut board, "a3", Team::White, hex_moves[8]);

    board
}

#[test]
fn cube_check_rasterscan_order() {
    // Make sure raster scan is behaving properly
    // Do a /referenece/tests/bug2.png game
    let snapshot = bug2_game_cube();

    // Get a raster scan of this game
    let code_raster = snapshot.rasterscan_board();

    // Expected raster scan output is (top to bottom, left to right)
    let expected = vec![
        (2, -3, 1),
        (0, -2, 2),
        (1, -2, 1),
        (-1, -1, 2),
        (0, -1, 1),
        (0, 0, 0),
        (1, 0, -1),
        (0, 1, -1),
    ];

    assert_eq!(expected, code_raster);
}

#[test]
fn cube_no_split_hive3() {
    // Run the /reference/tests/bug2.png game and do a legal move
    let mut snapshot = bug2_game_cube();

    assert_eq!(
        MoveStatus::Success,
        pmoore::try_move(&mut snapshot, "a3", Team::Black, (0, 2, -2))
    );
}

#[test]
fn cube_centroid_calc() {
    // Check the hex centroid distance calcs are working
    let coord_sys = Cube;
    assert_eq!(2.0, coord_sys.centroid_distance((0, 0, 0), (2, -2, -0)));
}

#[test]
fn cube_to_doubleheight() {
    // Test conversion from cube to doubleheight
    let coord_sys = Cube;
    let hex = (1, -1, 0); // up and right from the origin in cube coords

    assert_eq!((1, -1), coord_sys.mapto_doubleheight(hex));
}

#[test]
fn cube_from_doubleheight() {
    // Test conversion from doubleheight to cube

    let coord_sys = Cube;
    let hex = (-1, 1); // down and left from the origin in doubleheight coords

    assert_eq!((-1, 1, 0), coord_sys.mapfrom_doubleheight(hex));
}

#[test]
fn cube_ant_squeeze() {
    // Set up the board as shown in /reference/tests/ant_squeeze.jpeg, but with all ants
    // Try and move into the small gap, should be illegal
    let mut board = Board::test_board(Cube);

    // In doubleheight
    let moves_list: Vec<(i8,i8)> = vec![
        // Placement of two white pieces and a black
        (-2,4), // wa2
        (-2,2), // ba1
        (-1,5), // wa1
        // Placement of remaining black pieces
        (-2,0),
        (-2,-2),
        (-1,-3),
        (0,-2),
        (1,-1),
        (1,1),
        (0,2),
        // movement of wa1 into the small gap
        (-1,1),
    ];

    // Convert to cube
    let hex_moves = moves_list
    .iter()
    .map(|xy| board.coord.mapfrom_doubleheight(*xy))
    .collect::<Vec<(i8, i8, i8)>>();


    pmoore::try_move(&mut board, "a2", Team::White, hex_moves[0]);
    pmoore::try_move(&mut board, "a1", Team::Black, hex_moves[1]);
    pmoore::try_move(&mut board, "a1", Team::White, hex_moves[2]);
    pmoore::try_move(&mut board, "a2", Team::Black, hex_moves[3]);
    pmoore::try_move(&mut board, "a3", Team::Black, hex_moves[4]);
    pmoore::try_move(&mut board, "a4", Team::Black, hex_moves[5]);
    pmoore::try_move(&mut board, "a5", Team::Black, hex_moves[6]);
    pmoore::try_move(&mut board, "a6", Team::Black, hex_moves[7]);
    pmoore::try_move(&mut board, "a7", Team::Black, hex_moves[8]);
    pmoore::try_move(&mut board, "a8", Team::Black, hex_moves[9]);

    // Now try move a1 into a small gap
    assert_eq!(
        MoveStatus::SmallGap,
        pmoore::try_move(&mut board, "a1", Team::White, hex_moves[10])
    );

}