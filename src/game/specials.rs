/// Module for rules that govern the special moves of the pillbug and mosquito
use super::{board::Board, movestatus::MoveStatus};
use crate::game::comps::Chip;
use crate::maths::coord::Coord;
use std::collections::HashSet;

/// This checks if a pillbug can sumo another chip (move adjacent chip to an adjacent empty hex)
/// If it can, it will execute the move and return MoveStatus::Success.
pub fn pillbug_sumo<T: Coord>(
    board: &mut Board<T>,
    source: T,   // place to grab the sumo-ee from
    dest: T,     // place to sumo to
    position: T, // position of pillbug (sumo-er)
) -> MoveStatus {
    let the_pillbug = board.get_chip(position);
    let sumoee = board.get_chip(source);

    // If the pillbug or sumo-ee moved within last two turns, we can't sumo
    let recent_movers = board.history.last_two_turns(board.turns);

    // if the sumoer is a mosquito, change its name to m1 for the purposes of the recent move check
    let sumoer = match the_pillbug.unwrap().name.contains('m') {
        true => Some(Chip::new("m1", the_pillbug.unwrap().team)),
        false => the_pillbug,
    };

    // Prioritise returning the pillbug if both moved
    if recent_movers.contains(&sumoer) {
        return MoveStatus::RecentMove(sumoer.unwrap());
    }
    if recent_movers.contains(&sumoee) {
        return MoveStatus::RecentMove(sumoee.unwrap());
    }

    // Ensure source and destination hexes both neighbour the pillbug on layer 0
    let neighbours = board.coord.neighbours_layer0(position);

    // If they're not neighbours on layer 0, the pillbug can't sumo
    if !neighbours.contains(&source) || !neighbours.contains(&dest) {
        return MoveStatus::NotNeighbour;
    }

    // Check we can move the neighbour by checking the basic constraints of its move (e.g. hive breaks)
    let basic_constraints = board.basic_constraints(dest, source);

    // Check if we're sumoing through a beetle gate
    if sumo_beetle_gate(board, source, dest, position) {
        return MoveStatus::BeetleGate;
    } else if basic_constraints == MoveStatus::Success {
        // If all is fine, go ahead and execute the move
        board.update(sumoee.unwrap(), dest);
    }
    basic_constraints
}

/// Suck power from location suckfrom using mosquito at location position
pub fn mosquito_suck<T: Coord>(
    board: &mut Board<T>,
    suckfrom: T, // place to grab the power from
    position: T, // position of mosquito
) -> Option<&'static str> {
    // Get the mosquito who is sucking
    let mosquito = board.get_chip(position).unwrap();

    // Make sure the victim is a neighbour
    let neighbours = board.coord.neighbours_layer0(position);
    assert!(neighbours.contains(&suckfrom));

    // Get the victim chip
    let victim = match board.get_chip(suckfrom) {
        Some(value) => value,
        None => panic!("There's no chip here to suck from!"),
    };

    // Otherwise, rename the mosquito from m1 to m followed by the char of the victim
    match mosquito.morph(victim) {
        Some(morphed_mosquito) => {
            board.chips.remove(&mosquito);
            board.chips.insert(morphed_mosquito, Some(position));
            Some(morphed_mosquito.name)
        }
        None => None, // return none if we tried to morph into another mosquito
    }
}

/// Return all mosquitos in layer 0 to normal
pub fn mosquito_desuck<T: Coord>(board: &mut Board<T>) {
    // get the position of both mosquitos

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

                // if it's in layer 0
                //if position.get_layer() == 0 {
                // Overwrite the chip's name in the board's HashMap
                board.chips.remove(&chip);

                let newchip = chip.demosquito();

                board.chips.insert(newchip, Some(position));
            }

            //}
            None => (),
        }
    }
}
/// Doesn't happen often, but there's an obscure rule that a pillbug cannot sumo
/// through a beetle gate on the layer above, so this will check for the presence
/// of a beetle gate when sumoing from source to dest
fn sumo_beetle_gate<T: Coord>(
    board: &mut Board<T>,
    source: T,   // place to grab the sumo-ee from
    dest: T,     // place to sumo to
    position: T, // position of pillbug (sumo-er)
) -> bool {
    // Get the neighbouring chips one layer above the pillbug, the source and the dest
    // These will all be on layer 1 because of previous checks
    // There may be higher beetle gates, but these would require a beetle gate on layer 1

    let source_layer1_neighbours = checky(board, source);
    let position_layer1_neighbours = checky(board, position);
    let dest_layer1_neighbours = checky(board, dest);

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

fn checky<T: Coord>(board: &mut Board<T>, location: T) -> HashSet<T> {
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
