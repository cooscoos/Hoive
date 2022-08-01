// History keeps track of all moves in doubleheight co-ordinates
// This is useful for:
// - checking recent moves for pillbug;
// - saving a list of moves to conduct a test later
// - recording a game
use std::collections::HashMap;
use super::comps::Chip;

pub struct History {
    // Hashmap of turn-number (key) with value being an enum of (chip, location)
    history_map: HashMap<u32, (Chip,(i8,i8))>
}

impl History {

    // Return an fresh empty history
    pub fn new() -> Self {
        let history_map: HashMap<u32, (Chip,(i8,i8))> = HashMap::new();
        History{history_map}
    }

    // Add a record of the turn, the chip that moved and where it moved to
    pub fn add_record(&mut self, turn: u32, chip: Chip, location: (i8,i8)) {
        self.history_map.insert(turn, (chip, location));
    }


    // Tell me which chip moved last turn and the turn before
    pub fn prev_two(&self, this_turn: u32) -> [Option<Chip>;2] {
        [self.which_chip(this_turn-1), self.which_chip(this_turn-2)]
    }

    // Get the chip only that moved on a given turn
    fn which_chip(&self, turn: u32) -> Option<Chip> {
        match self.history_map.get(&turn) {
            Some((c,_)) => Some(*c),
            None => None,
        }
    }
}