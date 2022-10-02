use super::actions::BoardAction;
use super::comps::Team;
use serde::{Deserialize, Serialize};
/// Enum to return the attempt or result of a player action.
/// These can be results or attempts, described below.
/// ___
/// MoveStatus::Selection | Result
///--- | ---
/// Success| Action was successfully executed
/// Win(Option<Team>)| Team won the game (None=draw)
/// Nothing| Nothing happened (used to abort action)
/// NoBee| Haven't placed bee yet so can't relocate chips
/// BeeNeed| You need to place a bee on this turn
/// NoSkip | You can't skip your turn yet
/// NoSpecial | Requested sumo from non pillbug
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
/// ___
/// MoveStatus::Selection | Attempt
///--- | ---
/// Forfeit | Player requested forfeit
/// SkipTurn| Player requested skip turn
/// Action | Player Requested action
#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Clone)]
pub enum MoveStatus {
    // Results of actions
    Success,
    Win(Option<Team>),
    Nothing,
    NoBee,
    BeeNeed,

    NoSkip,
    NoSpecial,

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

    // Attempted actions
    Forfeit,
    SkipTurn,
    Action(BoardAction),
}

impl ToString for MoveStatus {
    /// Convert movestatus enum to a descriptive string
    fn to_string(&self) -> String {
        match self {
            MoveStatus::Success => "Action successful.".to_string(),
            MoveStatus::BadNeighbour => {
                "\n\x1b[31;1m<< Can't place a new chip next to other team >>\x1b[0m\n".to_string()
            }
            MoveStatus::HiveSplit => {
                "\n\x1b[31;1m<< No: this move would split the hive in two >>\x1b[0m\n".to_string()
            }
            MoveStatus::Occupied => {
                "\n\x1b[31;1m<< Can't move this chip to an occupied position >>\x1b[0m\n"
                    .to_string()
            }
            MoveStatus::Unconnected => {
                "\n\x1b[31;1m<< Can't move your chip to an unconnected position  >>\x1b[0m\n"
                    .to_string()
            }
            MoveStatus::SmallGap => {
                "\n\x1b[31;1m<< Gap too small for this piece to move into  >>\x1b[0m\n".to_string()
            }
            MoveStatus::NoSkip => {
                "\n\x1b[31;1m<< Can't skip turn until both bees are placed  >>\x1b[0m\n".to_string()
            }
            MoveStatus::NoSpecial => {
                "\n\x1b[31;1m<< This chip doesn't have special moves >>\x1b[0m\n".to_string()
            }
            MoveStatus::BadDistance(value) => {
                format!("\n\x1b[31;1m<<  No: this peice must move {value} space(s)  >>\x1b[0m\n")
            }
            MoveStatus::NoBee => {
                "\n\x1b[31;1m<< Can't move existing chips until you've placed your bee  >>\x1b[0m\n"
                    .to_string()
            }
            MoveStatus::BeeNeed => {
                "\n\x1b[31;1m<< It's your third turn, you must place your bee now  >>\x1b[0m\n"
                    .to_string()
            }
            MoveStatus::RecentMove(chip) => {
                format!("\n\x1b[31;1m<< Can't do that this turn because chip {} moved last turn  >>\x1b[0m\n", chip)
            }
            MoveStatus::NotNeighbour => {
                "\n\x1b[31;1m<< That is not a neighbouring hex >>\x1b[0m\n".to_string()
            }
            MoveStatus::BeetleBlock => {
                "\n\x1b[31;1m<< A beetle on top of you prevents you from taking action >>\x1b[0m\n"
                    .to_string()
            }
            MoveStatus::BeetleGate => {
                "\n\x1b[31;1m<< A beetle gate prevents this move >>\x1b[0m\n".to_string()
            }
            MoveStatus::NoJump => {
                "\n\x1b[31;1m<< Grasshopper can't make this jump >>\x1b[0m\n".to_string()
            }
            MoveStatus::NoSuck => {
                "\n\x1b[31;1m<< Mosquito can't suck another mosquito >>\x1b[0m\n".to_string()
            }
            MoveStatus::Win(teamopt) => {
                match teamopt {
                    Some(team) => {
                        //let team_str = draw::team_string(*team);
                        let team_str = team.to_string();
                        format!("\n << {team_str} team wins. Well done!  >> \n")
                    }
                    None => "\n << Draw. Both teams have suffered defeat! >> \n".to_string(),
                }
            }
            MoveStatus::Nothing
            | MoveStatus::Forfeit
            | MoveStatus::SkipTurn
            | MoveStatus::Action(_) => String::new(),
        }
    }
}
