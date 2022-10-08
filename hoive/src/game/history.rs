/// History keeps track of all moves in a game using Events (chip name, team + doubleheight co-ordinates)
use std::collections::BTreeMap;
use std::fmt::Error;
use std::fs::File;
use std::io::prelude::*;
use std::str::FromStr;

use super::actions::BoardAction;
use super::comps::{convert_static, convert_static_basic, get_team_from_chip, Chip, Team};

use crate::maths::coord::DoubleHeight;

/// Every event in a game of Hoive is a chip_name on a given team attempting a movement
#[derive(Debug, Eq, PartialEq, Clone)]
pub struct Event {
    pub chip_name: &'static str,
    pub team: Team,
    pub location: DoubleHeight,
}

impl ToString for Event {
    /// Converts events to comma separated strings
    fn to_string(&self) -> String {
        // If it's a "skip turn" event (chip_name = w)
        if self.chip_name == "w" {
            match self.team {
                Team::White => return "w.0,0".to_string(),
                Team::Black => return "W.0,0".to_string(),
            }
        }

        // Convert chip and the location to strings
        let chip_string = match self.team {
            Team::Black => self.chip_name.to_uppercase(),
            Team::White => self.chip_name.to_owned(),
        };
        let loc_string = self.location.to_string();

        format!("{}.{}", chip_string, loc_string)
    }
}

/// Converts a properly formatted str to an event
impl FromStr for Event {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // Separate the input by full-stop. This separates chip from col,row info
        let items = s.split('.').collect::<Vec<&str>>();

        let chip_string = items[0].to_owned();
        let team = get_team_from_chip(&chip_string);

        // Check if event is skip turn
        if chip_string.to_lowercase() == "w" {
            return Ok(Event::skip_turn(team));
        }

        let chip_name =
            convert_static_basic(chip_string.to_lowercase()).expect("Problem parsing chip string");

        // Parse coordinates
        let colrow_str = items[1];
        let location =
            DoubleHeight::from_str(colrow_str).expect("Error parsing col/row into DoubleHeight");

        Ok(Event {
            chip_name,
            team,
            location,
        })
    }
}

impl Event {
    /// Create a new event based on input chip string
    pub fn new_by_chipstring(chip_string: String, team: Team, row: i8, col: i8) -> Self {
        let location = DoubleHeight::from((row, col));

        let chip_name =
            convert_static(chip_string).expect("Error matching chip name, does not exist.");

        Event {
            chip_name,
            team,
            location,
        }
    }

    /// Create a new event based on input chip
    fn new_by_chip(chip: Chip, location: DoubleHeight) -> Self {
        Event {
            chip_name: chip.name,
            team: chip.team,
            location,
        }
    }

    /// Create a new event based on a board action
    pub fn new_by_action(action: &BoardAction) -> Self {
        Event {
            chip_name: action.get_chip_name(),
            team: action.which_team(),
            location: action.rowcol,
        }
    }

    /// Defines skip turn event for given team
    pub fn skip_turn(team: Team) -> Self {
        Event {
            chip_name: "w",
            team,
            location: DoubleHeight::default(),
        }
    }
}

/// History keeps track of Events (previous player actions) using a BTree.
/// The key = turn number, and value = the event.
/// BTreeMap is used so that turn events are ordered by turn number.
#[derive(Debug, Default, Eq, PartialEq, Clone)]
pub struct History {
    events: BTreeMap<usize, Event>,
}

impl FromStr for History {
    type Err = Error;
    /// Converts from str to history
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // Take the input and separate it by '/'
        let mut event_strs = s.split('/').collect::<Vec<&str>>();

        // Final event always empty because of a trailing '/' so remove
        event_strs.pop();

        // Parse event_strs into a BTreeMap (i.e. history)
        let events = event_strs
            .into_iter()
            .enumerate()
            .map(|(i, s)| (i, Event::from_str(s).expect("Problem parsing event str")))
            .collect::<BTreeMap<usize, Event>>();

        Ok(History { events })
    }
}

impl History {
    /// Add a record of what location a chip moved on a given turn (history doesn't record the reason for a chip move).
    pub fn add_event(&mut self, turn: usize, chip: Chip, location: DoubleHeight) {
        self.events.insert(turn, Event::new_by_chip(chip, location));
    }

    /// Save history as a csv in the local saved_games directory
    pub fn save(&self, filename: String) -> std::io::Result<()> {
        let mut file = File::create(format!("./saved_games/{}.csv", filename))?;

        // Write csv line by line
        writeln!(&mut file, "turn,team,name,row,col")?;
        for (turn, event) in self.events.iter() {
            writeln!(
                &mut file,
                "{},{:?},{},{},{}",
                turn, event.team, event.chip_name, event.location.col, event.location.row
            )?;
        }
        Ok(())
    }

    /// Returns which chips moved last turn and the turn before (used by pillbug)
    /// The return list order is: [last turn, the turn before]
    pub fn last_two_turns(&self, this_turn: usize) -> [Option<Chip>; 2] {
        [
            self.which_chip(this_turn - 1),
            self.which_chip(this_turn - 2),
        ]
    }

    /// Which chip moved on given turn? "None" if no chip moved that turn.
    fn which_chip(&self, turn: usize) -> Option<Chip> {
        self.events
            .get(&turn)
            .map(|e| Chip::new(e.chip_name, e.team))
    }
}
