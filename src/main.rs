use hoive::game::comps::Team;

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
    pmoore::turn(board, first);

    // repeat, will all be integrated soon
    let coord = hoive::maths::coord::Cube;
    let mut board = Board::default(coord);
    // show black player's chips only
    // println!(
    //     "Black player's chips: {:?}",
    //     board.list_chips(Some(Team::Black))
    // );

    // // show all chips
    // println!("Both team's chips: {:?}", board.list_chips(None));

    // Place black spider 1 at HECS position (1,0,0)
    println!("turn 1");
    board.try_move("s1", Team::Black, (1, 0, 0));

    // Place white spider 1 next to it
    println!("turn 2");
    board.try_move("s1", Team::White, (0, 1, 0));

    // place black spider 2 next to black spider 1
    println!("turn 3");
    board.try_move("s2", Team::Black, (0, 0, 0));

    // place white spider 2 next to white spider 1, and so on
    println!("turn 4");
    board.try_move("s2", Team::White, (1, 1, 0));

    println!("turn 5");
    board.try_move("s3", Team::White, (1, 1, 1));

    println!("turn 6");
    board.try_move("s4", Team::White, (0, 2, 0));

    // That's all the chips placed, let's try move white spider 3 to the moon
    println!("turn 6");
    board.try_move("s2", Team::Black, (8, 8, 8));

    // TODO:
    // Simple to complex moves:
    // ant, bee, spider
    // ladybird
    // beetle
    // grashopper
    // pillbug
    // mosquito
}
