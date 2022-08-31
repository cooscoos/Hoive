/// API is the middleman between the game's logic and the front-end. It converts string commands from the front
/// end into commands the board understands, and converts responses from the board into human-readable strings.
///
use crate::game::{board::Board, movestatus::MoveStatus};
use crate::maths::coord::Coord;
use crate::maths::coord::Cube;

/// Start a new game, create a db respond with how it went
fn new_game() {
    // Initialise game board in cube co-ordinates
    let coord = Cube::default();
    let mut board = Board::new(coord);
}

// We need a way of storing a board as a string in an sqlitedb
// need a table called gamestate which has:
// session id, a board (string representing board), user1, user2, current-player, ended (bool)

// Then have the option to find an existing session without a user2 and join it as a player

// let encoded = board.encode_spiral();
// println!("The spiral string is:\n {}", encoded);
// let newboard = board.decode_spiral(encoded);
// println!("SPIRAL BOARD\n{}", draw::show_board(&newboard));
