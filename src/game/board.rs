// All of the game's logic and rules

use std::collections::{HashMap, HashSet};

use super::comps::{Chip, Team};
use crate::draw;
use crate::maths::{coord::Coord, morphops}; // Coord trait applies to all hex co-ordinate systems

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
    SmallGap,     // gap is too small for an ant/spider/bee to access
}

// The board struct is the game and all of its logic
pub struct Board<T: Coord> {
    pub chips: HashMap<Chip, Option<(i8, i8, i8)>>,
    pub turns: u32, // tracks number of turns that have elapsed
    pub coord: T,   // The coordinate sytem for the board e.g. HECS, Cube
}

impl<T> Board<T>
where
    T: Coord,
{
    // At new game, initialise all of the chips for each team with position = None (in player hand)
    pub fn default(coord: T) -> Self {
        let chips: HashMap<Chip, Option<(i8, i8, i8)>> = HashMap::from([
            // Black team's chips
            (Chip::new("s1", Team::Black), None),
            (Chip::new("s2", Team::Black), None),
            (Chip::new("s3", Team::Black), None),
            (Chip::new("s4", Team::Black), None),
            // White team's chips
            (Chip::new("s1", Team::White), None),
            (Chip::new("s2", Team::White), None),
            (Chip::new("s3", Team::White), None),
            (Chip::new("s4", Team::White), None),
        ]);

        Board {
            chips,
            turns: 0,
            coord,
        }
    }

    // Parse the board out into doubleheight hex co-ordinates (a grid format more readable to humans)
    pub fn to_dheight(&self, size: i8) -> HashMap<(i8, i8), Option<Chip>> {
        // Initialise an empty doubleheight hashmap to store chips at each co-ordinate
        let mut dheight_hashmap = draw::empty(size);

        // Translate doubleheight co-ordinates to the current coord system being used by the board
        let board_coords = dheight_hashmap
            .iter()
            .map(|(xy, _)| self.coord.mapfrom_doubleheight(*xy))
            .collect::<HashSet<(i8, i8, i8)>>();

        // Check all board_coords for chips, and put the chips in dheight_hashmap if found
        board_coords.into_iter().for_each(|p| {
            dheight_hashmap.insert(self.coord.mapto_doubleheight(p), self.get_chip(p));
        });

        dheight_hashmap
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
                    Some(current_position) => {
                        // chip already has a position, so we must be relocating it
                        self.relocate_chip(chip_select, position, current_position)
                    }
                    None => {
                        // chip's current position == None (player hand), so we must be placing it
                        self.place_chip(chip_select, team, position)
                    }
                }
            }
            None => panic!(
                "Something went very wrong. The chip can't be moved because it doesn't exist."
            ),
        };
        move_status
    }

    // Move chip from player's hand to the board at selected position (the destination)
    fn place_chip(&mut self, chip: Chip, team: Team, destination: (i8, i8, i8)) -> MoveStatus {
        // There are three constraints for placement of new chip:
        // Constraint 1) it can't be placed on top of another chip;
        // Constraint 2) it must have at least one neighbour (after turn 1);
        // Constraint 3) its neighbours must be on the same team (after turn 2).

        // Any chips already on board at the destination?
        let constraint1 = self.get_placed().iter().any(|p| *p == destination);

        // Get the (a,r,c) values of hexes neighbouring the destination
        //let neighbour_hex = Board::neighbour_tiles(destination);
        let neighbour_hex = self.coord.neighbour_tiles(destination);

        // Do we have at least one neighbour at the destination?
        let constraint2 = !neighbour_hex
            .into_iter()
            .map(|p| self.get_team(p))
            .any(|t| t.is_some());

        // This will return true if any of the chips neighbouring the destination are on a different team
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
            self.chips.insert(chip, Some(destination)); // Overwrite the chip's position in the HashMap
            MoveStatus::Success
        }
    }

    // Relocate a chip on the board
    fn relocate_chip(
        &mut self,
        chip: Chip,
        destination: (i8, i8, i8),
        current_position: &(i8, i8, i8),
    ) -> MoveStatus {
        // Constraints for a relocation:
        // Constraint 1) chip relocate cannot break the hive in two;
        // Constraint 2) chip must end up adjacent to other tiles
        // Constraint 3) chip can't end up on top of another chip (unless beetle, but we'll worry about this later...)
        // And then there are animal-specific constraints



        // Any chips already on board at the destination?
        let constraint1 = self.get_placed().iter().any(|p| *p == destination);

        // Get hexes that neighbour the desired destination hex
        let neighbour_hex = self.coord.neighbour_tiles(destination);

        // Do we have at least one neighbour at the destination?
        let constraint2 = !neighbour_hex
            .into_iter()
            .map(|p| self.get_team(p))
            .any(|t| t.is_some());

        // Does moving the chip away from current position cause the hive to split?
        let constraint3 = self.hive_break_check(current_position, &destination);

        // Check animal-specific constraints of the move
        let constraint4 = self.animal_constraint(current_position, &destination);



        // check constraints in this order because all unconnected moves are also hive splits, and we want to return useful error messages
        if constraint1 {
            MoveStatus::Occupied
        } else if constraint2 {
            MoveStatus::Unconnected
        } else if constraint3 {
            MoveStatus::HiveSplit
        } else if constraint4 != MoveStatus::Success {
            constraint4
        } else {
            self.chips.insert(chip, Some(destination)); // Overwrite the chip's position in the HashMap
            MoveStatus::Success
        }
    }

    // Check if moving a chip out of the current position splits the hive
    fn hive_break_check(
        &self,
        current_position: &(i8, i8, i8),
        destination: &(i8, i8, i8),
    ) -> bool {
        // This function might fail tests when we introduce beetles later, so will need to edit then.

        // To achieve this, we need to do some connected component labelling.
        // A "one-component-at-a-time" algorithm is one of the simplest ways to find connected components in a grid.
        // More info: https://en.wikipedia.org/wiki/Connected-component_labeling?oldformat=true#Pseudocode_for_the_one-component-at-a-time_algorithm

        // Create an empty hash set to store the locations of all chips on the board that neighbour at least one other chip
        let mut store: HashSet<(i8, i8, i8)> = HashSet::new();

        // Get the positions of all the chips on the board
        let mut flat_vec = self.rasterscan_board();

        // Remove chip at our "current_position", and add to destination to simulate its move.
        flat_vec.retain(|&p| p != *current_position); // remove
        flat_vec.push(*destination); // add

        // The destination hex is as good a place as anywhere to start connected component labelling
        let mut queue = vec![*destination];

        // Keep searching for neighbours until the queue is empty
        loop {
            match queue.pop() {
                Some(position) => {
                    // Pop an element out of the queue and get the co-ordinates of neighbouring hexes
                    let neighbour_hexes = self.coord.neighbour_tiles(position);

                    // If any of these neighbouring hex co-ordinates also appear in the flat_vec, it means they're a neighbouring chip
                    // If they're a new entry, add them to the queue and the hashset, otherwise ignore them and move on
                    // Double for loop with an if doesn't seem very rusty, but it works for now.
                    for elem in neighbour_hexes.iter() {
                        for elem2 in flat_vec.clone().iter() {
                            if (elem == elem2) & (!store.contains(elem2)) {
                                store.insert(*elem2); // add the neighbour to the hashset
                                queue.push(*elem2); // also add it to the queue
                            }
                        }
                    }
                }
                None => break, // stop labelling if the queue is empty
            }
        }

        // The number of items stored should be all of the chips on the board.
        // If it's not then the move has created two hives, which is illegal.
        store.len() != self.rasterscan_board().len()
    }

    // Raster scan all chips on the board and returns their positions as a flat vector
    pub fn rasterscan_board(&self) -> Vec<(i8, i8, i8)> {
        // TODO: Could this just call fn get_placed??
        // Flatten the board's HashMap into a vector that only counts chips on the board (i.e. p.is_some())
        let mut flat_vec = self
            .chips
            .iter()
            .filter(|(_, p)| p.is_some())
            .map(|(_, p)| p.unwrap())
            .collect::<Vec<(i8, i8, i8)>>();

        let mut flat_vec = self.get_placed();
        // sort the vector in raster_scan order
        self.coord.raster_scan(&mut flat_vec);

        flat_vec
    }


    fn animal_constraint(&self, current_position: &(i8, i8, i8), destination: &(i8, i8, i8)) -> MoveStatus {

        // Check if any animal-specific constraints prevent the move

        // Match on chip animal (first character of chipname)
        match self.get_chip(*current_position).unwrap().name.chars().next().unwrap() {
            'a' | 's' | 'q' => self.ant_close(current_position, destination),   // ants, spiders and bees
            _ => MoveStatus::Success,   // todo, other animals
        }

    }

    fn ant_close(&self, current_position: &(i8, i8, i8), destination: &(i8, i8, i8)) -> MoveStatus {

        // Get positions of hexes that are inaccessible to ants, bees and spiders
        // Achieve this by morphological closing of a binary image of the board: i.e. dilation followed by erosion
        // Any new hexes that are generated by this operation will be in locations that ants can't access.

        // Get the positions of chips on the board as a flat sorted vector (i.e. raster scan the board)
        // Doesn't need to be a raster scan for this function to work, but we have the method defined already
        let mut flat_vec = self.rasterscan_board();

        // Remove the chip at our "current_position" from our flat vector, we don't want it to be part of our dilation
        flat_vec.retain(|&p| p != *current_position);

        // Get hexes that this ant/bee/spider can't access
        let forbidden_hexes = morphops::gap_closure(&self.coord, &flat_vec);

        // Are any of those hexes equal to the desired destination?
        match forbidden_hexes.iter().any(|t| t==destination){
            true => MoveStatus::SmallGap,
            false => MoveStatus::Success,
        }
    }

    // Get co-ordinates of all chips that are already placed on the board
    pub fn get_placed(&self) -> Vec<(i8, i8, i8)> {
        self.chips.values().flatten().copied().collect()
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
    fn get_chip(&self, position: (i8, i8, i8)) -> Option<Chip> {
        self.chips
            .iter()
            .find_map(|(c, p)| if *p == Some(position) { Some(*c) } else { None })
    }
}
