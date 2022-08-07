/// Module defining hexagonal co-ordinate systems for the board to use.
use std::collections::HashSet;
use std::hash::Hash;
use std::ops::{Add, Sub};


/// A trait ensuring all hex coordinate systems utilise the same methods
pub trait Coord: Hash + Eq + Clone + Copy + Add + Sub + Add<Output = Self> + Sub<Output = Self> {
    fn new(x:i8,y:i8,z:i8) -> Self;
    fn vector_sqsum(&self) -> u32;  // Square sum of vector components
    fn manhattan(&self) -> u32;     // Manhattan distance: sum of the abs value of each component
    fn to_cube(&self) -> CubePosition;       // Convert to cube coordinates
    fn neighbour_tiles<T: Coord>(&self, position: T) -> HashSet<T>; // a list of 6 neighbouring tiles
    fn centroid_distance<T: Coord>(&self, hex1: T, hex2: T) -> f32; // calculate centroid distance between two hexes
    fn hex_distance<T: Coord>(&self, hex1: T, hex2: T) -> u32; // calculate distance between two hexes
    fn mapto_doubleheight<T: Coord>(&self, hex: T) -> (i8, i8); // convert to and from doubleheight co-ords for the ascii renderer
    fn mapfrom_doubleheight<T: Coord>(&self, hex: (i8, i8)) -> Self;
}


/// Cube coordinate system
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct CubePosition{
    q: i8,
    r: i8,
    s: i8,
}

/// Define how to add two vectors in Cube coordinates
impl Add for CubePosition{
    type Output = Self;
    fn add(self, other: Self) -> Self {
        Self{
            q: self.q + other.q,
            r: self.r + other.r,
            s: self.s + other.s,
        }
    }
}

/// Define how to subtract two vectors in Cube coordinates
impl Sub for CubePosition{
    type Output = Self;
    fn sub(self, other: Self) -> Self {
        Self{
            q: self.q - other.q,
            r: self.r - other.r,
            s: self.s - other.s,
        }
    }
}

impl Coord for CubePosition {

    fn new(q:i8,r:i8,s:i8) -> Self {
        CubePosition { q, r, s }
    }

    /// Square sum of vector components
    fn vector_sqsum(&self) -> u32 {
        ((self.q).pow(2) + (self.r).pow(2) + (self.s).pow(2))
            .try_into()
            .unwrap()
    }

    /// Manhattan distance: sum of the abs value of each component
    fn manhattan(&self) -> u32 {
        ((self.q).abs() + (self.r).abs() + (self.s).abs())
            .try_into()
            .unwrap()
    }

    /// Convert to cube coordinates
    fn to_cube(&self) -> Self {
        *self
    }

    fn neighbour_tiles<T: Coord>(&self, position: T) -> HashSet<T> {
        HashSet::from([
            position + T::new(1, -1, 0),
            position + T::new(1, 0, -1),
            position + T::new(0, 1, -1),
            position + T::new(-1, 1, 0),
            position + T::new(-1, 0, 1),
            position + T::new(0, -1, 1),
        ])
    }

    /// Get the centroid distance between two hexes input in cube coordinates
    fn centroid_distance<T: Coord>(&self, hex1: T, hex2: T) -> f32 {
        // Calculate squared sum of vector distance
        let vector_distance = hex1 - hex2;
        let sq_sum = vector_distance.vector_sqsum();
        ((sq_sum as f32) / 2.0).powf(0.5)
    }

    /// Find the distance between two hexes input in cube coordinates.
    fn hex_distance<T: Coord>(&self, hex1: T, hex2: T) -> u32 {
        let vector_distance = hex1 - hex2;
        // Get absolute sum of each component divided by 2
        vector_distance.manhattan() / 2
    }


    /// Map cube coordinates to doubelheight
    fn mapto_doubleheight<T: Coord>(&self, hex: T) -> (i8, i8) {
        let cube_position = hex.to_cube();

        let col = cube_position.q;
        let row = 2 * cube_position.r + cube_position.q;
        (col, row)
    }

    /// Map doubleheight coordinates to cube coordinates
    fn mapfrom_doubleheight<T: Coord>(&self, hex: (i8, i8)) -> Self{

        let q = hex.0; // columns (x)
        let r = (hex.1 - hex.0) / 2; // rows (y)
        let s = -q - r;

        CubePosition{q,r,s}
    }
}







// Hexagonal Efficient Coordinate (HECS) co-ordinate system
// https://en.wikipedia.org/wiki/Hexagonal_Efficient_Coordinate_System

// Reached the end of its usefulness. Doesn't present any obvious benefits.

// pub struct Hecs;
// impl Coord for Hecs {
//     // Get 6 neighbouring tile co-ordinates in HECS
//     fn neighbour_tiles(&self, position: (i8, i8, i8)) -> [(i8, i8, i8); 6] {
//         let (a, r, c) = position;

//         [
//             (1 - a, r - (1 - a), c - (1 - a)),
//             (1 - a, r - (1 - a), c + a),
//             (a, r, c - 1),
//             (a, r, c + 1),
//             (1 - a, r + a, c - (1 - a)),
//             (1 - a, r + a, c + a),
//         ]
//     }

//     fn hex_distance(&self, hex1: (i8, i8, i8), hex2: (i8, i8, i8)) -> u32 {
//         !unimplemented!();
//     }

//     // Sort flat vector of co-ordinates (a,r,c) in raster scan order:
//     fn raster_scan(&self, flat_vec: &mut Vec<(i8, i8, i8)>) {
//         // For HECS, one way to raster scan (a,r,c) is:
//         // r descending first
//         // then a descending
//         // then c ascending
//         flat_vec
//             .sort_by(|(a1, r1, c1), (a2, r2, c2)| (r2, a2, c1).partial_cmp(&(r1, a1, c2)).unwrap());
//     }

//     // Get centroid distance between two hexes
//     fn centroid_distance(&self, hex1: (i8, i8, i8), hex2: (i8, i8, i8)) -> f32 {
//         !unimplemented!();
//     }

//     fn mapto_doubleheight(&self, hex: (i8, i8, i8)) -> (i8, i8) {
//         // Convert from HECS to doubleheight
//         !unimplemented!();
//     }

//     fn mapfrom_doubleheight(&self, hex: (i8, i8)) -> (i8, i8, i8) {
//         // Convert from doubleheight to HECS
//         !unimplemented!();
//     }
// }
