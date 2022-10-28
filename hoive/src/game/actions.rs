use crate::maths::coord::{Coord, DoubleHeight};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

use super::comps::{convert_static_basic, Team};

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub enum Command {
    Start,
    Select,
    Move,
    Mosquito,
    Pillbug,
    SkipTurn,
    Execute,
}

// Converts team to a string "B", "W"
impl ToString for Command {
    fn to_string(&self) -> String {

        use Command::*;
        
        match self {
            Start => "".to_string(),
            Select => "//cmd select".to_string(),
            Move => "//cmd moveto".to_string(),
            Mosquito => "//cmd mosquito".to_string(),
            Pillbug => "//cmd pillbug".to_string(),
            Execute => "//cmd execute".to_string(),
            SkipTurn => "".to_string(),
        }

    }
}

/// Used to formulate requests for in-game actions from the Hoive server
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct BoardAction {
    pub name: String,            // chip name
    pub rowcol: Option<DoubleHeight>,    // destination row, col
    pub special: Option<String>, // Contains source (row,col) if doing mosquito/pillbug special
    pub neighbours: Option<BTreeSet<String>>, // the neighbours of that chip
    pub command: Command,   // a thing to tell the program what stage comes next
    pub message: String, // a thing to print to the user, defined at compile time, could be stat?

}

impl BoardAction {

    pub fn default() -> Self {
        BoardAction {
            name: String::new(),
            rowcol: None,
            special: None,
            neighbours: None,
            command: Command::Start,
            message: "".to_string(),
        }
    }
    /// Generate command to forfeit a game
    pub fn forfeit() -> Self {
        BoardAction {
            name: "".to_string(),
            rowcol: None,
            special: Some("forfeit".to_string()),
            neighbours: None,
            command: Command::Start,
            message: "".to_string(),
        }
    }

    /// Generate a command to skip your turn
    pub fn skip() -> Self {
        BoardAction {
            name: "".to_string(),
            rowcol: None,
            special: Some("skip".to_string()),
            neighbours: None,
            command: Command::Start,
            message: "".to_string(),
        }
    }

    /// Generate command to make a move
    pub fn do_move(
        chip_name: &str,
        active_team: Team,
        rowcol: DoubleHeight,
        special_string: String,
    ) -> Self {
        let case_chip_name = match active_team {
            Team::Black => chip_name.to_uppercase(),
            Team::White => chip_name.to_string(),
        };

        let special = match special_string.is_empty() {
            true => None,
            false => Some(special_string),
        };

        BoardAction {
            name: case_chip_name,
            rowcol: Some(rowcol),
            special,
            neighbours: None,
            command: Command::Start,
            message: "".to_string(),
        }
    }

    /// Get chip name as a &'static str without team capitalisation
    pub fn get_chip_name(&self) -> &'static str {
        convert_static_basic(self.name.to_lowercase()).unwrap()
    }

    /// Get the special string
    pub fn get_special(&self) -> String {
        self.special.as_ref().unwrap().to_owned()
    }

    /// Get destination in board coords
    pub fn get_dest<T: Coord>(&self, coord: T) -> T {
        let dheight = self.rowcol;
        coord.mapfrom_doubleheight(dheight.unwrap())
    }

    /// Get the team of the chips which are doing the action
    pub fn which_team(&self) -> Team {
        // Black chips get passed as uppercase, white as lowercase
        crate::game::comps::get_team_from_chip(&self.name)
    }
}
