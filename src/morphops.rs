// Morphological operations. All require a co-ordinate system to know what they're doing.

use crate::coord::Coord;
use std::collections::HashSet;

// Dilation
pub fn dilate<T: Coord>(coord: &T, flat_vec: Vec<(i8, i8, i8)>) -> Vec<(i8, i8, i8)> {
    // Hashset for keeping track of chip locations
    let mut store: HashSet<(i8, i8, i8)> = HashSet::new();

    // Add all of the current hexes to the hashset
    flat_vec.iter().for_each(|v| {store.insert(*v);});

    // Dilate
    for position in flat_vec {
        // Get the co-ordinates of neighbouring hexes and add them to the hashset (this is equivalent to dilation with window size 1)
        let neighbour_hexes = coord.neighbour_tiles(position);
        neighbour_hexes.iter().for_each(|v| {store.insert(*v);});
    }
    store.into_iter().collect::<Vec<(i8,i8,i8)>>()
}

// Erosion
pub fn erode<T: Coord>(coord: &T, flat_vec: Vec<(i8, i8, i8)>) -> Vec<(i8, i8, i8)> {
    //Hashset for keeping track of chip locations
    let mut store: HashSet<(i8, i8, i8)> = HashSet::new();

    for position in flat_vec.clone() {
        // Get the co-ordinates of neighbouring hexes
        let neighbour_hexes = coord.neighbour_tiles(position);
        
        // if any of the neighbouring hex chips are empty, go to that neighbouring hex and delete all of the chips neighbouring it (this is erosion)
        // check if neighbours appear in flat_vec
        for elem in neighbour_hexes.iter() {
            for elem2 in flat_vec.clone().iter() {
                if elem != elem2 { // if a neighbouring hex is empty
                    // go to elem and delete all of its neighbours from the hashset
                    let for_deletion = coord.neighbour_tiles(*elem);
                    for_deletion.iter().for_each(|v| {store.remove(v);});
                }
            }
        }
    }
    store.into_iter().collect::<Vec<(i8,i8,i8)>>()
}
    
// Closing (dilate, then erode)
pub fn close<T: Coord>(coord: &T, flat_vec: Vec<(i8, i8, i8)>) -> Vec<(i8, i8, i8)> { 
    let dilated = dilate(coord, flat_vec);
    erode(coord, dilated)
}