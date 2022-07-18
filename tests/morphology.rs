// Test morphological operations

use hoive::coord::{Coord, Cube};
use hoive::morphops;

#[test]
fn test_dilate() {
    // Place hex at 0,0,0 then dilate
    let vec = vec![(0, 0, 0)];
    let coord = Cube;

    let mut veccy = morphops::dilate(&coord, &vec);

    // raster scan so that we go top to bottom, left to right
    coord.raster_scan(&mut veccy);

    let expected = vec![
        (0, -1, 1),
        (1, -1, 0),
        (-1, 0, 1),
        (0, 0, 0),
        (1, 0, -1),
        (-1, 1, 0),
        (0, 1, -1),
    ];

    assert_eq!(expected, veccy);
}

#[test]
fn test_dilate_erode() {
    // Erosion should reverse dilation

    // Place hex at 0,0,0 then dilate
    let vec = vec![(0, 0, 0)];
    let coord = Cube;

    let dilated = morphops::dilate(&coord, &vec);

    assert_eq!(vec, morphops::erode(&coord, &dilated));
}

#[test]
fn test_close_reversible() {
    // Erosion should reverse dilation

    // Place hex at 0,0,0 then dilate
    let vec = vec![(0, 0, 0)];
    let coord = Cube;

    assert_eq!(vec, morphops::close(&coord, &vec));
}

#[test]
fn test_close_closes() {
    // Close a gap in the centre of a ring
    let ring = vec![
        (0, -1, 1),
        (1, -1, 0),
        (-1, 0, 1),
        (1, 0, -1),
        (-1, 1, 0),
        (0, 1, -1),
    ];

    let mut expected = vec![
        (0, -1, 1),
        (1, -1, 0),
        (-1, 0, 1),
        (0, 0, 0),
        (1, 0, -1),
        (-1, 1, 0),
        (0, 1, -1),
    ];

    let coord = Cube;

    // Raster scan everything because it'll sort the vectors out
    coord.raster_scan(&mut expected);

    let mut closed_ring = morphops::close(&coord, &ring);
    coord.raster_scan(&mut closed_ring);

    assert_eq!(expected, closed_ring);
}

#[test]
fn test_close_new() {
    // Close a gap in the centre of a ring
    let ring = vec![
        (0, -1, 1),
        (1, -1, 0),
        (-1, 0, 1),
        (1, 0, -1),
        (-1, 1, 0),
        (0, 1, -1),
    ];

    let mut expected = vec![(0, 0, 0)];

    let coord = Cube;

    // Raster scan everything because it'll sort the vectors out
    coord.raster_scan(&mut expected);

    let mut closed_ring = morphops::close_new(&coord, &ring);
    coord.raster_scan(&mut closed_ring);

    assert_eq!(expected, closed_ring);
}

#[test]
fn test_gap_closure() {
    // Create a gap that an ant shouldn't be able to pass (see /reference/before_gap.png)
    let before_gap = vec![
        (-1, 3, -2),
        (-1, 2, -1),
        (-1, 1, 0),
        (0, 0, 0),
        (1, -1, 0),
        (2, -1, -1),
        (2, 0, -2),
        (2, 1, -3),
        (1, 2, -3),
        (-1, 0, 1),
        (1, -2, 1),
    ];

    // Create ghost hexes to close the gaap that the ant can't pass
    let coord = Cube;
    let mut ghosts = morphops::gap_closure(&coord, &before_gap);

    // Ghosts should appear at these locations
    let mut expected = vec![(0, 1, -1), (1, 0, -1), (0, 2, -2), (1, 1, -2)];

    // Raster scan both expected and ghosts to sort vector ordering
    coord.raster_scan(&mut ghosts);
    coord.raster_scan(&mut expected);

    assert_eq!(expected, ghosts);
}
