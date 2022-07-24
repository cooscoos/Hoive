// Snapshots of boards used for other tests
use hoive::game::board::*;
use hoive::game::comps::Team;
use hoive::maths::{coord::Coord, coord::Cube};
use hoive::pmoore;


pub fn snapshot_2() -> Board<Cube> {

    // Set up a gameboard for some spider and bee tests

    let mut board = Board::default(Cube);


    let moves_list = vec![
        (0,0),
        (0,-2),
        (0,-4),
    ];

    // Convert to cube
    let hex_moves = moves_list
        .iter()
        .map(|xy| board.coord.mapfrom_doubleheight(*xy))
        .collect::<Vec<(i8, i8, i8)>>();

    pmoore::try_move(&mut board, "a1", Team::White, hex_moves[0]);
    pmoore::try_move(&mut board, "a1", Team::Black, hex_moves[1]);
    pmoore::try_move(&mut board, "a2", Team::Black, hex_moves[2]);


    board


}
