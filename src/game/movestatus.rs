use super::comps::Team;
use serde::Serialize;

/// Enum to return the result of a player action
/// MoveStatus::Selection | Meaning
///--- | ---
/// Success| Action was successfully executed
/// Win(Option<Team>)| Team won the game (None=draw)
/// Nothing| Nothing happened (used to abort action)
/// NoBee| Haven't placed bee yet so can't relocate chips
/// BeeNeed| You need to place a bee on this turn
/// NoSkip | Can't skip turn
/// Occupied| Target already occupied
/// Unconnected| Target has no neighbours
/// BadNeighbour| Target is next to opposing team
/// HiveSplit| Would split the hive if you moved
/// SmallGap| Gap too small for animal to access
/// BadDistance(u32)| Can't travel this distance, must travel u32
/// RecentMove(Chip)| Chip moved too recently to act
/// NotNeighbour| Target hex isn't a neighbour
/// BeetleBlock | A beetle on top of you is blocking your move
/// BeetleGate | A beetle gate is preventing the move
/// NoJump | Grasshopper can't make this jump
/// NoSuck | Mosquito can't do this suck
#[derive(Debug, Eq, PartialEq, Serialize)]
pub enum MoveStatus {
    Success,
    Win(Option<Team>),
    Nothing,
    NoBee,
    BeeNeed,
    NoSkip,

    Occupied,
    Unconnected,
    BadNeighbour,
    HiveSplit,

    SmallGap,
    BadDistance(u32),
    RecentMove(String),
    NotNeighbour,

    BeetleBlock,
    BeetleGate,

    NoJump,
    NoSuck,
}
