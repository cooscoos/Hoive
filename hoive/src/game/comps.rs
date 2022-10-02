use crate::maths::coord::{Coord, Cube};
use serde::{Deserialize, Serialize};
/// Module with the components of a game: the teams and chips
use std::collections::HashMap;
use std::fmt::Error;
use std::hash::Hash;
use std::ops::Not;
use std::str::FromStr;

/// Enum for the two teams, Team::Black and Team::White
#[derive(Hash, Eq, Ord, PartialOrd, PartialEq, Debug, Clone, Copy, Deserialize, Serialize)]
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

// Converts team to a string "B", "W"
impl ToString for Team {
    fn to_string(&self) -> String {
        match self {
            Team::Black => "B".to_string(),
            Team::White => "W".to_string(),
        }
    }
}

// Converts "B" or "W" into a Team
impl FromStr for Team {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "B" => Ok(Team::Black),
            "W" => Ok(Team::White),
            _ => panic!("Unrecognised team str"),
        }
    }
}

/// Struct for the chips in a game
/// Each chip has a team and also a unique name that defines its animal and number.
///
/// For example:
/// * the first spider chip is s1,
/// * the third ant chip is a3.
#[derive(Hash, Eq, PartialEq, PartialOrd, Ord, Debug, Clone, Copy, Deserialize, Serialize)]
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

    pub fn get_char(&self) -> char {
        match self.name.chars().next() {
            Some(chara) => chara,
            None => panic!("Chip has invalid name"),
        }
    }

    /// Elevate a beetle
    pub fn elevate(&self) -> Self {
        let new_name = match self.name {
            "b1" => "b1*",
            "b2" => "b2*",
            "mb" => "mb*",
            "m1" => "mb*",
            _ => panic!("Tried to elevate something that can't elevate"),
        };
        Chip {
            name: new_name,
            team: self.team,
        }
    }

    /// Renames a mosquito from "m1" to "mc" where c is the first char of
    /// the victim chip, e.g. if victim is ant, rename to "ma".
    pub fn morph(self, victim: Chip) -> Option<Self> {
        // Get the first letter of the victim
        let append = victim.get_char();

        // If the victim is another mosquito, we can't suck, return None
        let new_name = match append {
            'a' => "ma",
            'q' => "mq",
            's' => "ms",
            'g' => "mg",
            'b' => "mb",
            'l' => "ml",
            'p' => "mp",
            'm' => return None,
            _ => panic!("Just tried to morph a mosquito into an unknown chip"),
        };

        Some(Chip {
            name: new_name,
            team: self.team,
        })
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

fn possible_chips<T: Coord>() -> HashMap<Chip, Option<T>> {
    let names_list = vec![
        "s1", "s2", "a1", "a2", "a3", "q1", "l1", "p1", "b1", "b2", "g1", "g2", "g3", "m1", "ms",
        "ma", "mq", "ml", "mp", "mb", "mg",
    ];

    Chip::new_from_list(names_list)
}

/// Convert a chip_name String (on the heap) to a static str on the stack)
pub fn convert_static(chip_string: String) -> Option<&'static str> {
    // Get all possible chip names. We can use any coordinate system we want.
    let chips = possible_chips::<Cube>();

    // Find the chip name that matches the chip_string and return that chip's name as static str
    chips
        .into_iter()
        .map(|(c, _)| c.name)
        .find(|n| *n.to_string() == chip_string)
}

/// Convert a chip_name String (on the heap) to a static str on the stack). Except use starting chips
pub fn convert_static_basic(chip_string: String) -> Option<&'static str> {
    // Get all possible chip names. We can use any coordinate system we want.
    let chips = starting_chips::<Cube>();

    // Find the chip name that matches the chip_string and return that chip's name as static str
    chips
        .into_iter()
        .map(|(c, _)| c.name)
        .find(|n| *n.to_string() == chip_string)
}

/// Get the team of a chip based on whether the first letter is caps or not
pub fn get_team_from_chip(chip_string: &str) -> Team {
    match chip_string.chars().next().unwrap().is_uppercase() {
        true => Team::Black,
        false => Team::White,
    }
}
