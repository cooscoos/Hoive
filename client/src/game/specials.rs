/// Module for rules that govern the special moves of the pillbug and mosquito
use super::{board::Board, movestatus::MoveStatus};
use crate::game::comps::Chip;
use crate::maths::coord::Coord;
use std::collections::HashSet;


/// Doesn't happen often, but there's an obscure rule that a pillbug cannot sumo
/// through a beetle gate on the layer above, so this will check for the presence
/// of a beetle gate when sumoing from source to dest
fn sumo_beetle_gate<T: Coord>(
    board: &mut Board<T>,
    source: T,   // place to grab the sumo-ee from
    dest: T,     // place to sumo to
    position: T, // position of pillbug (sumo-er)
) -> bool {
    // Get the neighbouring chips one layer above the pillbug
    // There may be higher beetle gates, but these all at least
    // require a beetle gate on layer 1.
    let source_layer1_neighbours = find_beetle_gates(board, source);
    let position_layer1_neighbours = find_beetle_gates(board, position);
    let dest_layer1_neighbours = find_beetle_gates(board, dest);

    // If there's overlap between source and position, or position and dest, we have a beetle gate

    (position_layer1_neighbours
        .intersection(&source_layer1_neighbours)
        .count()
        == 2)
        || (position_layer1_neighbours
            .intersection(&dest_layer1_neighbours)
            .count()
            == 2)
}

/// Get the neighbouring chips one layer above the pillbug, at the source (where we sumo from)
/// and dest (where pillbug will sumo to).
fn find_beetle_gates<T: Coord>(board: &mut Board<T>, location: T) -> HashSet<T> {
    // Get placed positions of board chips
    let placed_hexes = board.get_placed_positions();

    // The hexes neighbouring the chosen hex, but on layer 1
    let position_layer1_hexes = board
        .coord
        .neighbours_onlayer(location, location.get_layer() + 1);

    // The hexes which have chips in them
    position_layer1_hexes
        .intersection(&placed_hexes)
        .copied()
        .collect::<HashSet<T>>()
}

/// Absorb power from chip at location suck_from using mosquito at location position.
/// Doing this renames the mosquito from "m1" to m followed by the char of the victim,
/// e.g. "mb", "ma", "mq" so that it can pass board logic checks as if it were the victim.
pub fn mosquito_suck<T: Coord>(
    board: &mut Board<T>,
    suck_from: T, // place to grab the power from
    position: T,  // position of mosquito
) -> Option<&'static str> {
    // Get the sucker
    let mosquito = board.get_chip(position).unwrap();

    // Make sure the victim is a neighbour, it should be because pmoore does
    // a cracking job.
    let neighbours = board.coord.neighbours_layer0(position);
    assert!(neighbours.contains(&suck_from));

    // Get the victim chip
    let victim = match board.get_chip(suck_from) {
        Some(value) => value,
        None => panic!("There's no chip here to suck from!"),
    };

    // Rename the mosquito from m1 to m followed by the char of the victim
    match mosquito.morph(victim) {
        Some(morphed_mosquito) => {
            board.chips.remove(&mosquito);
            board.chips.insert(morphed_mosquito, Some(position));
            Some(morphed_mosquito.name)
        }
        None => None, // return none if we tried to morph into another mosquito
    }
}

/// Reset all mosquitos on the board so they go back to being called "m1".
pub fn mosquito_desuck<T: Coord>(board: &mut Board<T>) {
    // get the position of the mosquitos on the board

    let positions = board
        .chips
        .iter()
        .filter(|(c, _)| c.name.contains('m'))
        .map(|(_, p)| *p)
        .collect::<Vec<Option<T>>>();

    for positiony in positions {
        match positiony {
            Some(position) => {
                // Get mosquito
                let chip = board.get_chip(position).unwrap();

                // Overwrite the chip's name in the board's HashMap
                board.chips.remove(&chip);
                let mosquito = chip.demosquito();
                board.chips.insert(mosquito, Some(position));
            }

            None => (), // There weren't any mosquitos on the board to reset
        }
    }
}
