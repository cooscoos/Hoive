// An ascii renderer for the Hive board

// Players will interact with the board using doubled offset co-ordinates
// see: https://www.redblobgames.com/grids/hexagons/
// This results in a grid which is likely to be more familiar to human-people.
// Offset co-ordinate systems are easy to interpret, but they're a nighmare to work maths on,
// so we'll need to map from our cube (or other) co-ordinate system.
use crate::{Animal, Chip};

// parse chips from a row into a string
pub fn parse_row(chips: Vec<Option<Chip>>, row_no: i8) -> String {
    // Odd or even decides if tab is first
    let row_string = match row_no % 2 {
        0 => chips
            .into_iter()
            .map(|c| format!("\t{}", chip_to_str(c)))
            .collect::<String>(),
        _ => chips
            .into_iter()
            .map(|c| format!("{}\t", chip_to_str(c)))
            .collect::<String>(),
    };

    row_string
}

fn chip_to_str(chip: Option<Chip>) -> &'static str {
    // Convert chip to char (if none then the char is .)

    let return_str = match chip {
        Some(value) => value.name,
        None => ".",
    };
    return_str
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
