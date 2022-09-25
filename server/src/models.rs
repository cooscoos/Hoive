use super::schema::game_state;
use super::schema::user;
use serde::{Deserialize, Serialize};
use std::error::Error;
use hoive::{game::comps::Team, maths::coord::DoubleHeight};

#[derive(Serialize, Deserialize, Default, Queryable, Insertable, Debug, Clone)]
//#[table_name = "user"]
#[diesel(table_name = user)]
pub struct User {
    pub id: String,
    pub user_name: String,
    pub user_color: String,
}

#[derive(Serialize, Deserialize, Queryable, Debug, Clone)]
pub struct GameState {
    pub id: String,
    pub board: Option<String>,
    pub user_1: Option<String>,
    pub user_2: Option<String>,
    pub winner: Option<String>,
    pub last_user_id: Option<String>,
}


impl GameState {
    /// Which team's turn is it right now?
    pub fn whose_turn(&self) -> Result<Team, Box<dyn Error>> {
        // Find which team went last and return the opposite team
        let last_turn = self.last_user_id.as_ref();
        match self.last_user_id {
            _ if last_turn == Some(&"B".to_string()) => Ok(Team::White),
            _ if last_turn == Some(&"W".to_string()) => Ok(Team::Black),
            _ => panic!("Team is undefined"),
        }
    }
}

#[derive(Deserialize, Serialize, Insertable)]
//#[table_name = "game_state"]
#[diesel(table_name = game_state)]
pub struct NewGameState {
    pub id: String,
    pub board: Option<String>,
    pub user_1: Option<String>,
}




/// Used to formulate requests for in-game actions from the Hoive server
#[derive(Serialize, Debug, Clone)]
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

/// Used to carry information about who the winner was and why
#[derive(Default, Debug)]
pub struct Winner {
    pub team: Option<Team>, // who won?
    pub forfeit: bool,      // did they win because of a forfeit from other team?
}

impl Winner {
    /// Check if there was a winner and why, update the winner struct
    /// based on a returned winner value from server db's game_state
    /// and return true if there was a winner.
    pub fn happened(&mut self, winstring: &Option<String>) -> bool {
        println!("{:?}", self);
        println!("winstring: {:?}", winstring);

        match winstring {
            Some(value) => {
                if value.is_empty() {
                    return false;   // no winner
                }

                // Check if a forfeit happened
                if value.ends_with('F') {
                    self.forfeit = true;
                }

                // Check who won (black, white, or draw)
                self.team = match value {
                    _ if value.starts_with('B') => Some(Team::Black),
                    _ if value.starts_with('W') => Some(Team::White),
                    _ if value.starts_with('D') => None,
                    _ => panic!("Unrecognised winner"),
                };

                true // someone won
            }
            None => panic!("Server returned winstring = None. This should be impossible."),
        }
    }
}
