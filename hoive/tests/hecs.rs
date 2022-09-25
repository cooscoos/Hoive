// Tests that use HECS co-ordinates: cargo test hecs

// Redundant now. Saved it for austerity.

// use hoive::maths::{coord::Coord, coord::Hecs, morphops};

// mod basic; // basic tests that work with all co-ordinate systems

// use hoive::game::board::*;
// use hoive::game::comps::Team;
// use hoive::pmoore;

// #[test]
// fn hecs_first_turn() {
//     basic::first_turn(&mut Board::test_board(Hecs));
// }

// #[test]
// fn hecs_occupied() {
//     basic::occupied(&mut Board::test_board(Hecs));
// }

// #[test]
// fn hecs_to_the_moon() {
//     basic::to_the_moon(&mut Board::test_board(Hecs));
// }

// // These tests are hecs specific
// #[test]
// fn hecs_second_turn_neighbour() {
//     // Place a white chip next to a black chip but on the second turn (should be okay)
//     let mut board = Board::test_board(Hecs);
//     board.move_chip("a1", Team::Black, (1, 0, 0));
//     assert_eq!(
//         MoveStatus::Success,
//         board.move_chip("a1", Team::White, (0, 1, 0))
//     );
// }

// #[test]
// fn hecs_third_turn_badneighbour() {
//     // Place a white chip next to a black chip on the third turn (that's illegal)
//     let mut board = Board::test_board(Hecs);
//     board.move_chip("a1", Team::Black, (1, 0, 0));
//     board.move_chip("a1", Team::White, (0, 1, 0));
//     assert_eq!(
//         MoveStatus::BadNeighbour,
//         board.move_chip("a2", Team::White, (1, 0, 1))
//     );
// }

// #[test]
// fn hecs_fifth_turn_badneighbour() {
//     // Do a bunch of legal stuff with a BadNeighbour move at the end
//     let mut board = Board::test_board(Hecs);
//     board.move_chip("q1", Team::Black, (1, 0, 0));
//     board.move_chip("q1", Team::White, (0, 1, 0));
//     board.move_chip("a2", Team::Black, (0, 0, 0));
//     board.move_chip("a2", Team::White, (1, 1, 0));

//     assert_eq!(
//         MoveStatus::BadNeighbour,
//         board.move_chip("a3", Team::Black, (1, 1, 1))
//     );
// }

// #[test]
// fn hecs_split_hive() {
//     // Put down four chips and then split the hive by moving a white spider from the middle
//     let mut board = Board::test_board(Hecs);
//     board.move_chip("q1", Team::Black, (1, 0, 0));
//     board.move_chip("a1", Team::White, (0, 1, 0));
//     board.move_chip("a2", Team::Black, (0, 0, 0));
//     board.move_chip("q1", Team::White, (1, 1, 0));

//     assert_eq!(
//         MoveStatus::HiveSplit,
//         board.move_chip("a1", Team::White, (1, 1, 1))
//     );
// }

// #[test]
// fn hecs_attack() {
//     // Put down lots of chips and then relocate a white next to black after turn 6
//     // We haven't coded logic for bee allowing move yet, so we'll need to rewrite this test then
//     let mut board = Board::test_board(Hecs);
//     board.move_chip("q1", Team::Black, (1, 0, 0));
//     board.move_chip("q1", Team::White, (0, 1, 0));
//     board.move_chip("a2", Team::Black, (0, 0, 0));
//     board.move_chip("a2", Team::White, (1, 1, 0));
//     board.move_chip("a3", Team::White, (1, 1, -1));
//     board.move_chip("a4", Team::White, (0, 2, 0));

//     assert_eq!(
//         MoveStatus::Success,
//         board.move_chip("a3", Team::White, (0, 0, 1))
//     );
// }

// #[test]
// fn hecs_nosplit_hive() {
//     // Put down lots of chips and then do a move that doesn't split hive and is legal
//     let mut board = Board::test_board(Hecs);
//     board.move_chip("q1", Team::Black, (1, 0, 0));
//     board.move_chip("q1", Team::White, (0, 1, 0));
//     board.move_chip("a2", Team::Black, (0, 0, 0));
//     board.move_chip("a2", Team::White, (1, 1, 0));
//     board.move_chip("a3", Team::White, (1, 1, -1));
//     board.move_chip("a4", Team::White, (0, 2, 0));

//     assert_eq!(
//         MoveStatus::Success,
//         board.move_chip("a3", Team::White, (1, 2, 0))
//     );
// }
