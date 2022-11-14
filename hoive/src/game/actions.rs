use super::ask::Req;
use super::comps::{convert_static_basic, Team};
use crate::maths::coord::{Coord, DoubleHeight};
use serde::Serialize;
use std::collections::BTreeSet;
use crate::game::comps::Chip;

/// Used to formulate requests for in-game actions from a Hoive server
/// The fields of this struct are:
/// - name: the chip name (case sensitive, black team chips are capitalised)
/// - rowcol: the destination row, col of the move
/// - special: information on special actions (e.g. mosquito suck, pillbug sumo)
/// - neighbours: a sorted list of immediate neighbouring chips
/// - request: a request of what to do next: used to control UI logic
/// - message: information which is displayed to the player
#[derive(Serialize, Debug, Default, Clone, PartialEq, Eq)]
pub struct BoardAction {
    pub chip_name: String,
    pub rowcol: Option<DoubleHeight>,
    pub special: Option<String>,
    pub neighbours: Option<BTreeSet<Chip>>, // got rid of deserialize to prevent lifetime error, otherwise need to use String, may impact http webserver later...
    pub request: Req,
    pub message: String,
}

impl BoardAction {
    /// Default state is to ask the user to select a chip to move
    pub fn default() -> Self {
        BoardAction {
            chip_name: String::new(),
            rowcol: None,
            special: None,
            neighbours: None,
            request: Req::Select,
            message: "Select a chip to move. Hit enter to see the board and chips in your hand, h (help), w (skip turn), 'quit' (forfeit).".to_string(),
        }
    }

    // The following are only used by the http webserver. Could be deleted?

    /// Generate command to forfeit a game
    pub fn forfeit() -> Self {
        BoardAction {
            chip_name: "".to_string(),
            rowcol: None,
            special: Some("forfeit".to_string()),
            neighbours: None,
            request: Req::Nothing,
            message: "".to_string(),
        }
    }

    /// Generate a command to skip your turn
    pub fn skip() -> Self {
        BoardAction {
            chip_name: "".to_string(),
            rowcol: None,
            special: Some("skip".to_string()),
            neighbours: None,
            request: Req::Nothing,
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
            chip_name: case_chip_name,
            rowcol: Some(rowcol),
            special,
            neighbours: None,
            request: Req::Nothing,
            message: "".to_string(),
        }
    }

    /// Extract the chip name as a &'static str without capitalisation
    pub fn get_chip_name(&self) -> &'static str {
        convert_static_basic(self.chip_name.clone().to_lowercase()).unwrap()
    }

    /// Get the special string
    pub fn get_special(&self) -> String {
        self.special.as_ref().unwrap().to_owned()
    }

    /// Get the rowcol field in board coords
    pub fn get_dest<T: Coord>(&self, coord: T) -> T {
        let dheight = self.rowcol;
        coord.mapfrom_doubleheight(dheight.unwrap())
    }

    /// Get the team of the chips which are doing the action
    pub fn which_team(&self) -> Team {
        // Black chips get passed as uppercase, white as lowercase
        crate::game::comps::get_team_from_chip(&self.chip_name)
    }
}
