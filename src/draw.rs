use std::collections::BTreeMap;

// An ascii renderer for the Hive board
use crate::game::comps::Team;
use std::collections::HashMap;
use std::collections::HashSet;

// Players will interact with the board using doubled offset co-ordinates
// see: https://www.redblobgames.com/grids/hexagons/
// This results in a grid which is likely to be more familiar to human-people.
// Offset co-ordinate systems are easy to interpret, but they're a nighmare to work maths on,
// so we'll need to map from our cube (or other) co-ordinate system.
use crate::game::comps::Chip;

// parse chips from a row into a string
pub fn parse_row(dheight_hashmap: HashMap<(i8, i8), Option<Chip>>, size: i8) -> String {
    // TODO: do this less crazy
    // Sanity check the board.parse_out with some tests first.

    if size % 2 == 0 {
        panic!("The size of the ascii board render must be an odd number")
    }

    // Stuffing HashMaps into BTreeMaps will sort them based on the value of the key
    // We'll switch col and row co-ordinates so that the BTree sorts by rows first
    let mut dheight_tree: BTreeMap<(i8, i8), Option<Chip>> = dheight_hashmap
        .into_iter()
        .map(|((col, row), c)| ((row, col), c))
        .collect();

    // You can surmise what the size of this dheight_tree is based on the value of the first key. It'll be (-size, -size)
    // But it's better just to pass this renderer a size to render, that way we can zoom in/out later

    // split off at (-size, + size), then (-size+1, +size), all the way to (+size, +size)

    let mut user_display = String::new();

    //

    let mut header = String::new();
    for col_no in -size..size + 1 {
        header.push_str(&format!("{col_no}\t"));
    }
    let fin_header = format!("\n\nBOARD\t\t[col→]:\n\n[row↓]\t\t{header}\n\n");

    user_display.push_str(&fin_header);
    // row_no will be our counter that increments to +size in a loop
    for row_no in -size..size + 1 {
        // Split the BTree at the row
        let remainder = dheight_tree.split_off(&(row_no, size + 1));

        // This is the parsing bit
        let row_string = dheight_tree
            .into_iter()
            .map(|(_, c)| format!("{}\t\t", chip_to_str(c)))
            .collect::<String>();

        let append_string = match row_no % 2 {
            0 => format!("{row_no}\t\t\t{row_string}\n\n"), // even, extra tab
            _ => format!("{row_no}\t\t{row_string}\n\n"),   // odd, no extra tab
        };

        user_display.push_str(&append_string);

        // overwrite dheight_tree with the remainder for the next loop
        dheight_tree = remainder;
    }

    user_display
}

fn chip_to_str(chip: Option<Chip>) -> String {
    // Convert chip to char (if none then the char is .)

    let return_str = match chip {
        Some(value) => {
            let colour_char = match value.team {
                Team::Black => '4', // black chips coloured blue
                Team::White => '5', // white chips coloured magenta
            };
            format!("\x1b[3{}m{}\x1b[0m", colour_char, value.name)
        }
        None => ".".to_string(),
    };
    return_str
}

pub fn empty(n: i8) -> HashMap<(i8, i8), Option<Chip>> {
    // Generate an HashMap k, v, where:
    // k = chip positions in doubleheight co-ordinates
    // v = None, so the board is empty

    // Start with a HashSet for the tile positions
    let mut dheight_display = HashSet::new();

    // Generate tile positions over the range n: the size of the board
    for col in -n..n + 1 {
        for row in -n..n + 1 {
            // if both col row share oddness or evenness (this defines doubleheight coords)
            if ((row % 2 == 0) & (col % 2 == 0)) | ((row % 2 != 0) & (col % 2 != 0)) {
                dheight_display.insert((col, row));
            }
        }
    }

    // Initialise the empty hashmap, None for all hexes
    dheight_display
        .iter()
        .map(|xy| (*xy, None))
        .collect::<HashMap<(i8, i8), Option<Chip>>>()
}
