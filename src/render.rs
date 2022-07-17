// An ascii renderer for the Hive board


// Players will interact with the board using doubled offset co-ordinates
// see: https://www.redblobgames.com/grids/hexagons/
// This results in a grid which is likely to be more familiar to human-people.
// Offset co-ordinate systems are easy to interpret, but they're a nighmare to work maths on,
// so we'll need to map from our cube (or other) co-ordinate system.
use crate::{Chip, Animal};

// parse chips from a row into a string
pub fn parse_row(chips: Vec<Option<Chip>>, row_no: i8) -> String {

    let stringy = chips.into_iter().map(|c| chip_to_char(c)).collect::<String>();

    // create tabs as long as stringy
    let tab_iter = stringy.chars().map(|_| '\t');

    // If it's odd, put tabs in between stringy
    // interleaving ain't working...
    let returny = match row_no%2 {
        0 => tab_iter.chain(stringy.chars()).collect::<String>(),
        _ => stringy.chars().chain(tab_iter).collect::<String>(),
    };
    // put tabs in between stringy, if it's an odd row then they go between, if even they offset
    returny
}

fn chip_to_char(chip: Option<Chip>) -> &'static str {
    // Convert chip to char (if none then the char is .)
    
    let character = match chip {
        Some(value) => value.name,
        None => ".",
    };
    character
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

