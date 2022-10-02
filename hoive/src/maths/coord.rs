use crate::maths::funcs;
use hex_spiral::{ring, ring_offset};
use serde::{Deserialize, Serialize};
/// Module defining hexagonal co-ordinate systems for the board to use.
use std::collections::HashSet;
use std::fmt::{Debug, Error};
use std::hash::Hash;
use std::ops::{Add, Sub};
use std::str::FromStr;

/// A trait ensuring all genetic hex coordinate systems utilise the same methods
/// Any coordinate system that is used by game logic must implement this trait.
pub trait Coord:
    Debug
    + Hash
    + PartialOrd
    + Ord
    + Eq
    + Clone
    + Copy
    + Add
    + Sub
    + Add<Output = Self>
    + Sub<Output = Self>
{
    fn default() -> Self;
    fn new(x: i8, y: i8, z: i8) -> Self;
    fn new_layer(x: i8, y: i8, z: i8, l: i8) -> Self;
    fn get_layer(&self) -> i8;
    fn vector_sqsum(&self) -> u32; // Square sum of vector components
    fn manhattan(&self) -> u32; // Manhattan distance: sum of the abs value of each component
    fn get_unitvec(&self, a: Self, b: Self) -> Self; // get unit vector going from a to b
    fn to_cube(&self) -> Cube; // Convert to cube coordinates
    fn neighbours_layer0<T: Coord>(&self, position: T) -> HashSet<T>; // a list of 6 neighbouring tiles on layer 0
    fn neighbours_all<T: Coord>(&self, position: T) -> HashSet<T>; // a list of up to 8 neighbouring tiles on all layers (neighbours + up and down)
    fn neighbours_onlayer<T: Coord>(&self, position: T, layer: i8) -> HashSet<T>; // all of the neighbours on a specified layer
    fn centroid_distance<T: Coord>(&self, hex1: T, hex2: T) -> f32; // calculate centroid distance between two hexes
    fn hex_distance<T: Coord>(&self, hex1: T, hex2: T) -> u32; // calculate distance between two hexes
    fn to_doubleheight<T: Coord>(&self, hex: T) -> DoubleHeight; // convert to doubleheight from self
    fn mapfrom_doubleheight(&self, hex: DoubleHeight) -> Self; // convert from doubleheight to self
    fn ascend(&mut self); // increase or decrease the layer number
    fn descend(&mut self);
    fn to_bottom(&self) -> Self; // drop to layer 0
    fn mapfrom_spiral(&self, hex: Spiral) -> Self; // convert from spiral coordinates to self
    fn mapto_spiral(&self) -> Result<Spiral, &'static str>;
}

/// Doubleheight coordinate system used by the ascii renderer
/// It doesn't need to implement the Coord trait because it is
/// never used by the game logic.
#[derive(
    Debug, Default, Serialize, Deserialize, Copy, Clone, Eq, PartialEq, Hash, PartialOrd, Ord,
)]
pub struct DoubleHeight {
    pub col: i8,
    pub row: i8,
    pub l: i8, // the layer
}

impl ToString for DoubleHeight {
    /// Convert doubleheight to string. This ignores layer number as it's never used for recording moves
    fn to_string(&self) -> String {
        format!("{},{}", self.col, self.row)
    }
}

impl FromStr for DoubleHeight {
    /// Convert comma-separated str e.g. "0,0" to Doubleheight ignoring layer as it's never used for recording moves
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // separate the input by comma
        let items = s.split(',').collect::<Vec<&str>>();

        let col = items[0]
            .parse::<i8>()
            .expect("Error parsing input col into i8");
        let row = items[1]
            .parse::<i8>()
            .expect("Error parsing input row into i8");

        Ok(DoubleHeight::from((col, row)))
    }
}

impl DoubleHeight {
    /// Parse col, row, layer into doubleheight
    pub fn new(col: i8, row: i8, l: i8) -> Self {
        DoubleHeight { col, row, l }
    }

    /// Parse tuple into doubleheight, ignoring layer
    pub fn from(colrow: (i8, i8)) -> Self {
        DoubleHeight {
            col: colrow.0,
            row: colrow.1,
            l: 0,
        }
    }

    /// Convert to another coord system
    pub fn mapto<T: Coord>(self, coord: T) -> T {
        coord.mapfrom_doubleheight(self)
    }
}

/// Spiral coordinate system used to save the game board state as a string
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct Spiral {
    pub u: usize,
    pub l: i8, // the layer
}

/// Cube coordinate system, used by game logic
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub struct Cube {
    q: i8,
    r: i8,
    s: i8,
    l: i8, // the layer
}

/// Define how to add two vectors in Cube coordinates
impl Add for Cube {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        Self {
            q: self.q + other.q,
            r: self.r + other.r,
            s: self.s + other.s,
            l: self.l + other.l,
        }
    }
}

/// Define how to subtract two vectors in Cube coordinates
impl Sub for Cube {
    type Output = Self;
    fn sub(self, other: Self) -> Self {
        Self {
            q: self.q - other.q,
            r: self.r - other.r,
            s: self.s - other.s,
            l: self.l - other.l,
        }
    }
}

/// Methods for Cube coordinates (trait: Coord)
impl Coord for Cube {
    fn default() -> Self {
        Cube::new(0, 0, 0)
    }

    fn new(q: i8, r: i8, s: i8) -> Self {
        Cube { q, r, s, l: 0 }
    }

    fn new_layer(q: i8, r: i8, s: i8, l: i8) -> Self {
        Cube { q, r, s, l }
    }

    fn get_layer(&self) -> i8 {
        self.l
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

    /// Get the unit vector (direction) from a to b
    fn get_unitvec(&self, a: Self, b: Self) -> Self {
        // Get the vector going from a to b
        let res = b - a;

        // find normalisation. Divide by 2 because translating from 3-coord cubic to a hex grid needs
        let norm_sq = (res.vector_sqsum() / 2) as f32;
        let norm = norm_sq.sqrt() as i8;

        // This is the unit vector going from a to b
        Cube::new(res.q / norm, res.r / norm, res.s / norm)
    }

    /// Convert to cube coordinates
    fn to_cube(&self) -> Self {
        *self
    }

    /// Get all of the neighbours but on layer 0
    fn neighbours_layer0<T: Coord>(&self, position: T) -> HashSet<T> {
        // Check layer 0 regardless of what layer we're on
        let position = position.to_bottom();
        HashSet::from([
            position + T::new(1, -1, 0),
            position + T::new(1, 0, -1),
            position + T::new(0, 1, -1),
            position + T::new(-1, 1, 0),
            position + T::new(-1, 0, 1),
            position + T::new(0, -1, 1),
        ])
    }

    /// Get all of the neighbours on my own layer, plus any chips above and below me
    fn neighbours_all<T: Coord>(&self, position: T) -> HashSet<T> {
        HashSet::from([
            position + T::new(1, -1, 0),
            position + T::new(1, 0, -1),
            position + T::new(0, 1, -1),
            position + T::new(-1, 1, 0),
            position + T::new(-1, 0, 1),
            position + T::new(0, -1, 1),
            position + T::new_layer(0, 0, 0, 1), // one layer up
            position - T::new_layer(0, 0, 0, 1), // one layer down
        ])
    }

    /// Get my neighbours but on specified layer
    fn neighbours_onlayer<T: Coord>(&self, position: T, layer: i8) -> HashSet<T> {
        // Start at layer 0 so we can add the layer number to it
        let position = position.to_bottom();
        HashSet::from([
            position + T::new_layer(1, -1, 0, layer),
            position + T::new_layer(1, 0, -1, layer),
            position + T::new_layer(0, 1, -1, layer),
            position + T::new_layer(-1, 1, 0, layer),
            position + T::new_layer(-1, 0, 1, layer),
            position + T::new_layer(0, -1, 1, layer),
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
    fn to_doubleheight<T: Coord>(&self, hex: T) -> DoubleHeight {
        let cube_position = hex.to_cube();

        let col = cube_position.q;
        let row = 2 * cube_position.r + cube_position.q;
        DoubleHeight {
            col,
            row,
            l: cube_position.l,
        }
    }

    /// Map doubleheight coordinates to cube coordinates
    fn mapfrom_doubleheight(&self, hex: DoubleHeight) -> Self {
        let q = hex.col; // columns (x)
        let r = (hex.row - hex.col) / 2; // rows (y)
        let s = -q - r;

        Cube { q, r, s, l: hex.l }
    }

    /// Go up one layer
    fn ascend(&mut self) {
        self.l += 1;
    }

    /// Go down one layer
    fn descend(&mut self) {
        self.l -= 1;
    }

    /// Go to layer 0
    fn to_bottom(&self) -> Self {
        Cube::new(self.q, self.r, self.s)
    }

    /// Convert spiral hex coordinate x to cube coords (q,r,s).
    fn mapfrom_spiral(&self, hex: Spiral) -> Self {
        let x = hex.u;
        // The origin is a special case: return (0,0,0)
        if x == 0 {
            return Cube::default();
        }

        // Find the ring index and ring-offset for this spiral
        let ring_index = ring(x) as f32;
        let ring_offset = ring_offset(ring_index as usize) as f32;

        // Calculate q and r
        let q = growing_trunc_tri(x as f32, ring_index, ring_offset, 0.0);
        let r = growing_trunc_tri(x as f32, ring_index, ring_offset, 4.0);

        // Could alternatively manually calculate s as:
        // let s = growing_trunc_tri(x, ring_offset, p, ring_index, -4.0);
        let s = -q - r;

        Cube::new_layer(q as i8, r as i8, s as i8, hex.l)
    }

    fn mapto_spiral(&self) -> Result<Spiral, &'static str> {
        // The origin is a special case, return 0.
        if self.q == 0 && self.r == 0 && self.s == 0 {
            return Ok(Spiral { u: 0, l: self.l });
        }

        // Make sure we've been passed a valid cube coordinate. The components should sum to 0.
        if self.q + self.r + self.s != 0 {
            return Err("q + r + s != 0");
        }

        // Find the ring index based on the maximum absolute value of q, r or s.

        let ring_index = [self.q.abs(), self.r.abs(), self.s.abs()]
            .into_iter()
            .max()
            .unwrap() as usize;

        let ring_offset = ring_offset(ring_index);

        // We now know approximately where we are in the truncated triangle wave.
        // If we start at x = ring_offset and calculate q,r,s values from this point up to
        // x = (ring_offset + ring_index * 6), we should find matching q, r, s values for some value of x.

        let x = ring_offset..(ring_offset + ring_index * 6);

        println!("Search area is..{:?}", x);

        match x
            .into_iter()
            .map(|u| (u, self.mapfrom_spiral(Spiral { u, l: self.l })))
            .find(|(_, c)| c == self)
            .map(|(x, _)| x)
        {
            Some(value) => Ok(Spiral {
                u: value,
                l: self.l,
            }),
            None => Err("Couldn't find a solution"),
        }
    }
}

/// Calculates y = f(x) where f is a truncated triangle wave of initial period, p = 6, and amplitude, a = 1.5
/// The amplitude and period increase each cycle.
/// - c is the cycle number that we're currently on (i.e. c=1 for the first cycle, and so on)
/// - x_prime is the value of x that this cycle began on
/// - phi is a phase shift in the triangle wave
fn growing_trunc_tri(x: f32, c: f32, x_prime: f32, phi: f32) -> i32 {
    // The base period of the triangle wave during cycle 1 (the number of sides a hexagon has)
    let p = 6.0;

    // How far along we are in the current cycle
    let offset_x = x - x_prime;

    // We'll use the modulo version of the equation for a triangle wave
    // https://en.wikipedia.org/wiki/Triangle_wave
    // But we'll modify it so that the cycle number is used to multiply the amplitude and period,
    // making the triangle wave get taller and broader each cyle. Define some params used in the calc:
    let s = offset_x - (c / 4.0) * (2.0 * phi + p);
    let p_star = c * p;

    // Here y_1 = g(x), where g is the triangle wave before it's truncated
    let y_1 = 6.0 / p * (funcs::modulo(s, p_star) - c * p / 2.0).abs() - 1.5 * (c);

    // We now truncate the wave so that it never has an amplitude greater than the cycle number
    match y_1.abs() > c {
        true => (y_1.signum() * c) as i32,
        false => y_1 as i32,
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
