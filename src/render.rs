use std::collections::BTreeMap;
use std::ops::ControlFlow;
// An ascii renderer for the Hive board
use crate::Team;
use std::collections::HashMap;
use std::collections::HashSet;

// Players will interact with the board using doubled offset co-ordinates
// see: https://www.redblobgames.com/grids/hexagons/
// This results in a grid which is likely to be more familiar to human-people.
// Offset co-ordinate systems are easy to interpret, but they're a nighmare to work maths on,
// so we'll need to map from our cube (or other) co-ordinate system.
use crate::{Animal, Chip};

// parse chips from a row into a string
pub fn parse_row(dheight_hashmap: HashMap<(i8, i8), Option<Chip>>, size: i8) -> String {
    // TODO: do this less crazy
    // Sanity check the board.parse_out with some tests first.

    if size%2 == 0{
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
    for col_no in -size..size+1{
        header.push_str(&format!("{col_no}\t"));
    }
    let fin_header = format!("\n\nBOARD\t\t[col→]:\n\n[row↓]\t\t{header}\n\n");


    user_display.push_str(&fin_header);
    // row_no will be our counter that increments to +size in a loop
    for row_no in -size..size + 1 {
        // Split the BTree at the row
        let remainder = dheight_tree.split_off(&(row_no, size + 1));

        // do the parse on dheight_tree

        // This is the parsing bit
        // Odd or even decides if tab is first
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
            let prechar = match value.team {
                Team::Black => 'b',
                Team::White => 'w',
            };
            format!("{}{}", prechar, value.name)
        }
        None => ".".to_string(),
    };
    return_str
}

pub fn generate(n: i8) -> HashSet<(i8, i8)> {
    // Generate a hashnet of doubleheight values to display
    let mut dheight_display = HashSet::new();


    for col in -n..n + 1 {
        for row in -n..n + 1 {
            // if both col row share oddness or evenness
            if ((row%2 == 0) & (col%2 ==0)) | ((row%2 != 0) & (col%2 !=0)) {
                dheight_display.insert((col, row));
            }
        }
    }
    dheight_display
}

pub fn empty() -> &'static str {
    let label = "-5\t-4\t-3\t-2\t-1\t0\t1\t2\t3\t4\t5";

    // 26 tiles in a game of hive, so the highest or widest the board will ever be is 26
    // The board is very likely to be confined to 11 x 11, so we'll make a vector of 11 entries

    //empty_odd = ".\t.\t.\t.\t.\t.";
    //empty_even="\t.\t.\t.\t.\t.";

    "
   -5\t-4\t-3\t-2\t-1\t0\t1\t2\t3\t4\t5

-5  .		.		.		.		.		.



-4      .		.		.		.		.



-3  .		.		.		.		.		.



-2      .		.		.		.		.



-1  .		.		.		.		.		.



0	    .		.		o		.		.



1   .		.		.		.		.		.



2       .		.		.		.		.



3   .		.		.		.		.		.



4       .		.		.		.		.



5   .		.		.		.		.		.

"
}

// todo: render an ant in space 0,0
