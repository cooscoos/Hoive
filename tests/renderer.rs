// Test the renderer operations

use hoive::coord::{Coord, Cube};
use hoive::render;
use hoive::*;

#[test]
fn test_onerow() {
    //put down lots of chips
    let mut board = Board::default(Cube);
    board.try_move("s1", Team::Black, (0, 0, 0));
    board.try_move("s1", Team::White, (0, -1, 1));
    board.try_move("s1", Team::Black, (0, 1, -1));
    board.try_move("s2", Team::White, (1, -2, 1));
    board.try_move("s1", Team::Black, (1, 1, -2));
    board.try_move("s3", Team::White, (0, -2, 2));
    board.try_move("s1", Team::Black, (1, -3, 2));



    let dheight_hashmap = board.parse_out();
    let stringy = render::parse_row(dheight_hashmap);

    println!("{stringy}");


}
