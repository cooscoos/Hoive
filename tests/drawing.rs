// Test the renderer operations

use hoive::draw;
use hoive::maths::{coord::Coord, coord::Cube, morphops};
use hoive::*;
use std::collections::BTreeMap;
use std::collections::HashMap;

use hoive::game::board::*;
use hoive::game::comps::{Chip, Team};

#[test]
fn test_doubleheight_converter() {
    //put down lots of chips in Cube co-ords
    let mut board = Board::test_board(Cube);
    pmoore::try_move(&mut board, "a1", Team::Black, (0, 0, 0)); // centre
    pmoore::try_move(&mut board, "a1", Team::White, (-1, 1, 0)); // down and left
    pmoore::try_move(&mut board, "a2", Team::Black, (1, -1, 0)); // up and right
    pmoore::try_move(&mut board, "a3", Team::Black, (1, -2, 1)); // up from that

    // We'll test if the program parses this to a doubleheight coordinate HashMap correctly
    let dheight_hashmap = draw::to_dheight(&board, 5);

    // We'll ignore hex positions with None values, and just get where the chips are
    let dheight_ignorenone = dheight_hashmap
        .into_iter()
        .filter(|(p, c)| c.is_some())
        .map(|(p, c)| (p, c.unwrap()))
        .collect::<HashMap<(i8, i8), Chip>>();

    // Stuffing HashMaps into BTreeMaps will sort them based on the key (the xy co-ordinate)
    let dheight_tree: BTreeMap<(i8, i8), Chip> = dheight_ignorenone
        .into_iter()
        .map(|(v, k)| (v, k))
        .collect();

    // Now onto what the answer should be: manually code an equivalent board in doubleheight coords
    let mut expected_map = HashMap::new();
    expected_map.insert(
        (0, 0),
        Chip {
            name: "a1",
            //animal: Animal::Spider,
            team: Team::Black,
        },
    ); // centre
    expected_map.insert(
        (-1, 1),
        Chip {
            name: "a1",
            //animal: Animal::Spider,
            team: Team::White,
        },
    ); // down and left
    expected_map.insert(
        (1, -1),
        Chip {
            name: "a2",
            //animal: Animal::Spider,
            team: Team::Black,
        },
    ); // up and right
    expected_map.insert(
        (1, -3),
        Chip {
            name: "a3",
            //animal: Animal::Spider,
            team: Team::Black,
        },
    ); // up from that

    // Stuff into BTreeMap to sort
    let expected_tree: BTreeMap<(i8, i8), Chip> =
        expected_map.into_iter().map(|(v, k)| (v, k)).collect();

    assert_eq!(expected_tree, dheight_tree);
}

#[test]
fn test_parseout() {
    //put down lots of chips in Cube co-ords
    let mut board = Board::test_board(Cube);
    pmoore::try_move(&mut board, "a1", Team::Black, (0, 0, 0)); // centre
    pmoore::try_move(&mut board, "a1", Team::White, (-1, 1, 0)); // down and left
    pmoore::try_move(&mut board, "a2", Team::Black, (1, -1, 0)); // up and right

    // Size of the board
    let size = 3;

    // Draw the board
    let print_string = draw::show_board(&board, size);

    // run cargo test -- --nocapture to see if the board looks correct
    println!("{print_string}");
}
