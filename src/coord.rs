// Hex coordinate systems we define need to have the following methods
pub trait Coord {
    fn neighbour_tiles(&self, position: (i8, i8, i8)) -> [(i8, i8, i8); 6];
    fn raster_scan(&self, flat_vec: &mut Vec<(i8, i8, i8)>);
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
}
