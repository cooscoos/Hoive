// Handles the game's base logic and rules

use std::collections::{HashMap, HashSet, BTreeSet};

use super::comps::{other_team, starting_chips, test_chips, Chip, Team}; // Game components
use crate::game::animals; // Animal movement logic
use crate::maths::coord::Coord; // Coord trait applies to all hex co-ordinate systems

// A "move" in Hive is defined as either a:
// i) Placement: a new chip moves from player's hand to the board, i.e.: position = None --> position = Some(a, r, c);
// ii) Relocation: a chip already on the board moves to somewhere else on board, i.e.: position = Some(a, r, c) --> position = Some(a', r', c').

// Enum to return the status of whether a move was legal
#[derive(Debug, Eq, PartialEq)]
pub enum MoveStatus {
    Success, // The placement/relocation of the chip was legal, the move was executed
    // Following statuses are returned when move can't be executed because the target space...:
    Occupied,     // is already occupied
    Unconnected,  // is in the middle of nowhere
    BadNeighbour, // is next to opposing team
    HiveSplit,    // would split the hive in two

    // Following statuses are specific to animals / groups of animals
    SmallGap,         // gap is too small for an ant/spider/bee to access
    BadDistance(u32), // wrong distance for this animal to travel
    RecentMove(Chip), // chip moved too recently for its special move to be executed

    // Following statuses are returned early game
    NoBee,   // You can't move existing chips because not placed bee yet
    BeeNeed, // You need to place your bee on this turn

    // Finally
    Win(Option<Team>), // You won the game
    Nothing,           // You did nothing this turn
}

// The board struct is the game and all of its base logic
pub struct Board<T: Coord> {
    pub chips: HashMap<Chip, Option<(i8, i8, i8)>>,
    pub turns: u32, // tracks number of turns that have elapsed
    pub coord: T,   // The coordinate sytem for the board e.g. HECS, Cube
}

impl<T> Board<T>
where
    T: Coord,
{
    pub fn default(coord: T) -> Self {
        // At new game, initialise all of the chips for each team with position = None (in player hand)
        let chips = starting_chips();

        Board {
            chips,
            turns: 0,
            coord,
        }
    }

    pub fn test_board(coord: T) -> Self {
        // During testing we often want lots of pieces that move freely, so give each team 8 ants and one bee
        let chips = test_chips();

        Board {
            chips,
            turns: 0,
            coord,
        }
    }

    // Try move a chip of given name / team, to a new position. Return MoveStatus to tell the main loop how successful the attempt was.
    pub fn move_chip(
        &mut self,
        name: &'static str,
        team: Team,
        position: (i8, i8, i8),
    ) -> MoveStatus {
        let chip_select = Chip::new(name, team); // Select the chip

        // A chip's current position tells us if we're "placing" from player's hand, or "relocating" on board
        let move_status = match self.chips.clone().get(&chip_select) {
            Some(p) => {
                match p {
                    Some(source) => {
                        // chip already has a position, so we must be relocating it
                        self.relocate_chip(chip_select, position, source)
                    }
                    None => {
                        // chip's current position == None (player hand), so we must be placing it
                        self.place_chip(chip_select, position)
                    }
                }
            }
            None => panic!(
                "Something went very wrong. The chip can't be moved because it doesn't exist."
            ),
        };
        move_status
    }

    // Move chip from player's hand to the board at position = dest
    fn place_chip(&mut self, chip: Chip, dest: (i8, i8, i8)) -> MoveStatus {
        // There are constraints for placement of a new chip:
        // Constraint 0) team must have placed the bee (only need to check on players' turn 3, board turns 4 and 5)
        // Constraint 1) can't be placed on top of another chip;
        // Constraint 2) must have at least one neighbour (after turn 1);
        // Constraint 3) neighbours must be on the same team (after turn 2).

        // The current team
        let team = chip.team;

        // If turn number is 4 or 5, and if this player hasn't placed their bee and isn't trying to, then illegal.
        if (self.turns == 4) | (self.turns == 5) {
            let placed_chips = self.get_placed_chips(team);
            if !placed_chips.iter().any(|c| c.name == "q1") & (chip.name != "q1") {
                return MoveStatus::BeeNeed;
            }
        }

        // Any chips already on board already at dest?
        let constraint1 = self.get_placed_positions().iter().any(|p| *p == dest);

        // List of neighbouring hexes
        let neighbour_hex = self.coord.neighbour_tiles(dest);

        // Is there at least one chip neighbouring dest?
        let constraint2 = !neighbour_hex
            .into_iter()
            .map(|p| self.get_team(p))
            .any(|t| t.is_some());

        // This will return true if any of the chips neighbouring the dest are on a different team
        let constraint3 = !neighbour_hex
            .into_iter()
            .map(|p| self.get_team(p))
            .filter(|t| t.is_some())
            .all(|t| t.unwrap() == team);

        if constraint1 {
            MoveStatus::Occupied
        } else if self.turns >= 1 && constraint2 {
            MoveStatus::Unconnected
        } else if self.turns >= 2 && constraint3 {
            MoveStatus::BadNeighbour
        } else {
            self.chips.insert(chip, Some(dest)); // Overwrite the chip's position in the HashMap
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
        // Constraint 1) chip relocate cannot break the hive in two;
        // Constraint 2) chip must end up adjacent to other tiles
        // Constraint 3) chip can't end up on top of another chip (unless beetle, but we'll worry about this later...)
        // Constraint N) then there are animal-specific constraints

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
            self.chips.insert(chip, Some(dest)); // Overwrite the chip's position in the HashMap

            // Relocation of chip could result in the game end
            self.check_win_state(team) // Returns MoveStatus::Success if nobody won (game continues)
        }
    }

    pub fn basic_constraints(&mut self, dest: (i8, i8, i8), source: &(i8, i8, i8)) -> MoveStatus {
        // Basic constraints are checked during all player moves, but also when a pillbug forces
        // another chip to move. When pillbug forces move, only cons1-3 need checking.
        // Constraint 1) chip relocate cannot break the hive in two;
        // Constraint 2) chip must end up adjacent to other tiles
        // Constraint 3) chip can't end up on top of another chip
        // Constraint 2 will always be true for a pillbug, but we need to check constraints 1-3 in that order
        // so that we can return error messages that make sense.

        // Any chips already on board at the dest?
        let constraint1 = self.get_placed_positions().iter().any(|p| *p == dest);

        // Get hexes that neighbour dest
        let neighbour_hex = self.coord.neighbour_tiles(dest);

        // Do we have at least one neighbour at dest?
        let constraint2 = !neighbour_hex
            .into_iter()
            .map(|p| self.get_team(p))
            .any(|t| t.is_some());

        // Does moving the chip away from current position cause the hive to split?
        let constraint3 = self.hive_break_check(source, &dest);

        // check constraints in this order because all unconnected moves are also hive splits, and we want to return useful error messages
        if constraint1 {
            MoveStatus::Occupied
        } else if constraint2 {
            MoveStatus::Unconnected
        } else if constraint3 {
            MoveStatus::HiveSplit
        } else {
            MoveStatus::Success
        }
    }

    fn check_win_state(&self, team: Team) -> MoveStatus {
        // Are any bees surrounded by 6 neighbours?
        if (self.bee_neighbours(team) == 6) & (self.bee_neighbours(other_team(team)) == 6) {
            MoveStatus::Win(None) // both teams' bees have 6 neighbours, it's a draw
        } else if self.bee_neighbours(other_team(team)) == 6 {
            MoveStatus::Win(Some(team)) // opponent's bee has 6 neighbours, you win
        } else if self.bee_neighbours(team) == 6 {
            MoveStatus::Win(Some(other_team(team))) // your own bee has 6 neighbours, you lose
        } else {
            MoveStatus::Success // nothing, continue game
        }
    }

    // How many neighbours does this team's queen bee have?
    fn bee_neighbours(&self, team: Team) -> usize {
        let neighbours = self
            .chips
            .iter()
            .filter(|(c, p)| (c.team == team) & (c.name == "q1") & (p.is_some()))
            .map(|(_, p)| self.count_neighbours(p.unwrap()))
            .collect::<Vec<usize>>();

        neighbours[0]
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
        chip_positions.push(*dest); // add

        // Start connected component labelling at dest hex (doesn't matter where we start)
        let mut queue = vec![*dest];

        // Keep searching for neighbours until the queue is empty
        while let Some(position) = queue.pop() {
            // Pop an element out of the queue and get the co-ordinates of neighbouring hexes
            let neighbour_hexes = self.coord.neighbour_tiles(position);

            // If any of these neighbour hexes co-ordinates also appear in the chip_positions, it means they're a neighbouring chip
            // If they're a new entry, add them to the queue and the hashset, otherwise ignore them and move on
            // We have a double for loop with an if statement to try and find common elements in two vectors.
            // This is likely inefficient, and doesn't feel very rusty, but it works for now.
            for elem in neighbour_hexes.iter() {
                for elem2 in chip_positions.clone().iter() {
                    if (elem == elem2) & (!blobs.contains(elem2)) {
                        blobs.insert(*elem2);
                        queue.push(*elem2);
                    }
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

    // Get co-ordinates of all chips that are already placed on the board
    pub fn get_placed_positions(&self) -> Vec<(i8, i8, i8)> {
        self.chips.values().flatten().copied().collect()
    }

    // Raster scan chips on the board to return sorted positions
    // Not used much if at all during game, but is useful for tests
    pub fn rasterscan_board(&self) -> Vec<(i8, i8, i8)> {
        let mut chip_positions = self.get_placed_positions();
        self.coord.raster_scan(&mut chip_positions);
        chip_positions
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
            false => neighbour_chips
                .into_iter()
                .filter(|c| c.is_some())
                .map(|c| c.unwrap())
                .collect::<Vec<Chip>>(),
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

    fn count_neighbours(&self, position: (i8, i8, i8)) -> usize {
        // Count number of neighbouring chips

        // Store neighbours here
        let mut neighbours = HashSet::new();

        // Get the co-ordinates of neighbouring hexes
        let neighbour_hexes = self.coord.neighbour_tiles(position);

        // If any of these neighbouring hex coords also appear in the board's list of chips, it means they're a neighbouring chip
        let chip_positions = self.get_placed_positions();

        // Add common vector elements to the hashset using that terrible double-for loop
        for elem in neighbour_hexes.iter() {
            for elem2 in chip_positions.clone().iter() {
                if (elem == elem2) & (!neighbours.contains(elem2)) {
                    neighbours.insert(*elem2); // add the neighbour to the hashset
                }
            }
        }

        neighbours.len()
    }
}
