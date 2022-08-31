/// Converts a duodecimal number to decimal (in range 0-143)
pub fn duo_to_decimal(number: String) -> usize {
    let first = number.chars().next().unwrap();
    let second = number.chars().nth(1).unwrap();

    12 * duodec_decoder(first) + duodec_decoder(second)
}
/// Converts a number between 0 and 143 to duodecimal (0-9,x=10,y=11)
pub fn decimal_to_duo(number: usize) -> String {
    if number > 143 {
        panic!("Function should not be used to convert numbers >143");
    }

    let rem1 = number % 12;

    let rem2 = match (number - rem1) == 0 {
        true => 0,
        false => (number / 12) % 12,
    };

    format!("{}{}", duodec_encoder(rem2), duodec_encoder(rem1))
}

/// Converts a number from 0-11 to a duodecimal (0-9 or x or y)
fn duodec_encoder(number: usize) -> String {
    match number {
        0..=9 => number.to_string(),
        10 => "x".to_string(),
        11 => "y".to_string(),
        _ => panic!("Undefined"),
    }
}

/// Converts duodecimal to a usize
fn duodec_decoder(number: char) -> usize {
    match number {
        'x' => 10,
        'y' => 11,
        _ => number.to_string().parse().unwrap(),
    }
}

/// In Rust, a % b finds the remainder of a / b. This function finds the actual modulo (not the remainder) of a and b.
pub fn modulo<T: std::ops::Rem<Output = T> + std::ops::Add<Output = T> + Copy>(a: T, b: T) -> T {
    ((a % b) + b) % b
}
