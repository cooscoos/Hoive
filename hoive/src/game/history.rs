/// The history modules keeps track of all moves in a game using doubleheight co-ordinates
use std::collections::BTreeMap;
use std::fs::File;
use std::io::{prelude::*, BufReader};
use std::str::FromStr;
use std::fmt::Error;

use super::actions::BoardAction;
use super::board::Board;
use super::comps::{convert_static, Chip, Team, get_team_from_chip, convert_static_basic};
use crate::maths::coord::{Coord, DoubleHeight};

use super::specials;

/// Every event in a game of hive is a chip_name on a given team attempting a movement
#[derive(Debug, Eq, PartialEq, Clone)]
pub struct Event {
    chip_name: &'static str,
    team: Team,
    location: DoubleHeight,
}

/// Converts events to comma separated strings
impl ToString for Event {
    fn to_string(&self) -> String {

        // Convert chip and the location to strings
        let chip_string = match self.team {
            Team::Black => self.chip_name.to_uppercase(),
            Team::White => self.chip_name.to_owned(),
        };

        let loc_string = self.location.to_string();

        // Smush them together
        format!("{}.{}",chip_string,loc_string) 
    }
}

/// Converts comma separated strings to events
impl FromStr for Event {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {

        // separate the input by full-stop. This separates the chip from the col,row
        let items = s.split('.').collect::<Vec<&str>>();

        // Get the team from whether chip is lower/upper case
        let chip_string = items[0].to_owned();
        let team = get_team_from_chip(&chip_string);

        // Convert the chip_string to lower case static
        let chip_name = convert_static_basic(chip_string.to_lowercase()).expect("Problem parsing chip string");


        // Find the location
        let colrow_str = items[1];
        let location = DoubleHeight::from_str(colrow_str).expect("Error parsing col/row into DoubleHeight");

        Ok(Event {
            chip_name,
            team,
            location,
        })

    }
}

impl Event {
    /// Create a new event based on input string
    fn new_by_string(chip_string: String, team: Team, row: i8, col: i8) -> Self {
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


    /// Create a new event based on a board action passed to server
    pub fn new_by_action(action: &BoardAction) -> Self  {
        Event{
            chip_name: action.get_chip_name(),
            team: action.which_team(),
            location: action.rowcol,
        }
    }
    

}

/// History keeps track of Events (previous player actions) using a BTree.
///
/// The key = turn number, and value = the event.
/// BTreeMap is used so that turn events are ordered.
#[derive(Debug, Default, Eq, PartialEq, Clone)]
pub struct History {
    events: BTreeMap<usize, Event>,
}

impl FromStr for History {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
     
        // Take the input and separate it by /
        let mut event_strs = s.split('/').collect::<Vec<&str>>();

        // The final one will be empty because of trailing / so delete it
        event_strs.pop();

        // Parse each event_str into a <usize, event> BTreeMap
        let events = event_strs.into_iter().enumerate().map(|(i,s)| (i, Event::from_str(s).expect("Problem parsing event str"))).collect::<BTreeMap<usize,Event>>();

        Ok(History{
            events
        })

    }
}



impl History {
    /// Create a new empty history struct.
    pub fn new() -> Self {
        History {
            events: BTreeMap::new(),
        }
    }

    /// Add a record of what location a chip moved on a given turn (history doesn't record the reason for a chip moved).
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

    /// Return which chip moved on a given turn
    /// Returns None if no chip moved that turn
    fn which_chip(&self, turn: usize) -> Option<Chip> {
        self.events
            .get(&turn)
            .map(|e| Chip::new(e.chip_name, e.team))
    }
}

/// Convert a history csv of given filename into a set of moves that can be emulated
/// If test_flag == true, then csvs are loaded from ./reference/tests/snapshots directory
fn load_moves(filename: String, test_flag: bool) -> std::io::Result<Vec<Option<Event>>> {
    // If we're running a test we want to load files from another directory
    let file = match test_flag {
        true => File::open(format!("./tests/snapshots/{}.csv", filename))?,
        false => File::open(format!("./saved_games/{}.csv", filename))?,
    };

    let reader = BufReader::new(file);

    // A vector for storing moves, teams and chips
    let mut events = Vec::new();

    // The turn number last turn
    let mut last_turn = -1;

    // Read file line by line and push the moves to the events vector
    for (i, line) in reader.lines().enumerate() {
        if i == 0 {
            continue; // skip the header of the csv
        }

        // Get comma-separated entries on this line
        let this_line = match line {
            Ok(value) => value,
            Err(err) => panic!("Could not read line {} because: {}", i, err),
        };

        let items = this_line.split(',').collect::<Vec<&str>>();

        // The item[0] is the turn number.
        let this_turn = items[0]
            .trim()
            .parse::<i16>()
            .expect("Problem parsing turn number");

        // If the turn numbers don't increase by 1, then we need to push this many Nones to the events vector
        let nones_size = this_turn - last_turn - 1;
        for _ in 0..nones_size {
            events.push(None);
        }

        // item[1] is the team.
        let team = match items[1] {
            "Black" => Team::Black,
            "White" => Team::White,
            _ => panic!("Couldn't parse team name on line {}", i),
        };

        // Now parse the rest, note that row and col are in dheight
        let chip_name = items[2].to_string();
        let row = items[3].trim().parse::<i8>().expect("Problem parsing row");
        let col = items[4].trim().parse::<i8>().expect("Problem parsing col");

        // make a new event
        let event = Event::new_by_string(chip_name, team, row, col);

        events.push(Some(event));
        last_turn = this_turn;
    }
    Ok(events)
}

/// Emulate the moves contained within a history csv of given filename
/// If test_flag == true, then csvs are loaded from ./reference/tests/snapshots directory
pub fn emulate<T: Coord>(board: &mut Board<T>, filename: String, test_flag: bool) {
    // Load the moves as a vector from the csv
    let events = match load_moves(filename, test_flag) {
        Ok(values) => values,
        Err(err) => panic!("Error loading history: {}", err),
    };

    // mosquito names

    // Execute each move
    for event in events {
        println!("{:?}", event);
        match event {
            Some(event) => {
                // If the chip name ends with an alphabetical char, we've got a mosquito which
                // needs to absorb a power from another chip before it can move.
                if event.chip_name.ends_with(|c: char| c.is_alphabetic()) {
                    emulate_mosquito(board, &event);
                }

                let hex_move = board.coord.mapfrom_doubleheight(event.location); // map movement to board coords
                board.move_chip(event.chip_name, event.team, hex_move); // execute the move

                // Refresh mosquito names back to originals
                specials::mosquito_desuck(board);
            }
            None => board.turns += 1, // skip the turn
        }
    }
}

/// Figures out where the mosquito and its victim are, and then
/// makes the mosquito absorb the power from its victim.
fn emulate_mosquito<T: Coord>(board: &mut Board<T>, event: &Event) {
    // Get the second char of the mosquito, this is its victim's first char
    let secondchar = event.chip_name.chars().nth(1).unwrap();

    // Get the position of mosquito on the current team
    let position = board.get_position_byname(event.team, "m1").unwrap();

    // Get the mosquito's neighbours
    let neighbours = board.get_neighbour_chips(position);

    // Find the neighbour that starts with second char, that's the victim
    let victim = neighbours
        .into_iter()
        .find(|c| c.name.starts_with(secondchar))
        .unwrap();

    let suck_from = board.chips.get(&victim).unwrap().unwrap();

    // Perform the suck
    specials::mosquito_suck(board, suck_from, position);
}
