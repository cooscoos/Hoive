// Here we define hex co-ordinate systems for the game.

// First, some useful maths functions
// Vector subtraction
fn vector_subtract(a: &(i8, i8, i8), b: &(i8, i8, i8)) -> (i8, i8, i8) {
    (a.0 - b.0, a.1 - b.1, a.2 - b.2)
}

// Square sum of vector components
fn vector_sqsum(a: &(i8, i8, i8)) -> u32 {
    ((a.0).pow(2) + (a.1).pow(2) + (a.2).pow(2))
        .try_into()
        .unwrap()
}

// Hex coordinate systems we define need to have the following methods
pub trait Coord {
    fn neighbour_tiles(&self, position: (i8, i8, i8)) -> [(i8, i8, i8); 6]; // a list of 6 neighbouring tiles
    fn raster_scan(&self, flat_vec: &mut Vec<(i8, i8, i8)>); // a method to logically raster scan through each tile
    fn centroid_distance(&self, hex1: (i8, i8, i8), hex2: (i8, i8, i8)) -> f32; // calculate centroid distance between two hexes
    fn mapto_doubleheight(&self, hex: (i8, i8, i8)) -> (i8, i8); // convert to and from doubleheight co-ords for the ascii renderer
    fn mapfrom_doubleheight(&self, hex: (i8, i8)) -> (i8, i8, i8);
}

// Cube coordinate system
// https://www.redblobgames.com/grids/hexagons/
pub struct Cube;
impl Coord for Cube {
    // Get 6 neighbouring tile co-ordinates in cube co-ordinates
    fn neighbour_tiles(&self, position: (i8, i8, i8)) -> [(i8, i8, i8); 6] {
        let (q, r, s) = position;

        [
            (q + 1, r - 1, s),
            (q + 1, r, s - 1),
            (q, r + 1, s - 1),
            (q - 1, r + 1, s),
            (q - 1, r, s + 1),
            (q, r - 1, s + 1),
        ]
    }

    // Sort flat vector of co-ordinates in raster scan order:
    fn raster_scan(&self, flat_vec: &mut Vec<(i8, i8, i8)>) {
        // For cube co-ordinates, one way to raster scan (q,r,s) is:
        // r ascending first
        // then s descending
        // then q descending
        flat_vec
            .sort_by(|(q1, r1, s1), (q2, r2, s2)| (r1, s2, q2).partial_cmp(&(r2, s1, q1)).unwrap());
    }

    // Get centroid distance between two hexes
    fn centroid_distance(&self, hex1: (i8, i8, i8), hex2: (i8, i8, i8)) -> f32 {
        // Squared sum of components of vector distance
        let vector_distance = vector_subtract(&hex1, &hex2);
        let sq_sum = vector_sqsum(&vector_distance);

        ((sq_sum as f32) / 2.0).powf(0.5)
    }

    fn mapto_doubleheight(&self, hex: (i8, i8, i8)) -> (i8, i8) {
        // Convert from cube to doubleheight
        let col = hex.0;
        let row = 2 * hex.1 + hex.0;

        (col, row)
    }

    fn mapfrom_doubleheight(&self, hex: (i8, i8)) -> (i8, i8, i8) {
        // Convert from doubleheight to cube
        let q = hex.0; // columns (x)
        let r = (hex.1 - hex.0) / 2; // rows (y)
        let s = -q - r;

        (q, r, s)
    }
}

// Hexagonal Efficient Coordinate (HECS) co-ordinate system
// https://en.wikipedia.org/wiki/Hexagonal_Efficient_Coordinate_System
pub struct Hecs;
impl Coord for Hecs {
    // Get 6 neighbouring tile co-ordinates in HECS
    fn neighbour_tiles(&self, position: (i8, i8, i8)) -> [(i8, i8, i8); 6] {
        let (a, r, c) = position;

        [
            (1 - a, r - (1 - a), c - (1 - a)),
            (1 - a, r - (1 - a), c + a),
            (a, r, c - 1),
            (a, r, c + 1),
            (1 - a, r + a, c - (1 - a)),
            (1 - a, r + a, c + a),
        ]
    }

    // Sort flat vector of co-ordinates (a,r,c) in raster scan order:
    fn raster_scan(&self, flat_vec: &mut Vec<(i8, i8, i8)>) {
        // For HECS, one way to raster scan (a,r,c) is:
        // r descending first
        // then a descending
        // then c ascending
        flat_vec
            .sort_by(|(a1, r1, c1), (a2, r2, c2)| (r2, a2, c1).partial_cmp(&(r1, a1, c2)).unwrap());
    }

    // Get centroid distance between two hexes
    fn centroid_distance(&self, hex1: (i8, i8, i8), hex2: (i8, i8, i8)) -> f32 {
        !unimplemented!();
    }

    fn mapto_doubleheight(&self, hex: (i8, i8, i8)) -> (i8, i8) {
        // Convert from HECS to doubleheight
        !unimplemented!();
    }

    fn mapfrom_doubleheight(&self, hex: (i8, i8)) -> (i8, i8, i8) {
        // Convert from doubleheight to HECS
        !unimplemented!();
    }
}