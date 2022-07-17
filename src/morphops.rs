// Morphological operations. All require a co-ordinate system to know what they're doing.

use crate::coord::Coord;
use std::collections::HashSet;

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
pub fn erode<T: Coord>(coord: &T, flat_vec: &Vec<(i8, i8, i8)>) -> Vec<(i8, i8, i8)> {
    //Hashset for keeping track of chip locations
    let mut store: HashSet<(i8, i8, i8)> = HashSet::new();

    // Add all of the current hexes to the hashset
    flat_vec.iter().for_each(|v| {
        store.insert(*v);
    });

    for position in flat_vec.clone() {
        // Get the co-ordinates of neighbouring hexes
        let neighbour_hexes = coord.neighbour_tiles(position);

        // if it doesn't have all six neighbours, then it gets removed from the hashset
        // this is erosion
        let mut i = 0;
        for elem in neighbour_hexes.iter() {
            for elem2 in flat_vec.clone().iter() {
                if elem == elem2 {
                    i += 1;
                }
            }
        }

        if i!=6{
            store.remove(&position);
        }

    }
    store.into_iter().collect::<Vec<(i8, i8, i8)>>()
}



// Closing (dilate, then erode)
pub fn close<T: Coord>(coord: &T, flat_vec: &Vec<(i8, i8, i8)>) -> Vec<(i8, i8, i8)> {
    let dilated = dilate(coord, flat_vec);
    erode(coord, &dilated)
}
