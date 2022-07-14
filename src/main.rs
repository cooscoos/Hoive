use hoive::{Board, Team};
//use Hoive::Player;

fn main() {
    // Some code to show how to use some methods and functions coded so far
    // See tests for more examples

    // initialise a player (pointless at the moment)
    // let mut p1 = Player::default(Team::Black);

    // initialise a game board - it comes with 4 spiders for each time
    let mut board = Board::default();

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
    // At this point, trying to visualise HECS in my head or on paper is getting cumbersome
    // Would be good to have simple graphical representation of board before proceeding further
    //
    // The order to program animal move logic from simplest to most complex would be:
    // spider, ant, bee first (same peice with diff no. moves)
    // then ladybird and beetle (similar "on top of other animals" logic)
    // then grashopper (mental logic)
    // then mosquito (need all other animals first)
    // then pillbug (if feeling brave)

    // For ant, need to code up the thing to check if it can squeeze through small gaps
    // I think the way to do this is to check each step of the ant one at a time and see if it ever moves
    // from a position A to B where:
    // position A) any two opposing edges of the hex are touching something
    // position B) more than two edges of the hex are touching something
    // Need to think about it more though
}
