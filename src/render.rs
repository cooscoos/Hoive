use std::collections::BTreeMap;
// An ascii renderer for the Hive board
use std::collections::HashMap;
use std::collections::HashSet;

// Players will interact with the board using doubled offset co-ordinates
// see: https://www.redblobgames.com/grids/hexagons/
// This results in a grid which is likely to be more familiar to human-people.
// Offset co-ordinate systems are easy to interpret, but they're a nighmare to work maths on,
// so we'll need to map from our cube (or other) co-ordinate system.
use crate::{Animal, Chip};

// parse chips from a row into a string
pub fn parse_row(dheight_hashmap: HashMap<(i8, i8), Option<Chip>>) -> String {
    
    
    // TODO: do this less crazy
    // Sanity check the board.parse_out with some tests first.
    let mut user_display = String::new();

    // get all the chips for a row_no

    // need a btreemap

    // This a mental way of doing this but it probably works..

    for ((_, the_row), _) in dheight_hashmap.clone() {


        // make a btree for this row.
        // use a btree because they're ordered
        let dheight_btree = dheight_hashmap.clone()
            .into_iter()
            .filter(|((col_no, row_no), _)| *row_no == the_row)
            .map(|((col_no, row_no), c)| (col_no,c))
            .collect::<BTreeMap<i8,Option<Chip>>>();
        
        // Odd or even decides if tab is first
        let row_string = match the_row % 2 {
            0 => dheight_btree
                .into_iter()
                .map(|(_,c)| format!("\t{}", chip_to_str(c)))
                .collect::<String>(),
            _ => dheight_btree
                .into_iter()
                .map(|(_,c)| format!("{}\t", chip_to_str(c)))
                .collect::<String>(),
        };

        let append_string = format!("{}\n\n\n", row_string);

        user_display.push_str(&append_string);
        
    
    }
    

    user_display
}

fn chip_to_str(chip: Option<Chip>) -> &'static str {
    // Convert chip to char (if none then the char is .)

    let return_str = match chip {
        Some(value) => value.name,
        None => ".",
    };
    return_str
}

pub fn generate(n: i8) -> HashSet<(i8, i8)> {
    // Generate a hashnet of doubleheight values to display
    let mut dheight_display = HashSet::new();

    // These 5s can be changed to a variable later which depends on board extremes
    for col in -n..n + 1 {
        for row in -n..n + 1 {
            dheight_display.insert((col, row));
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
