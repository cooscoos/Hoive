/// Board module tracks the chips and executes their moves
use std::collections::{BTreeMap, HashMap, HashSet};
use std::fmt::Write as _;

use crate::maths::funcs;

use super::comps::{self, Chip, Team}; // Game components (chips, teams)
use crate::game::{animals, history::History, movestatus::MoveStatus}; // Animal logic, move tracking and history
use crate::maths::coord::Coord; // Hexagonal coordinate system
use crate::maths::coord::Spiral;

/// The Board struct keeps track of game's progress, history and execution of rules
#[derive(Debug, Eq, PartialEq, Clone)]
pub struct Board<T: Coord> {
    pub chips: HashMap<Chip, Option<T>>, // player chips (both teams)
    pub turns: usize,                    // number of turns that have elapsed
    pub coord: T,                        // coordinate sytem for the board e.g. Cube, HECS
    pub history: History,                // record of all previous moves
    pub size: i8,                        // the size of the board in dheight
}

impl<T> Board<T>
where
    T: Coord,
{
    /// Initialises a new board with a given coordinate system.
    pub fn new(coord: T) -> Self {
        // Chips for each team initialised in players' hands (position == None)
        let chips = comps::starting_chips();
        let history = History::default(); // Blank history

        Board {
            chips,
            turns: 0,
            coord,
            history,
            size: 5,
        }
    }

    /// Execute the move of chip to destination, update the board's history and increment turn number.
    pub fn update(&mut self, chip: Chip, dest: T) {
        // Overwrite the chip's position in the board's HashMap
        self.chips.insert(chip, Some(dest));

        // update the size of the board
        self.size = self.find_size();

        // Update history (in dheight coords)
        self.history
            .add_event(self.turns, chip, self.coord.to_doubleheight(dest));

        // Increment turns by 1
        self.turns += 1;
    }

    /// Finds the size of the board based on the chips that are placed at the extremeties.
    /// Will return an odd number >= 5. Value is used by ascii renderer to draw a sensibly sized board
    fn find_size(&self) -> i8 {
        // check the board extremeties to define the size
        let chip_positions = self.get_placed_positions();

        // find the biggest row and col placement of a chip in doubleheight
        let max_col = chip_positions
            .iter()
            .map(|d| self.coord.to_doubleheight(*d).col.abs())
            .max()
            .unwrap();
        let max_row = chip_positions
            .iter()
            .map(|d| self.coord.to_doubleheight(*d).row.abs())
            .max()
            .unwrap();

        // let the biggest of row or col define the board size
        // we multiply col (x) by 2 because we're using doubleheight coords, so
        // each 1 step horizontal is 2 steps vertical. Multiplying cols by 2
        // gives the cols and rows equivalent scaling.
        let max_rowcol = [max_row, max_col * 2];
        let biggest = max_rowcol.iter().max().unwrap();

        // The size of the board should be an odd number >= 5
        let mut size = (biggest - (biggest % 2)) + 3;

        if size <= 5 {
            size = 5;
        }

        size
    }

    /// Try move a chip, of given name and team, to a new position.
    /// Returned MoveStatus tells the caller how successful the attempt was.
    pub fn move_chip(&mut self, name: &'static str, team: Team, position: T) -> MoveStatus {
        let chip_select = Chip::new(name, team); // Select the chip

        // A chip's current position tells us if we're "placing" from a player's hand, or "relocating" on the board
        let move_status = match self.chips.clone().get(&chip_select) {
            Some(p) => {
                match p {
                    // chip already has a position, so we're relocating
                    Some(source) => self.relocate_chip(chip_select, position, *source),
                    // chip's current position == None (player hand), so we're placing
                    None => self.place_chip(chip_select, position),
                }
            }
            None => {
                panic!("Something went wrong in game logic. Chip can't be moved because it isn't defined.")
            }
        };

        move_status
    }

    /// Move chip from player's hand to the board at position == dest
    /// Check following constraints:
    /// 1) player must have placed the bee by their turn 4 (board turns 6 and 7)
    /// 2) can't place on top of another chip;
    /// 3) must have at least one neighbour (ater turn 1);
    /// 4) neighbours must be on the same team (after turn 2).
    fn place_chip(&mut self, chip: Chip, dest: T) -> MoveStatus {
        // Check if a bee has been placed by player turn 4
        if (self.turns == 6) | (self.turns == 7)
            && !self.bee_placed(chip.team) & (chip.name != "q1")
        {
            // Player hasn't placed bee yet and isn't trying to
            return MoveStatus::BeeNeed;
        }

        if self.get_chip(dest).is_some() {
            // // Any chips already on board at dest?
            MoveStatus::Occupied
        } else if self.turns > 0 && self.count_neighbours(dest) == 0 {
            // Is there at least one chip neighbouring dest after turn 0?
            MoveStatus::Unconnected
        } else if self.turns > 1 && self.unfriendly_neighbours(dest, chip.team) {
            // Are any neighbours not on my team after turn 1
            MoveStatus::BadNeighbour
        } else {
            // No problem: execute the move.
            self.update(chip, dest);
            MoveStatus::Success
        }
    }

    /// Checks if there are any neighbours on opposing teams next to the chosen destination
    fn unfriendly_neighbours(&self, dest: T, my_team: Team) -> bool {
        self.get_neighbour_chips(dest)
            .iter()
            .any(|c| c.team != my_team)
    }

    /// Relocate a chip on the board at source to dest checking constraints:
    /// 1) player must have placed their bee (only need to check prior to board turn 6)
    /// 2) check other basic_constraints for moves
    /// 3) animal-specific constraints
    fn relocate_chip(&mut self, chip: Chip, dest: T, source: T) -> MoveStatus {
        // Team can't relocate chips if they haven't placed bee.
        if self.turns <= 5 && !self.bee_placed(chip.team) {
            return MoveStatus::NoBee;
        }

        // Is the chip is a beetle (or a mosquito imitating one)?
        // Mosquitos on layer > 0 are still called "m1" but need to be classed as
        // beetles while the roam on top of the hive.
        let is_beetle = chip.name.contains('b')
            || self
                .get_position_byname(chip.team, chip.name)
                .unwrap()
                .get_layer()
                > 0;

        // Allow the chip to switch layers if it's a beetle
        let destin = match is_beetle {
            true => animals::layer_adjust(self, dest),
            false => dest,
        };

        // Check animal-specific constraints of the move
        let animal_rules = self.animal_constraint(chip, &source, &destin);

        // Check basic constraints, checked during all relocations on board
        let basic_constraints = self.basic_constraints(destin, source);

        if basic_constraints != MoveStatus::Success {
            basic_constraints
        } else if animal_rules != MoveStatus::Success {
            animal_rules
        } else {
            // No problem, execute the move
            self.update(chip, destin);
            // Relocation of chip could result in the game end
            self.check_win_state(chip.team) // Returns MoveStatus::Success if nobody won (game continues)
        }
    }

    /// Check if a team has placed their bee
    pub fn bee_placed(&self, team: Team) -> bool {
        self.get_placed_chips(team).iter().any(|c| c.name == "q1")
    }

    /// Basic constraints checked during all relocations, including pillbug sumos.
    /// A move from source to dest should not cause us to:
    /// 1) end turn on top of another chip (worry about beetle later);
    /// 2) have no neighbours, or;
    /// 3) split the hive.
    pub fn basic_constraints(&mut self, dest: T, source: T) -> MoveStatus {
        // check constraints in this order because they're not all mutally exclusive and we want to return useful errors to users

        if self.get_chip(dest).is_some() {
            // Do we end up on top of another chip? This won't fail if beetle because they are already ascended
            MoveStatus::Occupied
        } else if self.count_neighbours(dest) == 0 {
            // Do we have end up adjacent to no other tiles?
            // This won't fail even if beetle because count_neighbours always counts neighbours in layer 0
            // And there is no situation in the game where beetles can have 0 neighbours in layer 0, becuase there
            // must always be at least one own bee + one opponent chip on the board to move the beetle.
            MoveStatus::Unconnected
        } else if self.sat_on_me(source) {
            // Check if there's a beetle above
            MoveStatus::BeetleBlock
        } else if self.hive_break_check(&source, &dest) {
            // Does moving the chip split the hive?
            MoveStatus::HiveSplit
        } else {
            MoveStatus::Success
        }
    }

    /// Check if moving a chip from source to dest splits the hive
    /// This function will likely cause test failure when we introduce beetles, so will need to edit then.
    /// Uses "one-component-at-a-time" connected component labelling.
    /// See: https://en.wikipedia.org/wiki/Connected-component_labeling?oldformat=true#Pseudocode_for_the_one-component-at-a-time_algorithm
    pub fn hive_break_check(&self, source: &T, dest: &T) -> bool {
        // Store locations of blobs (almagamations of chips on the board that neighbour at least one other chip)
        let mut blobs: HashSet<T> = HashSet::new();

        // Get the positions of all the chips on the board
        let mut chip_positions = self.get_placed_positions();

        // Move current chip from source to dest to simulate its relocation.
        chip_positions.retain(|&p| p != *source); // remove
        chip_positions.insert(*dest); // add

        // Start connected component labelling at dest hex (doesn't matter where we start)
        let mut queue = vec![*dest];

        // Keep searching for neighbours until the queue is empty
        while let Some(position) = queue.pop() {
            // Pop an element out of the queue and get the co-ordinates of neighbouring hexes (including those 1 layer up and down if they exist)
            let neighbour_hexes = self.coord.neighbours_all(position);

            // If any of these neighbour hexes co-ordinates also appear in the chip_positions, it means they're a neighbouring chip
            // If they're a new entry, add them to the queue and the hashset, otherwise ignore them and move on
            for n in neighbour_hexes.into_iter() {
                if (chip_positions.contains(&n)) && (!blobs.contains(&n)) {
                    blobs.insert(n);
                    queue.push(n);
                }
            }
        }
        // The no. of chips in blobs should equal no. of chips on the board.
        // If it's not then the move has created two blobs (i.e. split the hive)
        blobs.len() != self.get_placed_positions().len()
    }

    /// Check if any animal-specific constraints of chip prevent a move from source to dest
    fn animal_constraint(&self, chip: Chip, source: &T, dest: &T) -> MoveStatus {
        // If it's a mosquito, we'll treat the chip name as the second char

        let is_mosquito = chip.name.starts_with('m');
        let checker = match is_mosquito {
            true => {
                match self
                    .get_position_byname(chip.team, chip.name)
                    .unwrap()
                    .get_layer()
                    > 0
                {
                    true => 'b',
                    false => chip.name.chars().nth(1).unwrap(), // skip the first letter if mosquito
                }
            }
            false => chip.name.chars().next().unwrap(),
        };

        // if it's a mosquito on layer >0, it's really a beetle

        // Match on chip animal (first character of chip.name)
        match checker {
            'a' => animals::ant_check(self, source, dest), // ants
            's' => animals::spider_check(self, source, dest), // spiders
            'q' | 'p' => animals::bee_check(self, source, dest), // bees and pillbugs
            'l' => animals::ladybird_check(self, source, dest), // ladybirds
            'b' => animals::bee_check(self, source, dest), // beetles
            'g' => animals::ghopper_check(self, source, dest), // grasshoppers
            _ => panic!("No other animals should exist"), // there are no other valid chip names, mosquitos don't have their own movesets
        }
    }

    /// Check if there's something one layer above this location
    fn sat_on_me(&self, source: T) -> bool {
        // Go up one layer from current position
        let mut my_position = source;
        my_position.ascend();

        // If there's something there, you're being beetle blocked
        self.get_chip(my_position).is_some()
    }

    /// See if either team has won (called at the end of current team's turn).
    /// Are any bees surrounded by 6 neighbours?
    fn check_win_state(&self, team: Team) -> MoveStatus {
        if (self.bee_neighbours(team) == 6) & (self.bee_neighbours(!team) == 6) {
            MoveStatus::Win(None) // both teams' bees have 6 neighbours, it's a draw
        } else if self.bee_neighbours(!team) == 6 {
            MoveStatus::Win(Some(team)) // opponent's bee has 6 neighbours, you win
        } else if self.bee_neighbours(team) == 6 {
            MoveStatus::Win(Some(!team)) // your own bee has 6 neighbours, you lose
        } else {
            MoveStatus::Success // nothing, continue game
        }
    }

    /// How many neighbours does this team's bee have?
    fn bee_neighbours(&self, team: Team) -> usize {
        match self
            .chips
            .iter()
            .find(|(c, p)| (c.team == team) & (c.name == "q1") & (p.is_some()))
            .map(|(_, p)| self.count_neighbours(p.unwrap()))
        {
            Some(value) => value,
            None => panic!("{:?} bee has no neighbours. Something went wrong.", team),
        }
    }

    /// Get co-ordinates of all chips that are already placed on the board
    pub fn get_placed_positions(&self) -> HashSet<T> {
        self.chips.values().flatten().copied().collect()
    }

    /// Get all chips that are already placed on the board by a given team
    fn get_placed_chips(&self, team: Team) -> Vec<Chip> {
        self.chips
            .iter()
            .filter(|(c, p)| (p.is_some()) & (c.team == team))
            .map(|(c, _)| *c)
            .collect()
    }

    /// Return the Chip that is at a given position (None if location is empty)
    /// TODO: This will break if we move away from a 3-coordinate system (as may other fns)
    pub fn get_chip(&self, position: T) -> Option<Chip> {
        self.chips
            .iter()
            .find_map(|(c, p)| if *p == Some(position) { Some(*c) } else { None })
    }

    /// Return a vector of neighbouring chips. This always returns the top-most chip if there is a stack
    /// of beetles.
    pub fn get_neighbour_chips(&self, position: T) -> Vec<Chip> {
        // Get neighbouring tiles on layer 0
        let layer0_neighbour = self.coord.neighbours_layer0(position);

        // Get the topmost chip on a stack
        let get_topmost = |mut dest: T| {
            // Start at layer 1
            dest.ascend();
            // If there's a chip there, go up a layer, keep going until no chip
            while self.get_chip(dest).is_some() {
                dest.ascend();
            }
            // Go down one layer to reach the position of the top-most chip
            dest.descend();
            self.get_chip(dest)
        };

        // Get the top-most neighbour hex
        let neighbour_chips = layer0_neighbour
            .into_iter()
            .map(get_topmost)
            .collect::<Vec<Option<Chip>>>();

        // Unwrap Vec<Option<Chip>> into Vec<Chip>
        match neighbour_chips.iter().all(|c| c.is_none()) {
            true => panic!("All neighbouring hexes have no chips. This should not happen!"),
            false => neighbour_chips.into_iter().flatten().collect::<Vec<Chip>>(),
        }
    }

    /// Return a chip's position based on its name and team
    pub fn get_position_byname(&self, team: Team, name: &'static str) -> Option<T> {
        let chip_select = Chip::new(name, team); // Select the chip

        // Get its location
        match self.chips.get(&chip_select) {
            Some(value) => *value,
            None => panic!("Something went very wrong: the chip doesn't exist."),
        }
    }

    /// Count number of neighbouring chips at given position
    /// This always counts neighbours in layer 0, even if position is in layer 1, 2, etc.
    pub fn count_neighbours(&self, position: T) -> usize {
        // Get the co-ordinates of neighbouring hexes
        let neighbour_hexes = self.coord.neighbours_layer0(position);

        // Get all placed chip positions
        let chip_positions = self.get_placed_positions();

        // Count the common elements
        neighbour_hexes.intersection(&chip_positions).count()
    }

    /// Convert the current board into a spiral notation string
    pub fn encode_spiral(&self) -> String {
        let mut return_string = String::new();

        // The first 3 couplets will define the turn number (0-9999), and the board size (0-99)
        let _ = write!(return_string, "{:0>4}", self.turns);
        let _ = write!(return_string, "{:0>2}", self.size);

        //return_string.push_str(&format!("{:0>4}", self.turns));
        //return_string.push_str(&format!("{:0>2}", self.size));

        // Return nothing for the board if it's empty
        if self.get_placed_positions().is_empty() {
            return return_string;
        }

        // Get spiral coord (key) of each chip (value) on board, as sorted BTree
        let spiral_tree = self
            .chips
            .iter()
            .filter(|(_, p)| p.is_some())
            .map(|(c, p)| (p.unwrap().mapto_spiral().unwrap(), *c))
            .collect::<BTreeMap<Spiral, Chip>>();

        // Create a variable to keep track of the previous hex coord we checked
        let mut previous = *spiral_tree.keys().next().unwrap(); // initialise as the first value in BTree
        for (coord, chip) in spiral_tree {
            // If we've moved more than 1 spiral hex, record how many gaps there are after a forward slash
            if coord.u - previous.u > 1 {
                // Record the number of spaces to skip in duodecimal (0-143)
                return_string
                    .push_str(&(funcs::decimal_to_duo(coord.u - previous.u - 1).to_string()));
            }

            // Black chips are recorded in allcaps
            let mut chip_str = match chip.team == Team::Black {
                true => chip.name.to_uppercase(),
                false => chip.name.to_string(),
            };

            // If it's on a layer above 0, it's an elevated beetle or a mosquito. Re-encode these.
            if coord.l > 0 {
                chip_str = match chip_str.chars().next() {
                    Some('B') => chip_str.replace('B', "["),
                    Some('b') => chip_str.replace('b', "("),
                    Some('M') => chip_str.replace('M', ">"),
                    Some('m') => chip_str.replace('m', "<"),
                    _ => panic!("Error in conversion of layer>0 chip str"),
                };
            }

            return_string.push_str(&chip_str);
            previous = coord; // update the previous for the next loop
        }
        return_string
    }

    /// Convert from spiral notation string into a live board
    /// Take in a co-ordinate system or an empty board?
    pub fn decode_spiral(&self, spiral_code_opt: String) -> Self {
        let mut newboard = Board::new(self.coord);

        // Return a blank board if there's no string
        let spiral_code = match spiral_code_opt.is_empty() {
            true => return newboard,
            false => spiral_code_opt,
        };

        // The first 2 couplets are the board turn number, the third couplet is the board size
        let (turn_size, spiral_code) = spiral_code.split_at(6);
        let (turn, size) = turn_size.split_at(4);

        newboard.turns = turn.parse().unwrap();
        newboard.size = size.parse().unwrap();

        // String needs decoding in couplets
        let first_char_iter = spiral_code.chars().step_by(2); // this gets the first.
        let second_char_iter = spiral_code.chars().skip(1).step_by(2); // this gets the second char

        // For each couplet
        let mut hex = 0_usize;
        let mut layer = 0;
        for (a, b) in first_char_iter.zip(second_char_iter) {
            // Match on the first char
            match a {
                _ if a.is_alphabetic() => {
                    // We have a chip on layer 0
                    layer = 0;
                    newboard.subdecode(a, b, hex, layer);
                    hex += 1;
                }
                _ if a.is_ascii_digit() || a == 'x' || a == 'y' => {
                    // We have a duodecimal number. Skip some spaces.
                    let skip = funcs::duo_to_decimal(format!("{a}{b}"));
                    hex += skip;
                }
                _ if ['[', '(', '<', '>'].iter().any(|c| *c == a) => {
                    // If it starts with one of these characters, it's going up one layer
                    hex -= 1;
                    layer += 1;

                    // Decide what it is
                    let new_a = match a {
                        '[' => 'B',
                        '(' => 'b',
                        '<' => 'm',
                        '>' => 'M',
                        _ => panic!("Rust broke, we tried to match on something unreachable"),
                    };

                    newboard.subdecode(new_a, b, hex, layer);
                    hex += 1;
                }
                _ => panic!("Chip name not recognised."),
            }
        }

        newboard
    }

    /// Updates a mutable board based on a char code couplet, hex position and layer
    fn subdecode(&mut self, a: char, b: char, hex: usize, layer: i8) {
        // Build a chip and its position from the available info
        let team = match a.is_uppercase() {
            true => Team::Black,
            false => Team::White,
        };

        let name = format!("{}{}", a.to_lowercase(), b);
        let position = self.coord.mapfrom_spiral(Spiral { u: hex, l: layer });
        let chip = Chip::new(comps::convert_static(name).unwrap(), team);

        // Force override the chip's position on the board without rule checks
        self.chips.insert(chip, Some(position));
    }

    /// Try skip turn by checking if both bees have been placed
    pub fn try_skip_turn(&mut self, active_team: Team) -> MoveStatus {
        if self.bee_placed(active_team) && self.bee_placed(!active_team) {
            self.turns += 1;
            MoveStatus::Success
        } else {
            MoveStatus::NoSkip
        }
    }
}
