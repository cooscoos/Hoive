// Snapshots of boards used for other tests

use std::collections::{HashMap, HashSet};

use hoive::game::{board::Board, comps::Chip, comps::Team, history::History};
use hoive::maths::coord::DoubleHeight;
use hoive::maths::{coord::Coord, coord::Cube};

pub fn test_board<T: Coord>(coord: T) -> Board<T> {
    // During testing we often want lots of pieces that move freely, so give each team 8 ants and one bee
    let chips = test_chips();
    let history = History::default();

    Board {
        chips,
        turns: 0,
        coord,
        history,
        size: 5,
    }
}

fn test_chips<T: Coord>() -> HashMap<Chip, Option<T>> {
    // During some tests we want lots of chips that move freely. Give each team 8 ants, 1 bee

    let names_list = vec!["a1", "a2", "a3", "a4", "a5", "a6", "a7", "a8", "q1"];

    Chip::new_from_list(names_list)
}

/// Create a vec of Cubes based on input vec of tuples
pub fn cubes_from_list(coord_list: Vec<(i8, i8, i8)>) -> Vec<Cube> {
    let mut coord_vec = Vec::new();

    coord_list.into_iter().for_each(|(q, r, s)| {
        coord_vec.push(Cube::new(q, r, s));
    });

    coord_vec
}

/// Create a HashSet of Cubes based on input vec of tuples
pub fn cubehash_from_list(coord_list: Vec<(i8, i8, i8)>) -> HashSet<Cube> {
    let mut coord_vec = HashSet::new();

    coord_list.into_iter().for_each(|(q, r, s)| {
        coord_vec.insert(Cube::new(q, r, s));
    });

    coord_vec
}

pub fn game_snapshot_1() -> Board<Cube> {
    // This function is called by a few subsequent tests
    // Run the game shown in /referenece/tests/bug2.png using cube co-ordinates
    let mut board = test_board(Cube::default());
    let hex_moves: [Cube; 9] = [
        Cube::new(0, 0, 0),
        Cube::new(0, -1, 1),
        Cube::new(1, 0, -1),
        Cube::new(1, -2, 1),
        Cube::new(-1, 1, 0),
        Cube::new(-1, -1, 2),
        Cube::new(0, 1, -1),
        Cube::new(0, -2, 2),
        Cube::new(2, -3, 1),
    ];

    // Do these moves on the board
    board.move_chip("q1", Team::White, hex_moves[0]);
    board.move_chip("q1", Team::Black, hex_moves[1]);

    board.move_chip("a2", Team::White, hex_moves[2]);
    board.move_chip("a2", Team::Black, hex_moves[3]);

    board.move_chip("a3", Team::White, hex_moves[4]);
    board.move_chip("a3", Team::Black, hex_moves[5]);

    board.move_chip("a4", Team::White, hex_moves[6]);
    board.move_chip("a4", Team::Black, hex_moves[7]);

    board.move_chip("a3", Team::White, hex_moves[8]);

    board
}

pub fn game_snapshot_2() -> Board<Cube> {
    // Set up a gameboard for some spider and bee tests

    let mut board = Board::new(Cube::default());

    let moves_list = vec![
        (0, 0),  // wq1
        (0, -2), // bq1
        (0, -4),
    ];

    // Convert to cube
    let hex_moves = moves_list
        .iter()
        .map(|xy| board.coord.mapfrom_doubleheight(DoubleHeight::from(*xy)))
        .collect::<Vec<Cube>>();

    board.move_chip("q1", Team::White, hex_moves[0]);
    board.move_chip("q1", Team::Black, hex_moves[1]);
    board.move_chip("a2", Team::Black, hex_moves[2]);

    board
}

pub fn game_snapshot_3() -> Board<Cube> {
    // Spider and ladybird test - barrier

    let mut board = Board::new(Cube::default());

    let moves_list = vec![
        (0, 0),   // wq1
        (1, -1),  // bq1
        (2, -2),  // ba1
        (-1, -1), // wa1
        (-2, -2), // wa2
        (0, 2),   // ws1
    ];

    // Convert to cube
    let hex_moves = moves_list
        .iter()
        .map(|xy| board.coord.mapfrom_doubleheight(DoubleHeight::from(*xy)))
        .collect::<Vec<Cube>>();

    board.move_chip("q1", Team::White, hex_moves[0]);
    board.move_chip("q1", Team::Black, hex_moves[1]);
    board.move_chip("a1", Team::Black, hex_moves[2]);
    board.move_chip("a1", Team::White, hex_moves[3]);
    board.move_chip("a2", Team::White, hex_moves[4]);
    board.move_chip("s1", Team::White, hex_moves[5]);

    board
}

pub fn game_snapshot_4() -> Board<Cube> {
    // Win game tests - the white bee is in trouble

    let mut board = Board::new(Cube::default());

    let moves_list = vec![
        (0, 0),   // wq1
        (0, 2),   // bq1
        (1, -1),  // wa1
        (-1, 3),  // ba1
        (-1, -1), // wa2
        (-1, 1),  // ba1
        (0, -2),  // ws1
        (-1, 3),  // ba2
        (0, -4),  // wa3
                  // can now move ba2 or wa3 to (1,1) to defeat white team
    ];

    // Convert to cube
    let hex_moves = moves_list
        .iter()
        .map(|xy| board.coord.mapfrom_doubleheight(DoubleHeight::from(*xy)))
        .collect::<Vec<Cube>>();

    board.move_chip("q1", Team::White, hex_moves[0]);
    board.move_chip("q1", Team::Black, hex_moves[1]);
    board.move_chip("a1", Team::White, hex_moves[2]);
    board.move_chip("a1", Team::Black, hex_moves[3]);
    board.move_chip("a2", Team::White, hex_moves[4]);
    board.move_chip("a1", Team::Black, hex_moves[5]);
    board.move_chip("s1", Team::White, hex_moves[6]);
    board.move_chip("a2", Team::Black, hex_moves[7]);
    board.move_chip("a3", Team::White, hex_moves[8]);

    board
}

pub fn game_snapshot_5() -> Board<Cube> {
    // Draw game tests - both bees are in trouble

    let mut board = Board::new(Cube::default());

    let moves_list = vec![
        (0, 0),   // wq1
        (0, 2),   // bq1
        (1, -1),  // wa1
        (-1, 3),  // ba1
        (-1, -1), // wa2
        (-1, 1),  // ba1
        (0, -2),  // ws1
        (-1, 3),  // ba2
        (0, -4),  // wa3
        (0, 4),   // ba3
        (1, 3),   // bs1
                  // can now move wa3 to (1,1) to defeat both teams
    ];

    // Convert to cube
    let hex_moves = moves_list
        .iter()
        .map(|xy| board.coord.mapfrom_doubleheight(DoubleHeight::from(*xy)))
        .collect::<Vec<Cube>>();

    board.move_chip("q1", Team::White, hex_moves[0]);
    board.move_chip("q1", Team::Black, hex_moves[1]);
    board.move_chip("a1", Team::White, hex_moves[2]);
    board.move_chip("a1", Team::Black, hex_moves[3]);
    board.move_chip("a2", Team::White, hex_moves[4]);
    board.move_chip("a1", Team::Black, hex_moves[5]);
    board.move_chip("s1", Team::White, hex_moves[6]);
    board.move_chip("a2", Team::Black, hex_moves[7]);
    board.move_chip("a3", Team::White, hex_moves[8]);
    board.move_chip("a3", Team::Black, hex_moves[9]);
    board.move_chip("s1", Team::Black, hex_moves[10]);

    board
}

pub fn game_snapshot_6() -> Board<Cube> {
    // Ladybird moves, based loosely on /reference/tests/bug3.png

    let mut board = Board::new(Cube::default());

    let moves_list = vec![
        (0, 0),  // bq1
        (0, -2), // wq1
        (0, -4), // wl1
        (1, -3), // wa2
        (2, -2), // wa1
        (3, -1), // wa3
                 // can now move wl1 to (1,-5) for backtrack, or (0,2) for advance, or -1,-3 = illegal
    ];

    // Convert to cube
    let hex_moves = moves_list
        .iter()
        .map(|xy| board.coord.mapfrom_doubleheight(DoubleHeight::from(*xy)))
        .collect::<Vec<Cube>>();

    board.move_chip("q1", Team::Black, hex_moves[0]);
    board.move_chip("q1", Team::White, hex_moves[1]);
    board.move_chip("l1", Team::White, hex_moves[2]);
    board.move_chip("a2", Team::White, hex_moves[3]);
    board.move_chip("a1", Team::White, hex_moves[4]);
    board.move_chip("a3", Team::White, hex_moves[5]);

    board
}

pub fn game_snapshot_7() -> Board<Cube> {
    // to test history.rs. reference/tests/snapshot_7.csv
    let mut board = Board::new(Cube::default());

    let moves_list = vec![
        (0, 0),   // wq1
        (0, 2),   // bq1
        (1, -1),  // wa1
        (-1, 3),  // ba1
        (-1, -1), // wa2
        (-1, 1),  // ba1
        (0, -2),  // ws1
        (-1, 3),  // ba2
        (0, -4),  // wa3
        (0, 4),   // ba3
    ];

    // Convert to cube
    let hex_moves = moves_list
        .iter()
        .map(|xy| board.coord.mapfrom_doubleheight(DoubleHeight::from(*xy)))
        .collect::<Vec<Cube>>();

    board.move_chip("q1", Team::White, hex_moves[0]);
    board.move_chip("q1", Team::Black, hex_moves[1]);
    board.move_chip("a1", Team::White, hex_moves[2]);
    board.move_chip("a1", Team::Black, hex_moves[3]);
    board.move_chip("a2", Team::White, hex_moves[4]);
    board.move_chip("a1", Team::Black, hex_moves[5]);
    board.move_chip("s1", Team::White, hex_moves[6]);
    board.move_chip("a2", Team::Black, hex_moves[7]);
    board.move_chip("a3", Team::White, hex_moves[8]);
    board.move_chip("a3", Team::Black, hex_moves[9]);

    board
}
