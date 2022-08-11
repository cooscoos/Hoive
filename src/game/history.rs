/// The history modules keeps track of all moves in a game using doubleheight co-ordinates
use std::collections::BTreeMap;
use std::fs::File;
use std::io::{prelude::*, BufReader};

use super::board::Board;
use super::comps::{convert_static, Chip, Team};
use crate::maths::coord::{Coord, DoubleHeight};

/// Struct to keep track of events (previous player actions).
///
/// Events are a BTreeMap where the key = turn number, and value = (chip,location).
/// BTreeMap is used so that turn events are ordered.
#[derive(Debug, Eq, PartialEq)]
pub struct History {
    events: BTreeMap<u32, (Chip, DoubleHeight)>,
}

impl Default for History {
    fn default() -> Self {
        Self::new()
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
    pub fn add_event(&mut self, turn: u32, chip: Chip, location: DoubleHeight) {
        self.events.insert(turn, (chip, location));
    }

    /// Save history as a csv in the local saved_games directory
    pub fn save(&self, filename: String) -> std::io::Result<()> {
        let mut file = File::create(format!("./saved_games/{}.csv", filename))?;

        // Write csv line by line
        writeln!(&mut file, "turn,team,name,row,col")?;
        for (turn, (chip, position)) in self.events.iter() {
            writeln!(
                &mut file,
                "{},{:?},{},{},{}",
                turn, chip.team, chip.name, position.col, position.row
            )?;
        }
        Ok(())
    }

    /// Returns which chips moved last turn and the turn before (used by pillbug)
    /// The return list order is: [last turn, the turn before]
    pub fn last_two_turns(&self, this_turn: u32) -> [Option<Chip>; 2] {
        [
            self.which_chip(this_turn - 1),
            self.which_chip(this_turn - 2),
        ]
    }

    /// Return which chip moved on a given turn
    /// Returns None if no chip moved that turn
    fn which_chip(&self, turn: u32) -> Option<Chip> {
        self.events.get(&turn).map(|(c, _)| *c)
    }
}

/// Convert a history csv of given filename into a set of moves that can be emulated
/// If test_flag == true, then csvs are loaded from ./reference/tests/snapshots directory
fn load_moves(filename: String, test_flag: bool) -> std::io::Result<Vec<(Team, String, i8, i8)>> {
    // If we're running a test we want to load files from another directory
    let file = match test_flag {
        true => File::open(format!("./reference/tests/snapshots/{}.csv", filename))?,
        false => File::open(format!("./saved_games/{}.csv", filename))?,
    };

    let reader = BufReader::new(file);

    // A vector for storing moves, teams and chips
    let mut events = Vec::new();

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

        // The item[0] is the turn number (ignore), item[1] is the team.
        let team = match items[1] {
            "Black" => Team::Black,
            "White" => Team::White,
            _ => panic!("Couldn't parse team name on line {}", i),
        };

        // Now parse the rest, note that row and col are in dheight
        let chip_name = items[2].to_string();
        let row = items[3].trim().parse::<i8>().expect("Problem parsing row");
        let col = items[4].trim().parse::<i8>().expect("Problem parsing col");

        events.push((team, chip_name, row, col));
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

    // Execute each move
    for (team, chip_name, row, col) in events {
        let hex_move = board
            .coord
            .mapfrom_doubleheight(DoubleHeight::from((row, col))); // Map dheight to board coords
        let chip_str = convert_static(chip_name).expect("Error matching chip name, does not exist");
        board.move_chip(chip_str, team, hex_move);
    }
}
