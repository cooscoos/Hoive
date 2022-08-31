/// We're going to use the spiral coordinate system defined by ljedrz
/// https://lib.rs/crates/hex-spiral
/// The crate hex-spiral will do some heavy lifting calculating the ring
/// number and ring offset for us.
use hex_spiral::{ring, ring_offset};

/// This function converts spiral coords to cube coords
/// See: https://www.redblobgames.com/grids/hexagons/
/// for a definition of Cube coords (q,r,s).
fn spiral_to_cube(&self, x: usize) -> (i8,i8,i8) {
    // The origin is a special case, so return (0,0,0) if we're here
    if x == 0 {
        return (0,0,0)
    }

    // Find the ring number and ring-offset for this spiral
    let ring_number = ring(hex.u) as f32;
    let ring_offset = ring_offset(hex.u) as f32;

    // Calculate q and r
    let q = growing_trunc_tri(x as f32, ring_number, ring_offset, 0.0);
    let r = growing_trunc_tri(x as f32, ring_number, ring_offset, 4.0);

    // You could manually calculate s like this:
    // let s = growing_trunc_tri(x, ring_offset, p, ring_number, -4.0);
    // But, we also know that q+r+s = 0 in Cube coords, so more efficient:
    let s = -q - r;

    (q,r,s)
}


/// Calculates y = f(x) where f is a truncated triangle wave of initial period, p = 6, and amplitude, a = 1.
/// The amplitude and period increase each cycle.
/// - c is the cycle number that we're currently on (e.g. c=1 for the first cycle, and so on)
/// - x_prime is the value of x that this cycle began on
/// - phi is a phase shift in the triangle wave
fn growing_trunc_tri(x: f32, c: f32, x_prime: f32, phi: f32) -> i8 {
    // The base period of the triangle wave during cycle 1 (the number of sides a hexagon has)
    let p = 6.0;

    // How far along we are in the current cycle
    let offset_x = x - x_prime;

    // We'll use the equation for triangle waves defined here:
    // https://en.wikipedia.org/wiki/Triangle_wave

    // But we'll modify (and simplify) it so that the cycle number is used to multiply the amplitude and period,
    // making the triangle wave get taller and broader each cyle. Define some params used in the calc:
    let s = offset_x - (c / 4.0) * (2.0 * phi + p);
    let p_star = c * p;

    // Here y_1 = g(x), where g is the triangle wave before it's truncated
    let y_1 = 6.0 / p * (modulo(s, p_star) - c * p / 2.0).abs() - 1.5 * (c);

    // We now truncate the wave so that it never has an amplitude greater than the cycle number
    match y_1 > c {
    true => (y_1.signum() * c) as i8,
    false => y_1 as i8,
    }
}

/// In Rust, a%b finds the remainder of a/b. This function finds the actual modulo (not the remainder) of a and b
fn modulo<T: std::ops::Rem<Output = T> + std::ops::Add<Output = T> + Copy>(a: T, b: T) -> T {
    ((a % b) + b) % b
}

#[cfg(test)]
mod tests {
    #[test]
    fn spiral_to_cube() {
        // Test a few input values in spiral coordinates
        let spiral_vals = vec![
            0,1,4,7,45
        ];

        // Try find their cube coords
        let result = spiral_vals
            .into_iter()
            .map(|x| spiral_to_cube(x))
            .collect::<Vec<_>>();

        // This is the result we expect to get
        let expected = vec![
            (0, 0, 0),
            (0, -1, 1),
            (0, 1, -1),
            (0, -2, 2),
            (4, 0, -4),
        ];

        assert_eq!(expected, result);
    }
}