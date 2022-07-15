// struct for HECS co-ordinate systems
pub struct HECS;

impl HECS {
    pub fn default() -> Self {
        HECS{}
    }
    // Get neighbouring tile co-ordinates in HECS
    pub fn neighbour_tiles(&self, position: (i8, i8, i8)) -> [(i8, i8, i8); 6] {
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
    pub fn raster_scan(&self, flat_vec: &mut Vec<(i8,i8,i8)>) {
        // For HECS this is:
        // r descending first
        // then a descending
        // then c ascending
        flat_vec
        .sort_by(|(a1, r1, c1), (a2, r2, c2)| (r2, a2, c1).partial_cmp(&(r1, a1, c2)).unwrap());
    }

}
