use hoive::game::board::Board;
use hoive::pmoore;

fn main() {
    // Initialise a game board using a cube co-ordinate system for hexes
    // The game board comes with 4 spiders, s1,s2,...,s4 for each team
    let coord = hoive::maths::coord::Cube;
    let mut board = Board::default(coord);

    // Say hello and tell the player who is going first
    let first = pmoore::intro();

    // Game loop
    loop {
        pmoore::take_turn(&mut board, first);
    }

    // TODO:
    // Simple to complex moves:
    // ant, bee, spider
    // ladybird
    // beetle
    // grashopper
    // pillbug
    // mosquito
}
