// Snapshots of boards used for other tests
use hoive::game::board::*;
use hoive::game::comps::Team;
use hoive::maths::{coord::Coord, coord::Cube};
use hoive::pmoore;

pub fn game_snapshot_1() -> Board<Cube> {
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
    pmoore::try_move(&mut board, "q1", Team::White, hex_moves[0]);
    pmoore::try_move(&mut board, "q1", Team::Black, hex_moves[1]);

    pmoore::try_move(&mut board, "a2", Team::White, hex_moves[2]);
    pmoore::try_move(&mut board, "a2", Team::Black, hex_moves[3]);

    pmoore::try_move(&mut board, "a3", Team::White, hex_moves[4]);
    pmoore::try_move(&mut board, "a3", Team::Black, hex_moves[5]);

    pmoore::try_move(&mut board, "a4", Team::White, hex_moves[6]);
    pmoore::try_move(&mut board, "a4", Team::Black, hex_moves[7]);

    pmoore::try_move(&mut board, "a3", Team::White, hex_moves[8]);

    board
}

pub fn game_snapshot_2() -> Board<Cube> {
    // Set up a gameboard for some spider and bee tests

    let mut board = Board::default(Cube);

    let moves_list = vec![
        (0, 0),  // wq1
        (0, -2), // bq1
        (0, -4),
    ];

    // Convert to cube
    let hex_moves = moves_list
        .iter()
        .map(|xy| board.coord.mapfrom_doubleheight(*xy))
        .collect::<Vec<(i8, i8, i8)>>();

    pmoore::try_move(&mut board, "q1", Team::White, hex_moves[0]);
    pmoore::try_move(&mut board, "q1", Team::Black, hex_moves[1]);
    pmoore::try_move(&mut board, "a2", Team::Black, hex_moves[2]);

    board
}
