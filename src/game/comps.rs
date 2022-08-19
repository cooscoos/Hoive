use crate::maths::coord::{Coord, Cube};
/// Module with the components of a game: the teams and chips
use std::collections::HashMap;
use std::hash::Hash;
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
    /// Create one new chip with given name and team.
    pub fn new(name: &'static str, team: Team) -> Self {
        Chip { name, team }
    }

    /// Create a HashMap of new chips for both teams at position None based a list of names
    pub fn new_from_list<T: Coord>(names_list: Vec<&'static str>) -> HashMap<Self, Option<T>> {
        let mut chip_map = HashMap::new();
        names_list.into_iter().for_each(|n| {
            chip_map.insert(Chip::new(n, Team::White), None); // white team
            chip_map.insert(Chip::new(n, Team::Black), None); // black team
        });
        chip_map
    }

    /// Elevate a beetle
    pub fn elevate(&self) -> Self {
        let new_name = match self.name {
            "b1" => "b1*",
            "b2" => "b2*",
            "m1" => "m1*",
            _ => panic!("Tried to elevate something that can't elevate"),
        };
        Chip {
            name: new_name,
            team: self.team,
        }
    }

    // Overwrite a chip name
    pub fn remosquito(self, append: char) -> Self {
        let new_name = match append {
            'a' => "ma",
            'q' => "mq",
            's' => "ms",
            'g' => "mg",
            'b' => "mb",
            'l' => "ml",
            'p' => "mp",
            _ => "m1",
        };

        Chip {
            name: new_name,
            team: self.team,
        }
    }

    // Get rid of suck
    pub fn demosquito(self) -> Self {
        Chip {
            name: "m1",
            team: self.team,
        }
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
    let names_list = vec![
        "s1", "s2", "a1", "a2", "a3", "q1", "l1", "p1", "b1", "b2", "g1", "g2", "g3", "m1",
    ];

    Chip::new_from_list(names_list)
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
