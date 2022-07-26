// Components of a game: the two teams and chips/tiles

use std::collections::HashMap;

// Enum for two teams
#[derive(Hash, Eq, PartialEq, Debug, Clone, Copy)]
pub enum Team {
    Black,
    White,
}

// Probably a better way of doing this, but this works.
// Tell me who the other team are
pub fn other_team(team: Team) -> Team {
    match team {
        Team::Black => Team::White,
        Team::White => Team::Black,
    }
}

// Struct for tiles/chips in a game
#[derive(Hash, Eq, PartialEq, Debug, Clone, Copy)]
pub struct Chip {
    pub name: &'static str, // identifies a unique animal/number, e.g. a2, s1 = ant 2, spider 1
    pub team: Team,         // black or white chip
}

impl Chip {
    // Create new chip
    pub fn new(name: &'static str, team: Team) -> Self {
        Chip { name, team }
    }
}

pub fn starting_chips() -> HashMap<Chip, Option<(i8, i8, i8)>> {
    // Give each team:
    // 1 bee, 2 spiders, 3 ants,
    // 2 beetles, 3 grasshoppers, 1 each of mosquito, ladybug, pill bug
    HashMap::from([
        // Black team's chips
        (Chip::new("s1", Team::Black), None),
        (Chip::new("s2", Team::Black), None),
        (Chip::new("a1", Team::Black), None),
        (Chip::new("a2", Team::Black), None),
        (Chip::new("a3", Team::Black), None),
        (Chip::new("q1", Team::Black), None),
        // White team's chips
        (Chip::new("s1", Team::White), None),
        (Chip::new("s2", Team::White), None),
        (Chip::new("a1", Team::White), None),
        (Chip::new("a2", Team::White), None),
        (Chip::new("a3", Team::White), None),
        (Chip::new("q1", Team::White), None),
    ])
}

pub fn test_chips() -> HashMap<Chip, Option<(i8, i8, i8)>> {
    // During tests we want lots of pieces that move freely, so give each team 8 ants and one bee
    HashMap::from([
        // Black team's chips
        (Chip::new("a1", Team::Black), None),
        (Chip::new("a2", Team::Black), None),
        (Chip::new("a3", Team::Black), None),
        (Chip::new("a4", Team::Black), None),
        (Chip::new("a5", Team::Black), None),
        (Chip::new("a6", Team::Black), None),
        (Chip::new("a7", Team::Black), None),
        (Chip::new("a8", Team::Black), None),
        (Chip::new("q1", Team::Black), None),
        // White team's chips
        (Chip::new("a1", Team::White), None),
        (Chip::new("a2", Team::White), None),
        (Chip::new("a3", Team::White), None),
        (Chip::new("a4", Team::White), None),
        (Chip::new("a5", Team::White), None),
        (Chip::new("a6", Team::White), None),
        (Chip::new("a7", Team::White), None),
        (Chip::new("a8", Team::White), None),
        (Chip::new("q1", Team::White), None),
    ])
}
