/// Board module tracks the chips and executes their moves
use std::collections::{HashMap, HashSet};

use super::comps::{self, Chip, Team}; // Game components (chips, teams)
use crate::game::{animals, history::History, movestatus::MoveStatus}; // Animal logic, move tracking and history
use crate::maths::coord::Coord; // Hexagonal coordinate system

/// The Board struct keeps track of game's progress, history and execution of rules
#[derive(Debug, Eq, PartialEq)]
pub struct Board<T: Coord> {
    pub chips: HashMap<Chip, Option<T>>, // player chips (both teams)
    pub turns: u32,                      // number of turns that have elapsed
    pub coord: T,                        // coordinate sytem for the board e.g. Cube, HECS
    pub history: History,                // record of all previous moves
}

impl<T> Board<T>
where
    T: Coord,
{
    /// Initialises a new board with a given coordinate system.
    pub fn new(coord: T) -> Self {
        // Chips for each team initialised in players' hands (position == None)
        let chips = comps::starting_chips();
        let history = History::new(); // Blank history

        Board {
            chips,
            turns: 0,
            coord,
            history,
        }
    }

    /// Execute the move of chip to destination, update the board's history and increment turn number.
    pub fn update(&mut self, chip: Chip, dest: T) {
        // Overwrite the chip's position in the board's HashMap
        self.chips.insert(chip, Some(dest));

        // Update history (in dheight coords)
        self.history
            .add_event(self.turns, chip, self.coord.mapto_doubleheight(dest));

        // Increment turns by 1
        self.turns += 1;
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
                    Some(source) => self.relocate_chip(chip_select, position, source),
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
    /// 1) player must have placed the bee by their turn 3 (board turns 4 and 5)
    /// 2) can't place on top of another chip;
    /// 3) must have at least one neighbour (ater turn 1);
    /// 4) neighbours must be on the same team (after turn 2).
    fn place_chip(&mut self, chip: Chip, dest: T) -> MoveStatus {
        // Check if a bee has been placed by player turn 3
        if (self.turns == 4) | (self.turns == 5)
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
        } else if self.turns > 1
            && self
                .get_neighbour_chips(dest)
                .iter()
                .any(|c| c.team != chip.team)
        {
            // Are any neighbours not on my team after turn 1
            MoveStatus::BadNeighbour
        } else {
            // No problem: execute the move.
            self.update(chip, dest);
            MoveStatus::Success
        }
    }

    /// Relocate a chip on the board at source to dest checking constraints:
    /// 1) player must have placed their bee (only need to check prior to board turn 6)
    /// 2) check other basic_constraints for moves
    /// 3) animal-specific constraints
    fn relocate_chip(&mut self, chip: Chip, dest: T, source: &T) -> MoveStatus {
        // Team can't relocate chips if they haven't placed bee.
        if self.turns <= 5 && !self.bee_placed(chip.team) {
            return MoveStatus::NoBee;
        }

        // Check basic constraints, checked during all relocations on board
        let basic_constraints = self.basic_constraints(dest, source);

        // Check animal-specific constraints of the move
        let animal_rules = self.animal_constraint(chip, source, &dest);

        if basic_constraints != MoveStatus::Success {
            basic_constraints
        } else if animal_rules != MoveStatus::Success {
            animal_rules
        } else {
            // No problem, execute the move
            self.update(chip, dest);
            // Relocation of chip could result in the game end
            self.check_win_state(chip.team) // Returns MoveStatus::Success if nobody won (game continues)
        }
    }

    /// Check if a team has placed their bee
    fn bee_placed(&self, team: Team) -> bool {
        self.get_placed_chips(team).iter().any(|c| c.name == "q1")
    }

    /// Basic constraints checked during all relocations, including pillbug sumos.
    /// A move from source to dest should not cause us to:
    /// 1) end turn on top of another chip (worry about beetle later);
    /// 2) have no neighbours, or;
    /// 3) split the hive.
    pub fn basic_constraints(&mut self, dest: T, source: &T) -> MoveStatus {
        // check constraints in this order because they're not all mutally exclusive and we want to return useful errors to users

        if self.get_chip(dest).is_some() {
            // Do we end up on top of another chip? (unless bettle);
            MoveStatus::Occupied
        } else if self.count_neighbours(dest) == 0 {
            // Do we have end up adjacent to no other tiles?
            MoveStatus::Unconnected
        } else if self.hive_break_check(source, &dest) {
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
    fn hive_break_check(&self, source: &T, dest: &T) -> bool {
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
            // Pop an element out of the queue and get the co-ordinates of neighbouring hexes
            let neighbour_hexes = self.coord.neighbour_tiles(position);

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
        // Match on chip animal (first character of chip.name)
        match chip.name.chars().next().unwrap() {
            'a' => animals::ant_check(self, source, dest), // ants
            's' => animals::spider_check(self, source, dest), // spiders
            'q' | 'p' => animals::bee_check(self, source, dest), // bees and pillbugs
            'l' => animals::ladybird_check(self, source, dest), // ladybirds
            _ => MoveStatus::Success,                      // todo, other animals
        }
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

    /// Return a vector of neighbouring chips
    pub fn get_neighbour_chips(&self, position: T) -> Vec<Chip> {
        let neighbour_hexes = self.coord.neighbour_tiles(position);

        // Get the chips in neighbouring hexes
        let neighbour_chips = neighbour_hexes
            .into_iter()
            .map(|h| self.get_chip(h))
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
    pub fn count_neighbours(&self, position: T) -> usize {
        // Get the co-ordinates of neighbouring hexes
        let neighbour_hexes = self.coord.neighbour_tiles(position);

        // Get all placed chip positions
        let chip_positions = self.get_placed_positions();

        // Count the common elements
        neighbour_hexes.intersection(&chip_positions).count()
    }
}
