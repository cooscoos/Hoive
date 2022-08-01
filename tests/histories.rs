use hoive::game::{board::Board, history};
use hoive::maths::coord::Cube;
use hoive::draw;


#[test]
fn history_load() {

    let mut board = Board::default(Cube);
    let filename = "test".to_string();

    history::emulate(&mut board, filename);

    // Show the board
    println!("{}", draw::show_board(&board,5));
}