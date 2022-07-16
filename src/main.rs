use hoive::{Board, Team};

//use Hoive::Player;

fn main() {
    // Some code to show how to use some methods and functions coded so far
    // See tests for more examples

    // initialise a player (pointless at the moment)
    // let mut p1 = Player::default(Team::Black);

    // initialise a game board - it comes with 4 spiders for each team

    let screen = hoive::coord::Hecs;

    let mut board = Board::default(screen);

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
