// The logic that defines allowed moves for animals in the game

use std::collections::{HashMap, HashSet};

use crate::game::board::*;
use crate::maths::{coord::Coord, morphops};

pub fn ant_check<T: Coord>(
    board: &Board<T>,
    source: &(i8, i8, i8),
    dest: &(i8, i8, i8),
) -> MoveStatus {
    // Get positions of hexes that are inaccessible to ants, bees and spiders
    // Do this by morphological closing of a binary image of the board: i.e. dilation followed by erosion
    // Any new hexes generated by closing will be at locations that ants, bees, spiders can't access.

    // Get the positions of chips on the board
    let mut chip_positions = board.get_placed_positions();

    // Remove the chip at our "source" from our flat vector, we don't want it to be part of our dilation
    chip_positions.retain(|&p| p != *source);

    // Get hexes that this ant/bee/spider can't access
    let forbidden_hexes = morphops::gap_closure(&board.coord, &chip_positions);

    // Are any of those hexes equal to the desired destination?
    match forbidden_hexes.iter().any(|t| t == dest) {
        true => MoveStatus::SmallGap,
        false => MoveStatus::Success,
    }
}

pub fn bee_check<T: Coord>(
    board: &Board<T>,
    source: &(i8, i8, i8),
    dest: &(i8, i8, i8),
) -> MoveStatus {
    // Do an ant_check plus make sure dest is a neighbour of source (bee moves 1)
    // Bee move check is also used for pillbug moves as it has the same relocation rules
    match ant_check(board, source, dest) {
        MoveStatus::SmallGap => MoveStatus::SmallGap,
        MoveStatus::Success => {
            // Check if the distance is within the bee's travel range (its neighbours)
            let neighbours = board.coord.neighbour_tiles(*source);
            match neighbours.contains(dest) {
                true => MoveStatus::Success,
                false => MoveStatus::BadDistance(1),
            }
        }
        _ => unreachable!(), // this chip can't return other movestatus types
    }
}

pub fn spider_check<T: Coord>(
    board: &Board<T>,
    source: &(i8, i8, i8),
    dest: &(i8, i8, i8),
) -> MoveStatus {
    // Do an ant check and then check the spider is moving exactly 3 places (around obstacles)
    match ant_check(board, source, dest) {
        MoveStatus::SmallGap => MoveStatus::SmallGap,
        MoveStatus::Success => {
            // Get list of hexes spider can visit within 3 moves (including around obstacles)
            // let visitable = dist_lim_floodfill(board, source, 3);
            let move_rules = vec![false, false, false];
            let visitable = mod_dist_lim_floodfill(board, source, move_rules);

            // If the destination is visitable on turn 3, the move is good.
            match visitable.contains(dest) {
                true => MoveStatus::Success,
                false => MoveStatus::BadDistance(3),
            }
        }
        _ => unreachable!(), // this chip can't return other movestatus types
    }
}

pub fn ladybird_check<T: Coord>(
    board: &Board<T>,
    source: &(i8, i8, i8),
    dest: &(i8, i8, i8),
) -> MoveStatus {
    // Do an ant check and then check the lady is moving 3 places (over 2 chips and then into an empty space)
    // Rules check for later: can ladybird move over beetle on top of other chip?
    match ant_check(board, source, dest) {
        MoveStatus::SmallGap => MoveStatus::SmallGap,
        MoveStatus::Success => {
            // Get list of hexes ladybird can visit within 3 moves
            let move_rules = vec![true, true, false]; // must the space be occupied?
            let visitable = mod_dist_lim_floodfill(board, source, move_rules);

            // If destination is visitable on turn 3, the move is good.
            match visitable.contains(dest) {
                true => MoveStatus::Success,
                false => MoveStatus::BadDistance(3),
            }
        }
        _ => unreachable!(), // this chip can't return other movestatus types
    }
}

pub fn mod_dist_lim_floodfill<T: Coord>(
    board: &Board<T>,
    source: &(i8, i8, i8),
    move_rules: Vec<bool>,
) -> HashSet<(i8, i8, i8)> {
    // A modified distance-limited flood fill which can find movement ranges around and over obstacles
    // Useful for spiders and ladybirds.
    // See: https://www.redblobgames.com/grids/hexagons/#distances
    // Returns all hexes that this chip could visit on its final turn
    // Each element of the move_rules vector can be:
    // false: must move to a non-occupied position on this move (e.g. all spider turns, ladybird turn 3)
    // true: must move to an occupied position on this move (e.g. ladybird turns 1 and 2)

    // Store visitable hexes here
    let mut visitable = HashSet::new();

    // Store fringes: a list of all hexes that can be reached within k steps
    let mut fringes = HashMap::new();

    // Add the current position to fringes. It can be reached in k = 0 steps.
    fringes.insert(*source, 0);

    // Also need the position of existing chips on the board
    let obstacles = board.get_placed_positions();

    for k in 1..=move_rules.len() {
        // Get the list of hexes within fringes that have values of k-1
        let check_hexes = fringes
            .iter()
            .filter(|(_, v)| **v == k - 1)
            .map(|(p, _)| *p)
            .collect::<Vec<(i8, i8, i8)>>();

        // For each of those hexes
        for check_hex in check_hexes {
            // Get the 6 neighbours
            let neighbours = board.coord.neighbour_tiles(check_hex);

            neighbours.iter().for_each(|n| {
                // These neighbours are visitable on this turn (k) if ...
                let can_visit = match move_rules[k - 1] {
                    // (match on k-1 because of how vectors are indexed)
                    true => obstacles.contains(n), // they are blocked by an obstacle
                    false => !obstacles.contains(n), // they are not blocked by an obstacle
                };

                if can_visit & !visitable.contains(n) {
                    // don't keep overwriting values in hashset (inefficient)
                    fringes.insert(*n, k); // add the neighbour to the list of fringes for this k

                    // We only care about what hexes this peice can visit on its final turn
                    if k == move_rules.len() {
                        visitable.insert(*n);
                    }
                }
            });
        }
    }
    visitable
}
