// History keeps track of all moves in a game using doubleheight co-ordinates
use std::collections::{BTreeMap, HashMap};
use std::fs::File;
use std::io::{prelude::*, BufReader};

use super::board::Board;
use super::comps::{convert_static, Chip, Team};
use crate::maths::coord::Coord;
use crate::pmoore;

#[derive(Debug, Eq, PartialEq)]
pub struct History {
    events: HashMap<u32, (Chip, (i8, i8))>, // key = turn-number, value = (chip, location)
}

impl Default for History {
    fn default() -> Self {
        Self::new()
    }
}

impl History {
    // Create new empty history
    pub fn new() -> Self {
        History {
            events: HashMap::new(),
        }
    }

    // Add a record of where a chip moved on a given turn (history doesn't record why the chip moved)
    pub fn add_event(&mut self, turn: u32, chip: Chip, location: (i8, i8)) {
        self.events.insert(turn, (chip, location));
    }

    // Save history to csv in saved_games directory
    pub fn save(&self, filename: String) -> std::io::Result<()> {
        let mut file = File::create(format!("./saved_games/{}.csv", filename))?;

        // Force the history into a BTree to order it.
        let btree_history = self
            .events
            .clone()
            .into_iter()
            .collect::<BTreeMap<u32, (Chip, (i8, i8))>>();

        // Write csv line by line
        writeln!(&mut file, "turn,team,name,row,col")?;
        for (turn, (chip, position)) in btree_history.into_iter() {
            writeln!(
                &mut file,
                "{},{:?},{},{},{}",
                turn, chip.team, chip.name, position.0, position.1
            )?;
        }
        Ok(())
    }

    // Tell me which chip moved last turn and the turn before (used by pillbug)
    pub fn last_two_turns(&self, this_turn: u32) -> [Option<Chip>; 2] {
        [
            self.which_chip(this_turn - 1),
            self.which_chip(this_turn - 2),
        ]
    }

    // Return the chip that moved on a given turn
    fn which_chip(&self, turn: u32) -> Option<Chip> {
        self.events.get(&turn).map(|(c, _)| *c)
    }
}

// Convert a history csv into a set of moves that can be emulated
fn load_moves(filename: String, test_flag: bool) -> std::io::Result<Vec<(Team, String, i8, i8)>> {
    // If we're running a test we want to load files from another directory
    let file = match test_flag {
        true => File::open(format!("./reference/tests/snapshots/{}.csv", filename))?,
        false => File::open(format!("./saved_games/{}.csv", filename))?,
    };

    let reader = BufReader::new(file);

    // A vectors for storing moves, teams and chips
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

        // Then the rest, row and col are in dheight
        let chip_name = items[2].to_string();
        let row = items[3].trim().parse::<i8>().expect("Problem parsing row");
        let col = items[4].trim().parse::<i8>().expect("Problem parsing col");

        events.push((team, chip_name, row, col));
    }
    Ok(events)
}

// Emulate the moves contained within a history csv
pub fn emulate<T: Coord>(board: &mut Board<T>, filename: String, test: bool) {
    // Load the moves as a vector from the csv
    let events = match load_moves(filename, test) {
        Ok(values) => values,
        Err(err) => panic!("Error loading history: {}", err),
    };

    // Execute each move
    for (team, chip_name, row, col) in events {
        let hex_move = board.coord.mapfrom_doubleheight((row, col)); // Map dheight to board coords
        let chip_str = convert_static(chip_name).expect("Error matching chip name, does not exist");
        pmoore::try_move(board, chip_str, team, hex_move);
    }
}
