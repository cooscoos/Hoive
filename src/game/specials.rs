// Special moves that are employed by animals such as the pillbug and mosquito

use std::collections::HashSet;

use super::board::{Board, MoveStatus};
use crate::maths::coord::Coord; // Coord trait applies to all hex co-ordinate systems

// Allows the pillbug to move an adjacent chip to a neighbouring empty hex
pub fn pillbug_toss<T: Coord>(
    board: &mut Board<T>,
    source: &(i8, i8, i8),// place to sumo from
    dest: (i8, i8, i8),   // place to sumo to
    position: (i8,i8,i8), // position of the pillbug
) -> MoveStatus {
    // Check that neither we nor our neighbour moved last turn.
    // To implement
    // If we can't, return enum with the problem chip, prioritising the pillbug as an error msg if both moved 
    // return MoveStatus::RecentMove(chip);
    let recent_movers = board.history.prev_two(board.turns);

    println!("Recent movers: {:?}",recent_movers);

    let sumoer =board.get_chip(position); // the sumo-er
    let sumoee = board.get_chip(*source); // the sumo-ee

    println!("Sumoer: {:?}, Sumoee: {:?}",sumoer,sumoee);

    println!("Recent movers contains sumoer?: {}", recent_movers.contains(&sumoer));

    // check the pillbug or its victim have moved within last two turns
    if recent_movers.contains(&sumoer) {
        println!("Returning");
        return MoveStatus::RecentMove(sumoer.unwrap());
    }
    if recent_movers.contains(&sumoee) {
        return MoveStatus::RecentMove(sumoee.unwrap());
    }

    // Check that the source and destination both neighbour the pillbug. If they don't return error
    let neighbours = board.coord.neighbour_tiles(position).into_iter().collect::<HashSet<_>>();
    if !neighbours.contains(source) || !neighbours.contains(&dest) {
        return MoveStatus::NotNeighbour;
    }

    // Finally, check that we can toss our neighbour by checking the basic constraints of its move
    let basic_constraints = board.basic_constraints(dest, source);

    match basic_constraints {
        MoveStatus::Success => {
            // Execute the move if all is fine. unwrapping should be fine, but check when less tired
            let chip = board.get_chip(*source).unwrap(); // Get the chip at the source

            // Overwrite the chip's position in the HashMap and update history
            board.update(chip, dest);

        }
        _ => (), // otherwise do nothing
    }

    // Return the movestatus
    basic_constraints
}
