use serde::{Deserialize, Serialize};

/// Enum that allows us to request responses from the game's client.
/// ___
/// Req::Selection | Player would like to...
///--- | ---
/// Nothing| Do nothing (no request)
/// Select| Select a chip
/// Move| Move a selected chip
/// Mosquito | Suck power from another chip
/// Pillbug | Select the option to sumo
/// Sumo| Select a neighbour to sumo
/// SumoTo| Select a hex to sumo a victim to
/// Execute| Execute a move
/// Save| Save the game (local games only)
/// ___
/// 
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub enum Req {
    Nothing,
    Select,
    Move,
    Mosquito,
    Pillbug,
    Sumo,
    SumoTo,
    Execute,
    Save,
}

// Converts a Do command into a string. Used by websocket client.
impl ToString for Req {
    fn to_string(&self) -> String {
        use Req::*;

        match self {
            Nothing => "".to_string(),
            Select => "//cmd select".to_string(),
            Move => "//cmd moveto".to_string(),
            Mosquito => "//cmd mosquito".to_string(),
            Pillbug => "//cmd pillbug".to_string(),
            Sumo => "//cmd sumo".to_string(),
            SumoTo => "//cmd sumoto".to_string(),
            Execute => "//cmd execute".to_string(),
            Save => "".to_string(), // unimplemented
        }
    }
}
