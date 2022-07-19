use hoive::game::comps::Team;
use rand::Rng; // To randomise which player goes first

use hoive::game::board::Board;

fn main() {
    // Initialise a game board using a cube co-ordinate system for hexes
    // The game board comes with 4 spiders, s1,s2,...,s4 for each team

    // gamemaster is the interface between the players and the board

    let coord = hoive::maths::coord::Cube;
    let mut board = Board::default(coord);

    println!(
        "
    ░█░█░█▀█░▀█▀░█░█░█▀▀
    ░█▀█░█░█░░█░░▀▄▀░█▀▀
    ░▀░▀░▀▀▀░▀▀▀░░▀░░▀▀▀"
    );

    // Select a random team to go first
    let mut rand = rand::thread_rng();
    let first = match rand.gen() {
        true => "\x1b[34;1mBlack\x1b[0m",
        false => "\x1b[35;1mWhite\x1b[0m",
    };

    println!("Welcome to Hoive. {first} team goes first.");

    println!("Select a tile to place or move. Type t to see list of tiles");

    // show black player's chips only
    println!(
        "Black player's chips: {:?}",
        board.list_chips(Some(Team::Black))
    );

    // show all chips
    println!("Both team's chips: {:?}", board.list_chips(None));

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

    // ant, spider, bee
    // ant doesn't need path planning or movement logic, he just needs this squeeze gaps rule
    // bee can use "must move to neighbour" rule
    // spider can use movement range https://www.redblobgames.com/grids/hexagons/
    // to prevent squeeze gaps, could fill gaps with ghost hexes that affect them (if hex surrounded by 5, then fill)

    // see size and spacing https://www.redblobgames.com/grids/hexagons/
    // fill the gap with a ghost when the centroids of "one-over" neighbouring hexes are 1.73 widths away
    // centroids 2 widths away is ok, no ghost.

    // need a way of finding a bridgeable gap or a "one-over" neighbour
    // find the neighbours of blank spaces...
    // a bridgeable gap happens when opposing faces of a ghost hex would be touched
    // any easier way?
}
