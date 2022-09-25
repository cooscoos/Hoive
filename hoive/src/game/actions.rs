use serde::{Deserialize, Serialize};
use super::comps::Team;

/// Used to formulate requests for in-game actions from the Hoive server
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct BoardAction {
    pub name: String,            // chip name
    pub rowcol: (i8, i8),        // destination row, col
    pub special: Option<String>, // Contains source (row,col) if doing mosquito/pillbug special
}

impl BoardAction {

    /// Generate command to forfeit a game
    pub fn forfeit() -> Self {
        BoardAction {
            name: "".to_string(),
            rowcol: (0, 0),
            special: Some("forfeit".to_string()),
        }
    }

    /// Generate a command to skip your turn
    pub fn skip() -> Self {
        BoardAction {
            name: "".to_string(),
            rowcol: (0,0),
            special: Some("skip".to_string()),
        }
    }

    /// Generate command to make a move
    pub fn do_move(chip_name:&str, active_team: Team, row: i8, col: i8, special_string: String) -> Self{
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
            rowcol: (row, col),
            special: special,
        }
    }


}
