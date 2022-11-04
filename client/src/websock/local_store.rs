///! Store info from WsGameSession, but locally

use hoive::game::actions::BoardAction;
use hoive::game::board::Board;
use hoive::game::comps::Team;
use hoive::maths::coord::Cube;
use server::models::GameState;

/// A stripped down version of WsGameSession (from websock server).
/// Allows us to store similar params, but for our local websock client.
#[derive(Default)]
pub struct LGameSession {
    /// unique client session id (mirrors the user_id in the sqlite db)
    pub id: String,

    /// Joined room, ("main" or game_state id, mirrored in sqlite db)
    pub room: String,

    /// Username
    pub name: Option<String>,

    /// The local client will send this before all messages to server
    pub precursor: String,

    /// In-game: Is it the client's turn in game?
    pub active: bool,

    /// In-game: Actions used to execute moves in Hoive games
    pub action: BoardAction,

    /// In-game: The current board
    pub board: Board<Cube>,

    /// In-game: What team the player is on
    pub team: Team,

    /// In-game: An in-game message to guide player on what to do next.
    /// This mirrors the BoardAction.message held on the server
    pub game_message: String,
}

impl LGameSession {
    /// Update local copy of board and active team based on a gamestate
    pub fn update(&mut self, gamestate: GameState) {
        let spiral_code = gamestate.board.as_ref().unwrap();
        self.board = self.board.decode_spiral(spiral_code.to_owned());
        self.active = self.id != gamestate.last_user_id.unwrap();
    }

    /// Get a string to say if it's your turn or not
    pub fn turn_string(&self) -> &'static str {
        match self.active {
            true => "It's your turn!",
            false => "Waiting for other player to take turn...",
        }
    }

    /// Are we currently in a game of Hoive?
    pub fn in_game(&self) -> bool {
        self.room != "main"
    }
}
