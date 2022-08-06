/// Module for rules that govern the special moves of the pillbug and mosquito
use super::{board::Board, movestatus::MoveStatus};
use crate::maths::coord::Coord;

/// This checks if a pillbug can sumo another chip (move adjacent chip to an adjacent empty hex)
/// If it can, it will execute the move and return MoveStatus::Success.
pub fn pillbug_sumo<T: Coord>(
    board: &mut Board<T>,
    source: &(i8, i8, i8),  // place to grab the sumo-ee from
    dest: (i8, i8, i8),     // place to sumo to
    position: (i8, i8, i8), // position of pillbug (sumo-er)
) -> MoveStatus {
    let the_pillbug = board.get_chip(position);
    let sumoee = board.get_chip(*source);

    // If the pillbug or sumo-ee moved within last two turns, we can't sumo
    let recent_movers = board.history.last_two_turns(board.turns);

    // Prioritise returning the pillbug if both moved
    if recent_movers.contains(&the_pillbug) {
        return MoveStatus::RecentMove(the_pillbug.unwrap());
    }
    if recent_movers.contains(&sumoee) {
        return MoveStatus::RecentMove(sumoee.unwrap());
    }

    // Ensure source and destination hexes both neighbour the pillbug
    let neighbours = board.coord.neighbour_tiles(position);

    if !neighbours.contains(source) || !neighbours.contains(&dest) {
        return MoveStatus::NotNeighbour;
    }

    // Check we can move the neighbour by checking the basic constraints of its move (e.g. hive breaks)
    let basic_constraints = board.basic_constraints(dest, source);

    // If all is fine, go ahead and execute the move
    if basic_constraints == MoveStatus::Success {
        board.update(sumoee.unwrap(), dest);
    }

    basic_constraints
}
