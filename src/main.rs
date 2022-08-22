use hoive::game::{board::Board, movestatus::MoveStatus};
use hoive::maths::coord::Coord;
use hoive::pmoore;

fn main() {
    // Initialise game board in cube co-ordinates
    let coord = hoive::maths::coord::Cube::default();
    let mut board = Board::new(coord);

    // Say hello, tell players who goes first
    let first = pmoore::intro();

    // Loop game until someone wins
    loop {
        if let MoveStatus::Win(_) = pmoore::take_turn(&mut board, first) {
            println!("Play again? y/n");
            let textin = pmoore::get_usr_input();
            match textin {
                _ if textin == "y" => main(),
                _ => break,
            }
        }
    }
}
