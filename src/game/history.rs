// History keeps track of all moves in doubleheight co-ordinates
// This is useful for:
// - checking recent moves for pillbug;
// - saving a list of moves to conduct a test later
// - recording a game
use std::collections::{BTreeMap, HashMap};
use std::fs::File;
use std::io::{prelude::*, BufReader};

use crate::maths::coord::Coord;
use crate::pmoore;

use super::board::Board;
use super::comps::{starting_chips, Chip, Team};

#[derive(Debug, Eq, PartialEq)]
pub struct History {
    // Hashmap of turn-number (key) with value being an enum of (chip, location)
    history_map: HashMap<u32, (Chip, (i8, i8))>,
}

impl History {
    // Return an fresh empty history
    pub fn new() -> Self {
        let history_map: HashMap<u32, (Chip, (i8, i8))> = HashMap::new();
        History { history_map }
    }

    // Add a record of the turn, the chip that moved and where it moved to
    pub fn add_record(&mut self, turn: u32, chip: Chip, location: (i8, i8)) {
        self.history_map.insert(turn, (chip, location));
    }

    // Save history hashmap to file
    pub fn save(&self, filename: String) -> std::io::Result<()> {
        let mut file = File::create(format!("./saved_games/{}.csv", filename))?;

        // Force the history into a BTree to order it...
        let btree_history = self
            .history_map
            .clone()
            .into_iter()
            .collect::<BTreeMap<u32, (Chip, (i8, i8))>>();

        // Write the lines of history into a csv
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

    // Tell me which chip moved last turn and the turn before
    pub fn prev_two(&self, this_turn: u32) -> [Option<Chip>; 2] {
        [
            self.which_chip(this_turn - 1),
            self.which_chip(this_turn - 2),
        ]
    }

    // Get the chip only that moved on a given turn
    fn which_chip(&self, turn: u32) -> Option<Chip> {
        match self.history_map.get(&turn) {
            Some((c, _)) => Some(*c),
            None => None,
        }
    }
}

// Load history up into a set of moves that can be emulated
fn load_moves(filename: String, test: bool) -> std::io::Result<Vec<(Team, String, i8, i8)>> {
    // If we're running a test we want to load files from a different folder
    let file = match test {
        true => File::open(format!("./reference/tests/snapshots/{}.csv", filename))?,
        false => File::open(format!("./saved_games/{}.csv", filename))?,
    };

    let reader = BufReader::new(file);

    // Some vectors for storing moves, teams and chips
    let mut history_list = Vec::new();

    // Read file line by line and execute the moves
    for (i, lines) in reader.lines().enumerate() {
        if i == 0 {
            continue; // skip the header
        }

        // Get the comma-separated entries on this line
        let line = lines.unwrap();
        let items = line.split(',').collect::<Vec<&str>>();

        // The first item is the turn no. The second is the team
        let team = match items[1] {
            "Black" => Team::Black,
            "White" => Team::White,
            _ => panic!("Couldn't parse team name on line {} of file", i),
        };

        // Then the rest, row and col are in dheight
        let chip_name = items[2].to_string();
        let row = items[3].trim().parse::<i8>().unwrap();
        let col = items[4].trim().parse::<i8>().unwrap();

        history_list.push((team, chip_name, row, col));
    }

    Ok(history_list)
}

pub fn emulate<T: Coord>(board: &mut Board<T>, filename: String, test: bool) {
    // Load the moves from the file
    let history_list = match load_moves(filename, test) {
        Ok(values) => values,
        Err(err) => panic!("Error loading history: {}", err),
    };

    for (team, chip_name, row, col) in history_list {
        // Map the dheight to the board's coordinate system
        let hex_move = board.coord.mapfrom_doubleheight((row, col));
        // Execute the move
        pmoore::try_move(board, convert_static(chip_name), team, hex_move);
    }
}

// Converts a chip_name (String) to a static str
fn convert_static(chip_string: String) -> &'static str {
    // Get all of the possible chip names
    let chips = starting_chips();
    let chip_names = chips
        .into_iter()
        .map(|(c, v)| c.name)
        .collect::<Vec<&str>>();

    // find the chip name that matches the chip_string and return that chip's name
    let matched = chip_names
        .into_iter()
        .filter(|n| n.to_string() == chip_string)
        .collect::<Vec<&str>>();
    matched[0]
}
