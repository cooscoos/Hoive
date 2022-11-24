use hex_spiral::{ring, ring_offset};
use hoive::maths::coord::Coord;
use hoive::maths::coord::Cube;
use hoive::maths::coord::Spiral;
mod common;
use hoive::game::board::Board;
use hoive::game::comps::Chip;
use std::collections::BTreeMap;

/// Helper function to generate an ordered BTreeMap of chips on the board and their positions
fn ordered_map<T: Coord>(board: Board<T>) -> BTreeMap<T, Chip> {
    board
        .chips
        .into_iter()
        .filter(|(_, p)| p.is_some())
        .map(|(c, p)| (p.unwrap(), c))
        .collect::<BTreeMap<T, Chip>>()
}

/// Load a board in from filename, covert it to spiral code and back, and check it's still the same board
fn test_decoder(filename: &str) {
    // Load in a snapshot of a board
    let board = common::emulate::load_board(filename.to_string());

    // Convert it to spiral code and back to a board again
    let board_string = board.encode_spiral();
    println!("Board String: {}", board_string);
    let board_copy = board.decode_spiral(board_string);

    // Create ordered BTree maps of chips and their positions on the original and decoded board
    let original = ordered_map(board);
    let decoded = ordered_map(board_copy);

    assert_eq!(original, decoded);
}

#[test]
fn convert_spiral_to_cube() {
    // Test a few input values in spiral coordinates
    let spiral_vals: Vec<usize> = vec![0, 1, 4, 7, 8, 45];

    let cube = Cube::default();

    // Try find their cube coords
    let result = spiral_vals
        .into_iter()
        .map(|u| cube.mapfrom_spiral(Spiral { u, l: 0 }))
        .collect::<Vec<Cube>>();

    // This is the result we expect to get
    let expected = [
        (0, 0, 0),
        (0, -1, 1),
        (0, 1, -1),
        (0, -2, 2),
        (1, -2, 1),
        (4, 0, -4),
    ]
    .into_iter()
    .map(|(q, r, s)| Cube::new(q, r, s))
    .collect::<Vec<Cube>>();

    assert_eq!(expected, result);
}

#[test]
fn convert_cube_to_spiral() {
    // Test a few input values in cube coordinates
    let cube = [
        (0, 0, 0),
        (0, -1, 1),
        (0, 1, -1),
        (0, -2, 2),
        (1, -2, 1),
        (4, 0, -4),
    ]
    .into_iter()
    .map(|(q, r, s)| Cube::new(q, r, s));

    // Try find their spiral coords
    let result = cube
        .into_iter()
        .map(|c| c.mapto_spiral().unwrap())
        .collect::<Vec<Spiral>>();

    let expected = [0, 1, 4, 7, 8, 45]
        .into_iter()
        .map(|u| Spiral { u, l: 0 })
        .collect::<Vec<Spiral>>();

    assert_eq!(expected, result);
}

#[test]
fn convert_invalid_qrs() {
    let cube = Cube::new(-1, -1, 0);

    // An invalid set of cube coords
    assert_eq!(Err("q + r + s != 0"), cube.mapto_spiral())
}

#[test]
fn spiral_ring_test() {
    // Test a bunch of values give us the correct ring number
    let hex_values = [0, 3, 6, 7, 9, 17, 19, 37];
    let result = hex_values.into_iter().map(|v| ring(v)).collect::<Vec<_>>();

    // We expect the result to be the correct ring numbers
    let expected = vec![0, 1, 1, 2, 2, 2, 3, 4];
    assert_eq!(expected, result);
}

#[test]
fn spiral_offset_test() {
    // test a bunch of values and find the right offsets
    let ring_numbers = [0, 1, 2, 3, 4];
    let result = ring_numbers
        .into_iter()
        .map(|v| ring_offset(v))
        .collect::<Vec<_>>();

    // We expect the result to be the correct ring numbers
    let expected = vec![0, 1, 7, 19, 37];
    assert_eq!(expected, result);
}

#[test]
fn spiral_decoding() {
    // Convert a board to spiral coordinates and back to a board to check it matches
    test_decoder("snapshot_21")
}

#[test]
fn spiral_decoding_no_origin() {
    // Convert a board to spiral coordinates and back to a board to check it matches, but no chip at 0,0
    test_decoder("snapshot_22")
}
