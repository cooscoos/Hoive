// Tests that use cube co-ordinates: cargo test cube
use hoive::game::comps::Team;
use hoive::game::{board::Board, movestatus::MoveStatus};
use hoive::maths::{coord::Coord, coord::Cube, coord::DoubleHeight};
use std::collections::BTreeSet;

mod common;
use common::basic; // basic tests that work with all co-ordinate systems
use common::games::cubes_from_list;
use common::games::{game_snapshot_1, test_board}; // Half played games of Hive

#[test]
fn cube_first_turn() {
    basic::first_turn(&mut test_board(Cube::default()));
}

#[test]
fn cube_occupied() {
    basic::occupied(&mut test_board(Cube::default()));
}

#[test]
fn cube_to_the_moon() {
    basic::to_the_moon(&mut test_board(Cube::default()));
}

// These tests are hecs specific
#[test]
fn cube_second_turn_neighbour() {
    // Place a white chip next to a black chip but on the second turn (should be okay)
    let mut board = test_board(Cube::default());
    board.move_chip("a1", Team::Black, Cube::new(0, 0, 0));
    assert_eq!(
        MoveStatus::Success,
        board.move_chip("a1", Team::White, Cube::new(1, 0, -1))
    );
}

#[test]
fn cube_third_turn_badneighbour() {
    // Place a white chip next to a black chip on the third turn (that's illegal)
    let mut board = test_board(Cube::default());
    board.move_chip("a1", Team::Black, Cube::new(0, 0, 0));
    board.move_chip("a1", Team::White, Cube::new(1, 0, -1));
    assert_eq!(
        MoveStatus::BadNeighbour,
        board.move_chip("a2", Team::White, Cube::new(-1, 0, 1))
    );
}

#[test]
fn cube_fifth_turn_badneighbour() {
    // Do a bunch of legal stuff with a BadNeighbour move at the end
    let mut board = test_board(Cube::default());
    board.move_chip("q1", Team::Black, Cube::new(0, 0, 0));
    board.move_chip("q1", Team::White, Cube::new(0, -1, 1));
    board.move_chip("a2", Team::Black, Cube::new(0, 1, -1));
    board.move_chip("a2", Team::White, Cube::new(0, -2, 2));

    assert_eq!(
        MoveStatus::BadNeighbour,
        board.move_chip("a3", Team::Black, Cube::new(1, -3, 2))
    );
}

#[test]
fn cube_split_hive() {
    // Put down four chips and then split the hive by moving a black chip from the middle
    let mut board = test_board(Cube::default());
    board.move_chip("a1", Team::Black, Cube::new(0, 0, 0));
    board.move_chip("q1", Team::White, Cube::new(0, -1, 1));
    board.move_chip("q1", Team::Black, Cube::new(0, 1, -1));
    board.move_chip("a2", Team::White, Cube::new(0, -2, 2));

    assert_eq!(
        MoveStatus::HiveSplit,
        board.move_chip("a1", Team::Black, Cube::new(0, -3, 3))
    );
}

#[test]
fn cube_attack() {
    // Put down chips and then move them about a lot
    let mut board = test_board(Cube::default());
    board.move_chip("q1", Team::Black, Cube::new(0, 0, 0));
    board.move_chip("q1", Team::White, Cube::new(0, -1, 1));
    board.move_chip("a1", Team::Black, Cube::new(0, 1, -1));
    board.move_chip("a2", Team::White, Cube::new(1, -2, 1));
    board.move_chip("a1", Team::Black, Cube::new(1, 1, -2));
    board.move_chip("a3", Team::White, Cube::new(0, -2, 2));

    assert_eq!(
        MoveStatus::Success,
        board.move_chip("a1", Team::Black, Cube::new(1, -3, 2))
    );
}

#[test]
fn cube_nosplit_hive() {
    // Put down lots of chips and then do a move that doesn't split hive and is legal
    let mut board = test_board(Cube::default());
    board.move_chip("q1", Team::Black, Cube::new(0, 0, 0));
    board.move_chip("q1", Team::White, Cube::new(0, -1, 1));
    board.move_chip("a1", Team::Black, Cube::new(0, 1, -1));
    board.move_chip("a2", Team::White, Cube::new(1, -2, 1));
    board.move_chip("a1", Team::Black, Cube::new(1, 1, -2));
    board.move_chip("a3", Team::White, Cube::new(0, -2, 2));
    board.move_chip("a1", Team::Black, Cube::new(1, -3, 2));

    assert_eq!(
        MoveStatus::Success,
        board.move_chip("a3", Team::White, Cube::new(-1, -1, 2))
    );
}

#[test]
fn cube_from_dheight_once() {
    // Convert from doubleheight to cube co-ordinates once
    let board = test_board(Cube::default());
    let dheight = DoubleHeight::from((0, -4));

    let cubehex = board.coord.mapfrom_doubleheight(dheight);

    assert_eq!(Cube::new(0, -2, 2), cubehex);
}

#[test]
fn cube_from_dheight_complicated() {
    // Convert from doubleheight to cube co-ordinates a lot

    // Do a bunch of moves in dheight co-ords to simulate the game shownn in /reference/tests/bug1.png
    let board = test_board(Cube::default());
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
    let hex_moves = moves_list
        .iter()
        .map(|xy| board.coord.mapfrom_doubleheight(DoubleHeight::from((*xy))))
        .collect::<Vec<Cube>>();

    let expected = cubes_from_list(vec![
        (0, 0, 0),
        (0, -1, 1),
        (0, -2, 2),
        (1, -2, 1),
        (2, -3, 1),
        (1, 0, -1),
        (0, 1, -1),
        (0, 2, -2),
    ]);

    // Shove both results into a BTreeSet to ensure order is the same
    let hex_ordered = hex_moves.into_iter().collect::<BTreeSet<_>>();
    let expected_ordered = expected.into_iter().collect::<BTreeSet<_>>();

    assert_eq!(expected_ordered, hex_ordered);
}

// This fn gets run by some of the next tests
fn doubleheight_to_cube<T: Coord>(board: &mut Board<T>) -> Vec<Cube>
where
    Vec<Cube>: FromIterator<T>,
{
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
        (0, 4),
    ];

    // Map all of these to hex
    let hex_moves = moves_list
        .iter()
        .map(|xy| board.coord.mapfrom_doubleheight(DoubleHeight::from((*xy))))
        .collect::<Vec<Cube>>();

    hex_moves
}

#[test]
fn cube_no_split_hive2() {
    // Put down chips in doubleheight co-ords and then do a move that doesn't split hive and is legal
    let mut board = test_board(Cube::default());
    let hex_moves = doubleheight_to_cube(&mut board);

    // Apply these moves on the board
    board.move_chip("q1", Team::White, hex_moves[0]);
    board.move_chip("q1", Team::Black, hex_moves[1]);

    board.move_chip("a2", Team::White, hex_moves[2]);
    board.move_chip("a2", Team::Black, hex_moves[3]);

    board.move_chip("a3", Team::White, hex_moves[4]);
    board.move_chip("a3", Team::Black, hex_moves[5]);

    board.move_chip("a4", Team::White, hex_moves[6]);
    board.move_chip("a4", Team::Black, hex_moves[7]);

    board.move_chip("a3", Team::White, hex_moves[8]);

    assert_eq!(
        MoveStatus::Success,
        board.move_chip("a3", Team::Black, hex_moves[9])
    );
}

#[test]
fn cube_no_split_hive3() {
    // Run the /reference/tests/bug2.png game and do a legal move
    let mut snapshot = game_snapshot_1();

    assert_eq!(
        MoveStatus::Success,
        snapshot.move_chip("a3", Team::Black, Cube::new(0, 2, -2))
    );
}

#[test]
fn cube_centroid_calc() {
    // Check the hex centroid distance calcs are working
    let coord_sys = Cube::default();
    assert_eq!(
        2.0,
        coord_sys.centroid_distance(Cube::new(0, 0, 0), Cube::new(2, -2, -0))
    );
}

#[test]
fn cube_to_doubleheight() {
    // Test conversion from cube to doubleheight
    let coord_sys = Cube::default();
    let hex = Cube::new(1, -1, 0); // up and right from the origin in cube coords

    assert_eq!(DoubleHeight::from((1, -1)), coord_sys.to_doubleheight(hex));
}

#[test]
fn cube_from_doubleheight() {
    // Test conversion from doubleheight to cube

    let coord_sys = Cube::default();
    let hex = (-1, 1); // down and left from the origin in doubleheight coords

    assert_eq!(
        Cube::new(-1, 1, 0),
        coord_sys.mapfrom_doubleheight(DoubleHeight::from(hex))
    );
}

#[test]
fn cube_ant_squeeze() {
    // Set up the board as shown in /reference/tests/ant_squeeze.jpeg, but with all ants (and one bee)
    // Try and move into the small gap, should be illegal
    let mut board = test_board(Cube::default());

    // In doubleheight
    let moves_list: Vec<(i8, i8)> = vec![
        // Placement of two white pieces and a black
        (-2, 4), // wq1 place queen bees first
        (-2, 2), // bq1
        (-1, 5), // wa1
        // Placement of remaining black pieces (all ants)
        (-2, 0),
        (-2, -2),
        (-1, -3),
        (0, -2),
        (1, -1),
        (1, 1),
        (0, 2),
        // movement of wa1 into the small gap
        (-1, 1),
    ];

    // Convert to cube
    let hex_moves = moves_list
        .iter()
        .map(|xy| board.coord.mapfrom_doubleheight(DoubleHeight::from(*xy)))
        .collect::<Vec<Cube>>();

    board.move_chip("q1", Team::White, hex_moves[0]);
    board.move_chip("q1", Team::Black, hex_moves[1]);
    board.move_chip("a1", Team::White, hex_moves[2]);
    board.move_chip("a2", Team::Black, hex_moves[3]);
    board.move_chip("a3", Team::Black, hex_moves[4]);
    board.move_chip("a4", Team::Black, hex_moves[5]);
    board.move_chip("a5", Team::Black, hex_moves[6]);
    board.move_chip("a6", Team::Black, hex_moves[7]);
    board.move_chip("a7", Team::Black, hex_moves[8]);
    board.move_chip("a8", Team::Black, hex_moves[9]);

    // Now try move a1 into a small gap
    assert_eq!(
        MoveStatus::SmallGap,
        board.move_chip("a1", Team::White, hex_moves[10])
    );
}

#[test]
fn cube_hex_distance() {
    // Make sure the hex distance calc is working
    let board = test_board(Cube::default());

    let pos1 = Cube::new(0, 0, 0);
    let pos2 = Cube::new(-3, 2, 1);

    assert_eq!(3, board.coord.hex_distance(pos1, pos2));
}
