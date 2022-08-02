use super::comps::{Chip, Team};

// Enum to return the status of whether a move was legal
#[derive(Debug, Eq, PartialEq)]
pub enum MoveStatus {
    Success, // The placement/relocation of the chip was legal, the move was executed
    // Following statuses are returned when move can't be executed because the target space...:
    Occupied,     // is already occupied
    Unconnected,  // is in the middle of nowhere
    BadNeighbour, // is next to opposing team
    HiveSplit,    // would split the hive in two

    // Following statuses are specific to animals / groups of animals
    SmallGap,         // gap is too small for an ant/spider/bee to access
    BadDistance(u32), // wrong distance for this animal to travel
    RecentMove(Chip), // pillbug: chip moved too recently for its special move to be executed
    NotNeighbour,     // pillbug: destination to sumo to is not a neighbouring chip

    // Following statuses are returned early game
    NoBee,   // You can't move existing chips because not placed bee yet
    BeeNeed, // You need to place your bee on this turn

    // Finally
    Win(Option<Team>), // You won the game
    Nothing,           // You did nothing this turn
}
