// Special moves that are employed by animals such as the pillbug and mosquito

use crate::maths::coord::Coord; // Coord trait applies to all hex co-ordinate systems
use super::board::{Board, MoveStatus};

// Allows the pillbug to move an adjacent chip to a neighbouring empty hex
pub fn pillbug_toss<T: Coord>(board: &mut Board<T>, source: &(i8, i8, i8), dest: (i8, i8, i8)) -> MoveStatus {


    // Check that neither we nor our neighbour moved last turn
    // To implement
    // If we can't, return enum with the right chip, prioritising pillbug as an error msg
    // return MoveStatus::RecentMove(chip);


    // Check that we can toss our neighbour by checking the basic constraints of its move
    let basic_constraints = board.basic_constraints(dest, source);

    match basic_constraints {
        MoveStatus::Success => {
            // Execute the move if all is fine. unwrapping should be fine, but check when less tired
            let chip = board.get_chip(*source).unwrap();   // Get the chip at the source
            board.chips.insert(chip, Some(dest));   // Overwrite the chip's position in the HashMap
        },
        _ => (), // otherwise do nothing
    }

    // Return the movestatus
    basic_constraints
}