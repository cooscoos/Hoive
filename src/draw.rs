// An ascii renderer for the Hive board
use crate::game::board::Board;
use crate::game::comps::{Chip, Team};
use crate::maths::coord::Coord;
use crate::maths::coord::DoubleHeight;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::fmt::Write as _; // import without risk of name clash

// Players will interact with the hex grid using "double-height offset co-ordinates"
// See: https://www.redblobgames.com/grids/hexagons/

// You can interact with these using familiar (x,y) or grid co-ordinate params.

// Offset co-ordinate systems are easy for human-people to interpret, but they're a nighmare to do maths with.
// We therefore need to map to and from the cube (or other) co-ordinate system that the game logic uses.

/// Parse the board out into doubleheight hex co-ordinates (a grid format more readable to humans)
pub fn to_dheight<T: Coord>(board: &Board<T>, size: i8) -> HashMap<DoubleHeight, Option<Chip>> {
    // Initialise an empty doubleheight hashmap to store chips at each co-ordinate
    let mut dheight_hashmap = empty(size);

    // Translate doubleheight co-ordinates to the current coord system being used by the board
    let board_coords = dheight_hashmap
        .iter()
        .map(|(xy, _)| board.coord.from_doubleheight(*xy))
        .collect::<HashSet<T>>();

    // Check all board_coords for chips, and put the chips in dheight_hashmap if found
    board_coords.into_iter().for_each(|p| {
        dheight_hashmap.insert(board.coord.to_doubleheight(p), board.get_chip(p));
    });

    dheight_hashmap
}

// Draw the board / table
pub fn show_board<T: Coord>(board: &Board<T>, size: i8) -> String {
    // Create dheight hashmap
    let dheight_hashmap = to_dheight(board, size);

    // pass to the parser
    parse_to_ascii(dheight_hashmap, size)
}

// Parse a doubleheight hashmap of chips into an ascii string to print board to terminal
fn parse_to_ascii(dheight_hashmap: HashMap<DoubleHeight, Option<Chip>>, size: i8) -> String {
    // Display size of ascii board should be 3, 5, 7,...
    if (size % 2 == 0) | (size == 1) {
        panic!("The size of the ascii board render must be an odd number > 1.")
    }

    let mut ascii_board = String::new();

    // Stuffing HashMaps into BTreeMaps sorts them based on the value of the key.
    // We'll switch col and row co-ordinates so that the BTree sorts by rows first
    // For now only deal with layer 0
    let mut dheight_tree: BTreeMap<(i8, i8), Option<Chip>> = dheight_hashmap
        .iter()
        .filter(|(p, _)| p.l == 0) // only do layer 0
        .map(|(p, c)| ((p.row, p.col), *c))
        .collect();

    // Now check higher layers. If we find beetles here they overwrite position input as an elevated beetle with a *, e.g. b1*
    let mut moo = HashMap::new();
    for layer in 1..=4 {
        // The beetles in this layer are here
        let beetles = dheight_hashmap
            .iter()
            .filter(|(p, c)| p.l == layer && c.is_some())
            .map(|(p, c)| (*p, *c))
            .collect::<Vec<_>>();

        // if there are some beetles, insert them in the hashmap
        if !beetles.is_empty() {
            beetles.into_iter().for_each(|(p, c)| {
                // keep a log of what they're covering to display later
                let covered = dheight_tree.get(&(p.row, p.col)).unwrap();
                let coveree = Some(c.unwrap().elevate());
                moo.insert((chip_to_str(coveree), chip_to_str(*covered)), (p.row, p.col));

                dheight_tree.insert((p.row, p.col), Some(c.unwrap().elevate()));
            });
        }
    }

    // Make a header for the ascii board
    let mut header_info = String::new();
    for col_no in -size..size + 1 {
        let mut s = String::new();
        let _ = write!(s, "{}\t", col_no);
        header_info.push_str(&s);
    }
    let header = format!("\n\nBOARD\t\t[col→]\n\n[row↓]\t\t{header_info}\n\n");
    ascii_board.push_str(&header);

    // Parse the BTree into formatted text row by row
    for row_no in -size..size + 1 {
        // Split the BTree at this row
        let remainder = dheight_tree.split_off(&(row_no, size + 1));

        // Parse this row into ascii text
        let row_string = dheight_tree
            .into_iter()
            .map(|(_, c)| format!("{}\t\t", chip_to_str(c)))
            .collect::<String>();

        // Even rows get an extra tab to offset them to make a hexagonal grid
        let push_row = match row_no % 2 {
            0 => format!("{row_no}\t\t\t{row_string}\n\n"), // even, extra tab
            _ => format!("{row_no}\t\t{row_string}\n\n"),   // odd, no extra tab
        };

        ascii_board.push_str(&push_row);

        // overwrite BTree with its remaining unparsed section ready for the next loop
        dheight_tree = remainder;
    }

    // If there are elevated beetles on the board, we need to state what they are covering
    if !moo.is_empty() {
        let mut beetle_string = String::new();
        for ((coverer, coveree), location) in moo {
            let beetle_line = format!("{} is blocking {} at {:?}\n", coverer, coveree, location);
            beetle_string.push_str(&beetle_line);
        }
        ascii_board.push_str(&beetle_string);
    }

    ascii_board
}

fn chip_to_str(chip: Option<Chip>) -> String {
    // Convert chip to a string character (if None then display as ".")

    let return_string = match chip {
        Some(value) => {
            let colour_char = match value.team {
                Team::Black => '4', // black chips coloured blue
                Team::White => '5', // white chips coloured magenta
            };
            format!("\x1b[3{}m{}\x1b[0m", colour_char, value.name) // uses hex colour for terminal
        }
        None => ".".to_string(),
    };
    return_string
}

pub fn empty(n: i8) -> HashMap<DoubleHeight, Option<Chip>> {
    // Generate an HashMap k, v, where:
    // k = chip positions in doubleheight co-ordinates
    // v = None, so the board is empty

    // Start with a HashSet for the tile positions
    let mut dheight_display = HashSet::new();

    // Generate tile positions over the range n: the size of the board
    for col in -n..n + 1 {
        for row in -n..n + 1 {
            for layer in 0..4 {
                // if both col row share oddness or evenness (this defines doubleheight coords)
                if ((row % 2 == 0) & (col % 2 == 0)) | ((row % 2 != 0) & (col % 2 != 0)) {
                    dheight_display.insert(DoubleHeight::new(col, row, layer));
                }
            }
        }
    }

    // Initialise the empty hashmap, None for all hexes
    dheight_display
        .iter()
        .map(|p| (*p, None))
        .collect::<HashMap<DoubleHeight, Option<Chip>>>()
}

// Convert team to a pretty string
pub fn team_string(team: Team) -> &'static str {
    match team {
        Team::Black => "\x1b[34;1mBlack\x1b[0m",
        Team::White => "\x1b[35;1mWhite\x1b[0m",
    }
}

// List all chips belonging to a given team that are in their hand. Return a colourful single string for display.
pub fn list_chips<T: Coord>(board: &Board<T>, team: Team) -> String {
    // Filter out the chips that are hand of given team (in hand  position = None)
    let chip_list = board
        .chips
        .clone()
        .into_iter()
        .filter(|(c, p)| (p.is_none()) & (c.team == team))
        .map(|(c, _)| chip_to_str(Some(c)))
        .collect::<Vec<String>>();

    // Convert to colourful formatted string
    make_chip_string(chip_list)
}

// List chips that you are passed and return a colourful single string for display
pub fn list_these_chips(chips_vec: Vec<Chip>) -> String {
    let chip_list = chips_vec
        .into_iter()
        .map(|c| chip_to_str(Some(c)))
        .collect::<Vec<String>>();

    make_numbered_chip_string(chip_list)
}

// Do the colourful bit
fn make_chip_string(mut chip_list: Vec<String>) -> String {
    // sort alphabetically
    chip_list.sort();

    // Create a single tring to return
    let mut chip_string = chip_list
        .iter()
        .map(|c| format!(" {},", c))
        .collect::<String>();

    // Delete the trailing comma
    chip_string.pop();

    chip_string
}

// Make a numbered list
fn make_numbered_chip_string(mut chip_list: Vec<String>) -> String {
    // sort alphabetically
    chip_list.sort();

    // Create a single tring to return
    let mut chip_string = chip_list
        .iter()
        .enumerate()
        .map(|(i, c)| format!("{}) {},", i, c))
        .collect::<String>();

    // Delete the trailing comma
    chip_string.pop();

    chip_string
}
