// Test morphological operations

use hoive::maths::{coord::Coord, coord::Cube, morphops};
use std::collections::BTreeSet;
use std::collections::HashSet;

#[test]
fn test_dilate() {
    // Place hex at 0,0,0 then dilate
    let vec = HashSet::from([Cube::new(0, 0, 0)]);
    let coord = Cube::default();

    let dilated = morphops::dilate(&coord, &vec);

    let expected = vec![
        Cube::new(0, -1, 1),
        Cube::new(1, -1, 0),
        Cube::new(-1, 0, 1),
        Cube::new(0, 0, 0),
        Cube::new(1, 0, -1),
        Cube::new(-1, 1, 0),
        Cube::new(0, 1, -1),
    ];

    // Shove both results into a BTreeSet to ensure order is the same
    let dilated_ordered = dilated.into_iter().collect::<BTreeSet<_>>();
    let expected_ordered = expected.into_iter().collect::<BTreeSet<_>>();

    assert_eq!(expected_ordered, dilated_ordered);
}

#[test]
fn test_dilate_erode() {
    // Erosion should reverse dilation

    // Place hex at 0,0,0 then dilate
    let vec = HashSet::from([Cube::new(0, 0, 0)]);
    let coord = Cube::default();

    let dilated = morphops::dilate(&coord, &vec);

    assert_eq!(vec, morphops::erode(&coord, &dilated));
}

#[test]
fn test_close_reversible() {
    // Erosion should reverse dilation

    // Place hex at 0,0,0 then dilate
    let vec = HashSet::from([Cube::new(0, 0, 0)]);
    let coord = Cube::default();

    assert_eq!(vec, morphops::close(&coord, &vec));
}

#[test]
fn test_close_closes() {
    // Close a gap in the centre of a ring
    let ring = HashSet::from([
        Cube::new(0, -1, 1),
        Cube::new(1, -1, 0),
        Cube::new(-1, 0, 1),
        Cube::new(1, 0, -1),
        Cube::new(-1, 1, 0),
        Cube::new(0, 1, -1),
    ]);

    let mut expected = HashSet::from([
        Cube::new(0, -1, 1),
        Cube::new(1, -1, 0),
        Cube::new(-1, 0, 1),
        Cube::new(0, 0, 0),
        Cube::new(1, 0, -1),
        Cube::new(-1, 1, 0),
        Cube::new(0, 1, -1),
    ]);

    let coord = Cube::default();

    let mut closed_ring = morphops::close(&coord, &ring);

    // Shove both results into a BTreeSet to ensure order is the same
    let closed_ordered = closed_ring.into_iter().collect::<BTreeSet<_>>();
    let expected_ordered = expected.into_iter().collect::<BTreeSet<_>>();

    assert_eq!(expected_ordered, closed_ordered);
}

#[test]
fn test_close_new() {
    // Close a gap in the centre of a ring
    let ring = HashSet::from([
        Cube::new(0, -1, 1),
        Cube::new(1, -1, 0),
        Cube::new(-1, 0, 1),
        Cube::new(1, 0, -1),
        Cube::new(-1, 1, 0),
        Cube::new(0, 1, -1),
    ]);

    let mut expected = vec![Cube::new(0, 0, 0)];

    let coord = Cube::default();

    let mut closed_ring = morphops::close_new(&coord, &ring);

    // Shove both results into a BTreeSet to ensure order is the same
    let closed_ordered = closed_ring.into_iter().collect::<BTreeSet<_>>();
    let expected_ordered = expected.into_iter().collect::<BTreeSet<_>>();

    assert_eq!(expected_ordered, closed_ordered);
}

#[test]
fn test_gap_closure() {
    // Create a gap that an ant shouldn't be able to pass (see /reference/before_gap.png)
    let before_gap = HashSet::from([
        Cube::new(-1, 3, -2),
        Cube::new(-1, 2, -1),
        Cube::new(-1, 1, 0),
        Cube::new(0, 0, 0),
        Cube::new(1, -1, 0),
        Cube::new(2, -1, -1),
        Cube::new(2, 0, -2),
        Cube::new(2, 1, -3),
        Cube::new(1, 2, -3),
        Cube::new(-1, 0, 1),
        Cube::new(1, -2, 1),
    ]);

    // Create ghost hexes to close the gaap that the ant can't pass
    let coord = Cube::default();
    let mut ghosts = morphops::gap_closure(&coord, &before_gap);

    // Ghosts should appear at these locations
    let mut expected = vec![
        Cube::new(0, 1, -1),
        Cube::new(1, 0, -1),
        Cube::new(0, 2, -2),
        Cube::new(1, 1, -2),
    ];

    // Shove both results into a BTreeSet to ensure order is the same
    let ghosts_ordered = ghosts.into_iter().collect::<BTreeSet<_>>();
    let expected_ordered = expected.into_iter().collect::<BTreeSet<_>>();

    assert_eq!(expected_ordered, ghosts_ordered);
}
