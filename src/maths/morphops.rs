// Morphological operations.

use std::collections::HashSet;

use super::coord::Coord;

// Dilation
pub fn dilate<T: Coord>(coord: &T, flat_vec: &Vec<(i8, i8, i8)>) -> Vec<(i8, i8, i8)> {
    // Hashset for keeping track of chip locations
    let mut store: HashSet<(i8, i8, i8)> = HashSet::new();

    // Add all of the current hexes to the hashset
    flat_vec.iter().for_each(|v| {
        store.insert(*v);
    });

    // Dilate
    for position in flat_vec {
        // Get the co-ordinates of neighbouring hexes and add them to the hashset (this is equivalent to dilation with window size 1)
        let neighbour_hexes = coord.neighbour_tiles(*position);
        neighbour_hexes.iter().for_each(|v| {
            store.insert(*v);
        });
    }
    store.into_iter().collect::<Vec<(i8, i8, i8)>>()
}

// Erosion
pub fn erode<T: Coord>(coord: &T, flat_vec: &[(i8,i8,i8)]) -> Vec<(i8, i8, i8)> {
    //Hashset for keeping track of chip locations
    let mut store: HashSet<(i8, i8, i8)> = HashSet::new();

    // Add all of the current hexes to the hashset
    flat_vec.iter().for_each(|v| {
        store.insert(*v);
    });

    for position in flat_vec {
        // Get the co-ordinates of neighbouring hexes
        let neighbour_hexes = coord.neighbour_tiles(*position);

        // if it doesn't have all six neighbours, then it gets removed from the hashset
        // this is erosion
        let mut i = 0;
        for elem in neighbour_hexes.iter() {
            for elem2 in flat_vec.iter() {
                if elem == elem2 {
                    i += 1;
                }
            }
        }

        if i != 6 {
            store.remove(position);
        }
    }
    store.into_iter().collect::<Vec<(i8, i8, i8)>>()
}

// Closing (dilate, then erode)
pub fn close<T: Coord>(coord: &T, flat_vec: &Vec<(i8, i8, i8)>) -> Vec<(i8, i8, i8)> {
    let dilated = dilate(coord, flat_vec);
    erode(coord, &dilated)
}

// Closing and returning only the new additions
pub fn close_new<T: Coord>(coord: &T, flat_vec: &Vec<(i8, i8, i8)>) -> Vec<(i8, i8, i8)> {
    // Do the closure
    let closed = close(coord, flat_vec);

    let mut store = HashSet::new();
    // Add all of the closed hexes to the hashset
    closed.iter().for_each(|v| {
        store.insert(*v);
    });

    // Delete all of the originals to see what's new
    // Add all of the closed hexes to the hashset
    flat_vec.iter().for_each(|v| {
        store.remove(v);
    });

    store.into_iter().collect::<Vec<(i8, i8, i8)>>()
}

// Closing, and then deleting new additions which don't have 5 or more neighbours
pub fn gap_closure<T: Coord>(coord: &T, flat_vec: &Vec<(i8, i8, i8)>) -> Vec<(i8, i8, i8)> {
    // get the ghost tiles from a closure
    let ghosts = close_new(coord, flat_vec);

    let mut store = HashSet::new();
    // Add all of the closed hexes to the hashset
    ghosts.iter().for_each(|v| {
        store.insert(*v);
    });

    // create a hashset of all tiles, ghost and real
    let mut all = HashSet::new();
    ghosts.iter().for_each(|v| {
        all.insert(*v);
    });
    flat_vec.iter().for_each(|v| {
        all.insert(*v);
    });

    for position in ghosts {
        // Get the co-ordinates of neighbouring hexes
        let neighbour_hexes = coord.neighbour_tiles(position);

        // if it doesn't have all five or more neighbours, then it gets removed from the hashset
        // this is light additional erosion

        // Count the neighbours
        let mut i = 0;
        for elem in neighbour_hexes.iter() {
            for elem2 in all.clone().iter() {
                if elem == elem2 {
                    i += 1;
                }
            }
        }

        // Delete if less than 5
        if i < 5 {
            store.remove(&position);
        }
    }
    store.into_iter().collect::<Vec<(i8, i8, i8)>>()
}
