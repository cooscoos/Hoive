use super::comps::Team;
use super::actions::BoardAction;
use serde::{Deserialize, Serialize};
/// Enum to return the result of a player action
/// MoveStatus::Selection | Meaning
///--- | ---
/// Success| Action was successfully executed
/// Win(Option<Team>)| Team won the game (None=draw)
/// Nothing| Nothing happened (used to abort action)
/// NoBee| Haven't placed bee yet so can't relocate chips
/// BeeNeed| You need to place a bee on this turn
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
#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Clone)]
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

    Forfeit,
    SkipTurn,
    Action(BoardAction),
}


impl ToString for MoveStatus {
    /// Convert movestatus enum to a descriptive string
    fn to_string(&self) -> String {

    match self {
        MoveStatus::Success => {
            format!("Action successful.")
        }
        MoveStatus::BadNeighbour => {
            format!("\n\x1b[31;1m<< Can't place a new chip next to other team >>\x1b[0m\n")
        }
        MoveStatus::HiveSplit => {
            format!("\n\x1b[31;1m<< No: this move would split the hive in two >>\x1b[0m\n")
        }
        MoveStatus::Occupied => {
            format!("\n\x1b[31;1m<< Can't move this chip to an occupied position >>\x1b[0m\n")
        }
        MoveStatus::Unconnected => {
            format!("\n\x1b[31;1m<< Can't move your chip to an unconnected position  >>\x1b[0m\n")
        }
        MoveStatus::SmallGap => {
            format!("\n\x1b[31;1m<< Gap too small for this piece to move into  >>\x1b[0m\n")
        }
        MoveStatus::NoSkip => {
            format!("\n\x1b[31;1m<< Can't skip turn until both bees are placed  >>\x1b[0m\n")
        }
        MoveStatus::BadDistance(value) => {
            format!("\n\x1b[31;1m<<  No: this peice must move {value} space(s)  >>\x1b[0m\n")
        }
        MoveStatus::NoBee => {
            format!("\n\x1b[31;1m<< Can't move existing chips until you've placed your bee  >>\x1b[0m\n")
        }
        MoveStatus::BeeNeed => {
            format!(
                "\n\x1b[31;1m<< It's your third turn, you must place your bee now  >>\x1b[0m\n"
            )
        }
        MoveStatus::RecentMove(chip) => {
            format!("\n\x1b[31;1m<< Can't do that this turn because chip {} moved last turn  >>\x1b[0m\n", chip)
        }
        MoveStatus::NotNeighbour => {
            format!("\n\x1b[31;1m<< That is not a neighbouring hex >>\x1b[0m\n")
        }
        MoveStatus::BeetleBlock => {
            format!(
                "\n\x1b[31;1m<< A beetle on top of you prevents you from taking action >>\x1b[0m\n"
            )
        }
        MoveStatus::BeetleGate => {
            format!("\n\x1b[31;1m<< A beetle gate prevents this move >>\x1b[0m\n")
        }
        MoveStatus::NoJump => {
            format!("\n\x1b[31;1m<< Grasshopper can't make this jump >>\x1b[0m\n")
        }
        MoveStatus::NoSuck => {
            format!("\n\x1b[31;1m<< Mosquito can't suck another mosquito >>\x1b[0m\n")
        }
        MoveStatus::Win(teamopt) => {

            match teamopt {
                Some(team) => {
                    //let team_str = draw::team_string(*team);
                    let team_str = team.to_string();
                    format!("\n << {team_str} team wins. Well done!  >> \n")
                }
                None => {
                    format!("\n << Draw. Both teams have suffered defeat! >> \n")
                }
            }
        }
        MoveStatus::Nothing | MoveStatus::Forfeit | MoveStatus::SkipTurn | MoveStatus::Action(_)=> String::new()
    }
}

}
