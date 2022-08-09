use crate::maths::coord::{Coord, Cube};
/// Module with the components of a game: the teams and chips
use std::collections::HashMap;
use std::ops::Not;

/// Enum for the two teams, Team::Black and Team::White
#[derive(Hash, Eq, PartialEq, Debug, Clone, Copy)]
pub enum Team {
    Black,
    White,
}

// This allows us to find out who the opposing team are, e.g.
// !Team::White == Team::Black
impl Not for Team {
    type Output = Self;
    fn not(self) -> Self::Output {
        match self {
            Team::Black => Team::White,
            Team::White => Team::Black,
        }
    }
}

/// Struct for the chips in a game
/// Each chip has a team and also a unique name that defines its animal and number.
///
/// For example:
/// * the first spider chip is s1,
/// * the third ant chip is a3.
#[derive(Hash, Eq, PartialEq, Debug, Clone, Copy)]
pub struct Chip {
    pub name: &'static str,
    pub team: Team,
}

impl Chip {
    /// Create a new chip with given name and team.
    pub fn new(name: &'static str, team: Team) -> Self {
        Chip { name, team }
    }
}

/// Generate the starting chips for both teams.
///
/// All chips start off in the players' hands with position == None.
///  Each team gets:
/// * 1 bee, 2 spiders, 3 ants,
/// * 2 beetles, 3 grasshoppers,
/// * 1 each of mosquito, ladybird, pill bug.
pub fn starting_chips<T: Coord>() -> HashMap<Chip, Option<T>> {
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
        (Chip::new("b1", Team::Black), None),
        (Chip::new("b2", Team::Black), None),
        // White team's chips
        (Chip::new("s1", Team::White), None),
        (Chip::new("s2", Team::White), None),
        (Chip::new("a1", Team::White), None),
        (Chip::new("a2", Team::White), None),
        (Chip::new("a3", Team::White), None),
        (Chip::new("q1", Team::White), None),
        (Chip::new("l1", Team::White), None),
        (Chip::new("p1", Team::White), None),
        (Chip::new("b1", Team::White), None),
        (Chip::new("b2", Team::White), None),
    ])
}

/// Convert a chip_name String (on the heap) to a static str on the stack)
pub fn convert_static(chip_string: String) -> Option<&'static str> {
    // Get all possible chip names. We can use any coordinate system we want.
    let chips = starting_chips::<Cube>();

    // Find the chip name that matches the chip_string and return that chip's name as static str
    chips
        .into_iter()
        .map(|(c, _)| c.name)
        .find(|n| *n.to_string() == chip_string)
}
