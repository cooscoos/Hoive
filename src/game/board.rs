/// Board module tracks the chips and executes their moves
use std::collections::{HashMap, HashSet};

use super::comps::{self, Chip, Team}; // Game components (chips, teams)
use crate::game::{animals, history::History, movestatus::MoveStatus}; // Animal logic, move tracking and history
use crate::maths::coord::Coord; // Hexagonal coordinate system

/// The Board struct keeps track of game's progress, history and execution of rules
#[derive(Debug, Eq, PartialEq)]
pub struct Board<T: Coord> {
    pub chips: HashMap<Chip, Option<(i8, i8, i8)>>, // player chips (both teams)
    pub turns: u32,                                 // number of turns that have elapsed
    pub coord: T,         // coordinate sytem for the board e.g. Cube, HECS
    pub history: History, // record of all previous moves
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
    pub fn update(&mut self, chip: Chip, dest: (i8, i8, i8)) {
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
    pub fn move_chip(
        &mut self,
        name: &'static str,
        team: Team,
        position: (i8, i8, i8),
    ) -> MoveStatus {
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
    fn place_chip(&mut self, chip: Chip, dest: (i8, i8, i8)) -> MoveStatus {
        // There are constraints for placement of a new chip:
        // Constraint 0) team must have placed the bee (only need to check on players' turn 3, board turns 4 and 5)
        // Constraint 1) can't be placed on top of another chip (ever, even if beetle);
        // Constraint 2) must have at least one neighbour (after turn 1);
        // Constraint 3) neighbours must be on the same team (after turn 2).

        // If turn number is 4 or 5, and if this player hasn't placed their bee and isn't trying to, then illegal.
        if (self.turns == 4) | (self.turns == 5) {
            let placed_chips = self.get_placed_chips(chip.team);
            if !placed_chips.iter().any(|c| c.name == "q1") & (chip.name != "q1") {
                return MoveStatus::BeeNeed;
            }
        }

        // There must be better way of doing this...
        // List of neighbouring hexes
        let neighbour_hex = self.coord.neighbour_tiles(dest);

        // This will return true if any of the chips neighbouring the dest are on a different team
        let constraint3 = !neighbour_hex
            .iter()
            .map(|p| self.get_team(*p))
            .filter(|t| t.is_some())
            .all(|t| t.unwrap() == chip.team);

        if self.get_chip(dest).is_some() {
            // // Any chips already on board already at dest?
            MoveStatus::Occupied
        } else if self.turns >= 1 && self.count_neighbours(dest) == 0 {
            // Is there at least one chip neighbouring dest after turn 1?
            MoveStatus::Unconnected
        } else if self.turns >= 2 && constraint3 {
            MoveStatus::BadNeighbour
        } else {
            // Overwrite the chip's position in the HashMap and update history
            self.update(chip, dest);
            MoveStatus::Success
        }
    }

    // Relocate a chip on the board
    fn relocate_chip(
        &mut self,
        chip: Chip,
        dest: (i8, i8, i8),
        source: &(i8, i8, i8),
    ) -> MoveStatus {
        // Constraints for a relocation:
        // Constraint 0) player must have placed their bee (only need to check prior to board turn 6)
        // Constraints 1-3) see method in basic_constraints
        // Constraint 4) animal-specific constraints

        let team = chip.team;

        // Constraint 0: team can't relocate chips if they haven't placed bee.
        if self.turns <= 5 {
            let placed_chips = self.get_placed_chips(team);
            if !placed_chips.iter().any(|c| c.name == "q1") {
                return MoveStatus::NoBee;
            }
        }

        // Constraints 1-3 are checked using method base_constraints
        let basic_constraints = self.basic_constraints(dest, source);

        // Check animal-specific constraints of the move
        let constraint_4 = self.animal_constraint(chip, source, &dest);

        if basic_constraints != MoveStatus::Success {
            basic_constraints
        } else if constraint_4 != MoveStatus::Success {
            constraint_4
        } else {
            // Overwrite the chip's position in the HashMap and update history
            self.update(chip, dest);

            // Relocation of chip could result in the game end
            self.check_win_state(team) // Returns MoveStatus::Success if nobody won (game continues)
        }
    }

    /// Basic constraints are checked during all moves, including pillbug sumos.
    /// We ask, does the move from source to dest cause any of the following:
    /// 1) we end turn on top of another chip (worry about beetle later);
    /// 2) we end up adjacent to no other tiles, or;
    /// 3) move splits the hive.
    /// 
    /// The returned MoveStatus describes which problems occurred (or "Success" if none occurred).
    pub fn basic_constraints(&mut self, dest: (i8, i8, i8), source: &(i8, i8, i8)) -> MoveStatus {


        // check constraints in this order because they're not all mutally exclusive and we want to return useful errors to users
        if self.get_chip(dest).is_some() {
            // Do we end up on top of another chip? (unless bettle, but worry about that later);
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

    fn hive_break_check(&self, source: &(i8, i8, i8), dest: &(i8, i8, i8)) -> bool {
        // Check if moving a chip out of the current position splits the hive
        // This function will likely cause test failure when we introduce beetles, so will need to edit then.

        // We need to do some connected component labelling. Use "one-component-at-a-time", because it's simple.
        // See: https://en.wikipedia.org/wiki/Connected-component_labeling?oldformat=true#Pseudocode_for_the_one-component-at-a-time_algorithm

        // Store locations of blobs (chips on the board that neighbour at least one other chip)
        let mut blobs: HashSet<(i8, i8, i8)> = HashSet::new();

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
        // If it's not then the move has created two blobs (split hive): illegal.
        blobs.len() != self.get_placed_positions().len()
    }

    fn animal_constraint(
        &self,
        chip: Chip,
        source: &(i8, i8, i8),
        dest: &(i8, i8, i8),
    ) -> MoveStatus {
        // Check if any animal-specific constraints prevent the move

        // Match on chip animal (first character of chipname)
        match chip.name.chars().next().unwrap() {
            'a' => animals::ant_check(self, source, dest), // ants
            's' => animals::spider_check(self, source, dest), // spiders
            'q' | 'p' => animals::bee_check(self, source, dest), // bees and pillbugs
            'l' => animals::ladybird_check(self, source, dest), // ladybirds
            _ => MoveStatus::Success,                      // todo, other animals
        }
    }

    fn check_win_state(&self, team: Team) -> MoveStatus {
        // Are any bees surrounded by 6 neighbours?
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

    // How many neighbours does this team's queen bee have?
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

    // Get co-ordinates of all chips that are already placed on the board
    pub fn get_placed_positions(&self) -> HashSet<(i8, i8, i8)> {
        self.chips.values().flatten().copied().collect()
    }

    // Get the chips that are already placed on the board by a given team
    fn get_placed_chips(&self, team: Team) -> Vec<Chip> {
        self.chips
            .iter()
            .filter(|(c, p)| (p.is_some()) & (c.team == team))
            .map(|(c, _)| *c)
            .collect()
    }

    // Get the Team enum of the chip at the given position. Return None if the hex is empty.
    fn get_team(&self, position: (i8, i8, i8)) -> Option<Team> {
        self.chips.iter().find_map(|(c, p)| {
            if *p == Some(position) {
                Some(c.team)
            } else {
                None
            }
        })
    }

    // Return the Chip that is at a given position (None if location is empty)
    // This will break if we move away from a 3-coordinate system
    pub fn get_chip(&self, position: (i8, i8, i8)) -> Option<Chip> {
        self.chips
            .iter()
            .find_map(|(c, p)| if *p == Some(position) { Some(*c) } else { None })
    }

    // Return a vector of neighbouring chips
    pub fn get_neighbour_chips(&self, position: (i8, i8, i8)) -> Vec<Chip> {
        let neighbour_hexes = self.coord.neighbour_tiles(position);
        let neighbour_chips = neighbour_hexes
            .into_iter()
            .map(|h| self.get_chip(h))
            .collect::<Vec<Option<Chip>>>();

        // The other rules should ensure that there will always be at least one neighbouring chip.
        // Still, if there are no neighbouring chips when this method is called, then panic
        // otherwise, return  vector of neighbouring chips.
        match neighbour_chips.iter().all(|c| c.is_none()) {
            true => panic!("All neighbouring hexes have no chips. This should not happen!"),
            false => neighbour_chips.into_iter().flatten().collect::<Vec<Chip>>(),
        }
    }

    // Return a chip's position based on its name and team
    pub fn get_position_byname(&self, team: Team, name: &'static str) -> Option<(i8, i8, i8)> {
        let chip_select = Chip::new(name, team); // Select the chip

        // Get its location
        match self.chips.get(&chip_select) {
            Some(value) => *value,
            None => panic!("Something went very wrong, the chip doesn't exist."),
        }
    }

    // Count number of neighbouring chips
    pub fn count_neighbours(&self, position: (i8, i8, i8)) -> usize {
        // Get the co-ordinates of neighbouring hexes
        let neighbour_hexes = self.coord.neighbour_tiles(position);

        // Get all placed chip positions
        let chip_positions = self.get_placed_positions();

        // Count the common elements
        neighbour_hexes.intersection(&chip_positions).count()
    }
}
