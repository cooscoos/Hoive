// Test the renderer operations

use hoive::coord::{Coord, Cube};
use hoive::render;
use hoive::*;

#[test]
fn test_onerow() {

    let input = vec![
        None,
        Some(Chip::default("s2", Animal::Spider, Team::Black)),
        Some(Chip::default("s3", Animal::Spider, Team::Black)),
        None,
        Some(Chip::default("s2", Animal::Spider, Team::White)),
        Some(Chip::default("s3", Animal::Spider, Team::White)),
        None,
    ];

    

    println!("{:?}",render::parse_row(input,-5));



}