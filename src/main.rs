use hoive::game::{board::Board, movestatus::MoveStatus};
use hoive::pmoore;

fn main() {
    // Initialise game board in cube co-ordinates
    let coord = hoive::maths::coord::Cube;
    let mut board = Board::new(coord);

    // Say hello, tell players who goes first
    let first = pmoore::intro();

    #[cfg(feature = "benchmarking")]
    {
        println!("It's benchmarking time!!")
    }

    // Loop game until someone wins
    loop {
        if let MoveStatus::Win(_) = pmoore::take_turn(&mut board, first) {
            break;
        }
    }
}
