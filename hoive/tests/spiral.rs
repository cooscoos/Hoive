use hex_spiral::{ring, ring_offset};
use hoive::maths::coord::Coord;
use hoive::maths::coord::Cube;
use hoive::maths::coord::Spiral;

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
