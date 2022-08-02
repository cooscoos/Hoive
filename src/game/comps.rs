// Components of a game: the teams and chips

use std::collections::HashMap;
use std::ops::Not;

// Enum for two teams
#[derive(Hash, Eq, PartialEq, Debug, Clone, Copy)]
pub enum Team {
    Black,
    White,
}

// Tell me who the other team are when I use !team
impl Not for Team {
    type Output = Self;
    fn not(self) -> Self::Output{
        match self {
            Team::Black => Team::White,
            Team::White => Team::Black,
        }
    }
}

// Struct for chips in a game
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
    // 2 beetles, 3 grasshoppers, 1 each of mosquito, ladybird, pill bug
    HashMap::from([
        // Black team's chips
        (Chip::new("s1", Team::Black), None),
        (Chip::new("s2", Team::Black), None),
        (Chip::new("a1", Team::Black), None),
        (Chip::new("a2", Team::Black), None),
        (Chip::new("a3", Team::Black), None),
        (Chip::new("q1", Team::Black), None),
        (Chip::new("l1", Team::Black), None),
        (Chip::new("p1", Team::Black), None),
        // White team's chips
        (Chip::new("s1", Team::White), None),
        (Chip::new("s2", Team::White), None),
        (Chip::new("a1", Team::White), None),
        (Chip::new("a2", Team::White), None),
        (Chip::new("a3", Team::White), None),
        (Chip::new("q1", Team::White), None),
        (Chip::new("l1", Team::White), None),
        (Chip::new("p1", Team::White), None),
    ])
}

pub fn test_chips() -> HashMap<Chip, Option<(i8, i8, i8)>> {
    // During some tests we want lots of chips that move freely. Give each team 8 ants, 1 bee
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

// Convert a chip_name (String on the heap) to a static str (on the stack)
pub fn convert_static(chip_string: String) -> Option<&'static str> {
    // Get all possible chip names
    let chips = starting_chips();

    // Find the chip name that matches the chip_string and return that chip's name as static str
    chips
        .into_iter()
        .map(|(c, _)| c.name)
        .find(|n| *n.to_string() == chip_string)

}
