use hoive::game::history::History;

/// Structs that are used by the database, server and client.
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::str::FromStr;

use super::schema::game_state;
use super::schema::user;

use hoive::game::{board::Board, comps::Team, history::Event};
use hoive::maths::coord::Cube;

#[derive(Serialize, Deserialize, Default, Queryable, Insertable, Debug, Clone)]
#[diesel(table_name = user)]
pub struct User {
    pub id: String,
    pub user_name: String,
}

#[derive(Serialize, Deserialize, Default, Queryable, Debug, Clone, PartialEq, Eq)]
pub struct GameState {
    pub id: String,
    pub board: Option<String>,
    pub user_1: Option<String>,
    pub user_2: Option<String>,
    pub winner: Option<String>,
    pub last_user_id: Option<String>,
    pub history: Option<String>,
}

impl GameState {

    /// Returns the player who does not have this user id
    pub fn not_this_user(&self, id: &str) -> Result<String, Box<dyn Error>> {
        match self.user_1.as_ref().unwrap() == &id.to_string() {
            true => Ok(self.user_2.as_ref().unwrap().to_owned()),
            false => Ok(self.user_1.as_ref().unwrap().to_owned()),
        }
    }

    /// Returns the team of the given user id
    pub fn which_team_user(&self, id: &str) -> Result<Team, Box<dyn Error>> {
        // User 1 is black team, user 2 is white team
        match id {
           _ if id == self.user_1.as_ref().unwrap() => Ok(Team::Black),
           _ if id == self.user_2.as_ref().unwrap() => Ok(Team::White),
           _ => panic!("Team undefined"),
        }

    }


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

    /// Returns the user is of the inactive player
    pub fn inactive_user(&self) -> Result<String, Box<dyn Error>> {
        // Find the active team
        let active_team = self.which_team()?;

        match !active_team {
            Team::Black => Ok(self.user_1.to_owned().unwrap()),
            Team::White => Ok(self.user_2.to_owned().unwrap()),
        }
    }

    /// Generate a board from a gamestate's spiral coordinates on the db
    pub fn to_cube_board(&self) -> Board<Cube> {
        let mut board = Board::new(Cube::default());

        // Get the board from the gamestate
        let board_state = match &self.board {
            Some(value) => value,
            None => return board,
        };

        // Generate a board based on this gamestate's board state
        board = board.decode_spiral(board_state.to_owned());

        // Add the history in
        board.history = match &self.history {
            Some(value) if value.is_empty() => History::default(), // No string, history empty
            Some(value) => History::from_str(value).expect("Problem parsing history"), // parse history from str
            None => History::default(), // No value, history empty
        };

        board
    }

    /// Add an event to a gamestate's history and return the history
    pub fn add_event(self, event: Event) -> String {
        // Get the existing history of the gamestate
        let mut history = match self.history {
            Some(value) => value,
            None => String::new(),
        };

        // Convert the event into a string
        let new_event = event.to_string();

        // append it and overwrite the gamestate's history
        history.push_str(&format!("{}/", new_event));

        history
    }

    /// Get the winner
    pub fn get_winner(&self) -> Option<Winner> {
        match &self.winner {
            Some(value) => {
                // We're getting a comma separated list where the first is the winner_team, then their id, then maybe an F (or not)

                let mut winner = Winner::default();

                let v: Vec<&str> = value.split(',').collect();
                let winner_team = v[0];

                if winner_team.starts_with('D') {
                    // It's a draw
                    winner.team = None;
                    Some(winner)
                } else {
                    let winner_name = v[1];

                    // Check if a forfeit happened
                    if value.ends_with('F') {
                        winner.forfeit = true;
                    }

                    // Check who won (black, white, or draw)
                    winner.team = match winner_team {
                        _ if winner_team.starts_with('B') => Some(Team::Black),
                        _ if winner_team.starts_with('W') => Some(Team::White),
                        _ => panic!("Unrecognised winner"),
                    };

                    winner.username = winner_name.to_owned();

                    Some(winner)
                }
            }
            None => None,
        }
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
    pub username: String,   // username of winner
}

impl Winner {
    /// Check if there was a winner and why, update the winner struct
    /// based on a returned winner value from server db's game_state
    /// and return true if there was a winner.
    pub fn happened(&mut self, winstring: &Option<String>) -> bool {
        match winstring {
            Some(value) => {
                // We're getting a comma separated value where the first is the winner_team, then their id, then maybe an F (or not)

                let v: Vec<&str> = value.split(',').collect();
                let winner_team = v[0];
                let winner_name = v[1];

                // Check if a forfeit happened
                if value.ends_with('F') {
                    self.forfeit = true;
                }

                // Check who won (black, white, or draw)
                self.team = match winner_team {
                    _ if winner_team.starts_with('B') => Some(Team::Black),
                    _ if winner_team.starts_with('W') => Some(Team::White),
                    _ if winner_team.starts_with('D') => None,
                    _ => panic!("Unrecognised winner"),
                };

                self.username = winner_name.to_owned();

                true // someone won
            }
            _ => false, // no winner yet
        }
    }
}
