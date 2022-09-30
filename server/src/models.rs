/// Structs that are used by the database, server and client.
use serde::{Deserialize, Serialize};
use std::error::Error;

use super::schema::game_state;
use super::schema::user;

use hoive::game::{board::Board, comps::Team};
use hoive::maths::coord::{Coord, Cube};

#[derive(Serialize, Deserialize, Default, Queryable, Insertable, Debug, Clone)]
#[diesel(table_name = user)]
pub struct User {
    pub id: String,
    pub user_name: String,
}

#[derive(Serialize, Deserialize, Default, Queryable, Debug, Clone)]
pub struct GameState {
    pub id: String,
    pub board: Option<String>,
    pub user_1: Option<String>,
    pub user_2: Option<String>,
    pub winner: Option<String>,
    pub last_user_id: Option<String>,
}

impl GameState {
    /// Returns the team of the active player
    pub fn which_team(&self) -> Result<Team, Box<dyn Error>> {
        // Find which user went last and return the opposite team
        // If user_1 went last, it must be user_2 (white team) turn.
        match &self.last_user_id {
            Some(value) if value == self.user_1.as_ref().unwrap() => Ok(Team::White),
            Some(value) if value == self.user_2.as_ref().unwrap() => Ok(Team::Black),
            _ => panic!(
                "Team is undefined because last_user_id is {:?}",
                self.last_user_id
            ),
        }
    }

    /// Returns the user id of the active player
    pub fn which_user(&self) -> Result<String, Box<dyn Error>> {
        // Find the active team
        let active_team = self.which_team()?;

        match active_team {
            Team::Black => Ok(self.user_1.to_owned().unwrap()),
            Team::White => Ok(self.user_2.to_owned().unwrap()),
        }
    }

    /// Generate a board from a gamestate's spiral coordinates on the db
    pub fn to_cube_board(&self) -> Board<Cube> {
        let board = Board::new(Cube::default());

        // Get the board from the gamestate
        let board_state = match &self.board {
            Some(value) => value,
            None => return board,
        };

        // generate and return a board based on this gamestate
        board.decode_spiral(board_state.to_owned())
    }
}

#[derive(Deserialize, Serialize, Insertable)]
#[diesel(table_name = game_state)]
pub struct NewGameState {
    pub id: String,
    pub board: Option<String>,
    pub user_1: Option<String>,
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
        match winstring {
            Some(value) => {
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
            _ => false, // no winner yet
        }
    }
}
