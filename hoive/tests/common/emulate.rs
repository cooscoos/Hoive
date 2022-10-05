/// Sets up emulating a board from the contents of a saved csv
use std::fs::File;
use std::io::{prelude::*, BufReader};

use hoive::game::comps::Team;
use hoive::game::{board::Board, history::Event, specials};
use hoive::maths::coord::Coord;

/// Emulate the moves contained within a history csv of given filename
/// If test_flag == true, then csvs are loaded from ./tests/snapshots directory
pub fn emulate<T: Coord>(board: &mut Board<T>, filename: String, test_flag: bool) {
    // Load the moves as a vector from the csv
    let events = match load_moves(filename, test_flag) {
        Ok(values) => values,
        Err(err) => panic!("Error loading history: {}", err),
    };

    // Execute each move
    for event in events {
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

/// Convert a history csv of given filename into a set of moves that can be emulated.
/// If test_flag == true, then csvs are loaded from ./tests/snapshots directory.
fn load_moves(filename: String, test_flag: bool) -> std::io::Result<Vec<Option<Event>>> {
    let file = match test_flag {
        true => File::open(format!("./tests/snapshots/{}.csv", filename))?,
        false => File::open(format!("./saved_games/{}.csv", filename))?,
    };

    let reader = BufReader::new(file);

    // A vector of events for storing moves, teams and chips
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
        let event = Event::new_by_chipstring(chip_name, team, row, col);

        events.push(Some(event));
        last_turn = this_turn;
    }
    Ok(events)
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
