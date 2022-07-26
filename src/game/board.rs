// All of the game's logic and rules

use std::collections::{HashMap, HashSet};
use std::hash::Hash;

use super::comps::{other_team, Chip, Team};
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
    SmallGap,    // gap is too small for an ant/spider/bee to access
    TooFar(u32), // too far for this animal to travel

    // Following statuses are returned early game
    NoBee,   // You can't move existing chips because not placed bee yet
    BeeNeed, // You need to place your bee on this turn

    // Finally
    Win(Option<Team>), // You won the game
    Nothing,   // You did nothing this turn
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
            // 1 bee, 2 spiders, 2 beetles, 3 grasshoppers, 3 ants, 1 each of mosquito, ladybug, pill bug
            (Chip::new("s1", Team::Black), None),
            (Chip::new("s2", Team::Black), None),
            (Chip::new("a1", Team::Black), None),
            (Chip::new("a2", Team::Black), None),
            (Chip::new("a3", Team::Black), None),
            (Chip::new("q1", Team::Black), None),
            // White team's chips
            (Chip::new("s1", Team::White), None),
            (Chip::new("s2", Team::White), None),
            (Chip::new("a1", Team::White), None),
            (Chip::new("a2", Team::White), None),
            (Chip::new("a3", Team::White), None),
            (Chip::new("q1", Team::White), None),
        ]);

        Board {
            chips,
            turns: 0,
            coord,
        }
    }

    // During tests we want lots of pieces that move freely, so give each team 8 ants and one bee
    pub fn test_board(coord: T) -> Self {
        let chips: HashMap<Chip, Option<(i8, i8, i8)>> = HashMap::from([
            // Black team's chips
            (Chip::new("a1", Team::Black), None),
            (Chip::new("a2", Team::Black), None),
            (Chip::new("a3", Team::Black), None),
            (Chip::new("a4", Team::Black), None),
            (Chip::new("a5", Team::Black), None),
            (Chip::new("a6", Team::Black), None),
            (Chip::new("a7", Team::Black), None),
            (Chip::new("a8", Team::Black), None),
            (Chip::new("q1", Team::Black), None),
            // White team's chips
            (Chip::new("a1", Team::White), None),
            (Chip::new("a2", Team::White), None),
            (Chip::new("a3", Team::White), None),
            (Chip::new("a4", Team::White), None),
            (Chip::new("a5", Team::White), None),
            (Chip::new("a6", Team::White), None),
            (Chip::new("a7", Team::White), None),
            (Chip::new("a8", Team::White), None),
            (Chip::new("q1", Team::White), None),
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

    // Move chip from player's hand to the board at selected position (the destination)
    fn place_chip(&mut self, chip: Chip, destination: (i8, i8, i8)) -> MoveStatus {
        // There are three constraints for placement of new chip:
        // Constraint 0) team must have placed their bee (only need to check on players' turn 3, board turns 4 and 5)
        // Constraint 1) it can't be placed on top of another chip;
        // Constraint 2) it must have at least one neighbour (after turn 1);
        // Constraint 3) its neighbours must be on the same team (after turn 2).

        let team = chip.team;

        // If turn number is 4 or 5, if this player hasn't placed their bee and isn't trying to, throw a fit.
        if (self.turns == 4) | (self.turns == 5) {
            let placed_chips = self.get_placed_chips(team);
            if !placed_chips.iter().any(|c| c.name == "q1") & (chip.name != "q1") {
                return MoveStatus::BeeNeed;
            }
        }

        // Any chips already on board at the destination?
        let constraint1 = self
            .get_placed_positions()
            .iter()
            .any(|p| *p == destination);

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
        source: &(i8, i8, i8),
    ) -> MoveStatus {
        // Constraints for a relocation:
        // Constraint 0) player must have placed their bee (only need to check prior to board turn 6)
        // Constraint 1) chip relocate cannot break the hive in two;
        // Constraint 2) chip must end up adjacent to other tiles
        // Constraint 3) chip can't end up on top of another chip (unless beetle, but we'll worry about this later...)
        // And then there are animal-specific constraints

        let team = chip.team;

        // If turn number is 5 or less, if this player hasn't placed their bee, they can't move pieces.
        if self.turns <= 5 {
            let placed_chips = self.get_placed_chips(team);
            if !placed_chips.iter().any(|c| c.name == "q1") {
                return MoveStatus::NoBee;
            }
        }

        // Any chips already on board at the destination?
        let constraint1 = self
            .get_placed_positions()
            .iter()
            .any(|p| *p == destination);

        // Get hexes that neighbour the desired destination hex
        let neighbour_hex = self.coord.neighbour_tiles(destination);

        // Do we have at least one neighbour at the destination?
        let constraint2 = !neighbour_hex
            .into_iter()
            .map(|p| self.get_team(p))
            .any(|t| t.is_some());

        // Does moving the chip away from current position cause the hive to split?
        let constraint3 = self.hive_break_check(source, &destination);

        // Check animal-specific constraints of the move
        let constraint4 = self.animal_constraint(source, &destination);

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

            println!("My {:?} bee has {} neighbours", team ,self.bee_neighbours(team));
            println!("My opponent's {:?} bee has {} neighbours", other_team(team) ,self.bee_neighbours(other_team(team)));

            // Relocation of a peice could result in the game being won (or drawn)
            // This will simply return MoveStatus::Success if nobody won (i.e. game should continue)
            self.check_win_state(team)
        }
    }


    fn check_win_state(&self, team: Team) -> MoveStatus {
        // Checks the board to see if any bees are surrounded by 6 neighbours
        if (self.bee_neighbours(team) == 6) & (self.bee_neighbours(other_team(team)) == 6) {
            MoveStatus::Win(None)                   // both teams' bees have 6 neighbours, it's a draw
        } else if self.bee_neighbours(other_team(team)) == 6 {
            MoveStatus::Win(Some(team))             // opponent's bee has 6 neighbours, you win
        } else if self.bee_neighbours(team) == 6 {
            MoveStatus::Win(Some(other_team(team))) // your own bee has 6 neighbours, you lose
        } else {
            MoveStatus::Success                     // nothing happened, continue the game
        }
    }

    // Check if moving a chip out of the current position splits the hive
    fn hive_break_check(&self, source: &(i8, i8, i8), destination: &(i8, i8, i8)) -> bool {
        // This function might fail tests when we introduce beetles later, so will need to edit then.

        // To achieve this, we need to do some connected component labelling.
        // A "one-component-at-a-time" algorithm is one of the simplest ways to find connected components in a grid.
        // More info: https://en.wikipedia.org/wiki/Connected-component_labeling?oldformat=true#Pseudocode_for_the_one-component-at-a-time_algorithm

        // Create an empty hash set to store the locations of all chips on the board that neighbour at least one other chip
        let mut store: HashSet<(i8, i8, i8)> = HashSet::new();

        // Get the positions of all the chips on the board
        let mut flat_vec = self.rasterscan_board();

        // Remove chip at our "source", and add to destination to simulate its move.
        flat_vec.retain(|&p| p != *source); // remove
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
        let mut flat_vec = self.get_placed_positions();
        // sort the vector in raster_scan order
        self.coord.raster_scan(&mut flat_vec);

        flat_vec
    }

    fn animal_constraint(&self, source: &(i8, i8, i8), destination: &(i8, i8, i8)) -> MoveStatus {
        // Check if any animal-specific constraints prevent the move

        // Match on chip animal (first character of chipname)
        match self.get_chip(*source).unwrap().name.chars().next().unwrap() {
            'a' => self.ant_check(source, destination),       // ants
            's' => self.spider_check(source, destination, 3), // spiders
            'q' => self.bee_check(source, destination, 1),    // queens
            _ => MoveStatus::Success,                         // todo, other animals
        }
    }

    fn ant_check(&self, source: &(i8, i8, i8), destination: &(i8, i8, i8)) -> MoveStatus {
        // Get positions of hexes that are inaccessible to ants, bees and spiders
        // Achieve this by morphological closing of a binary image of the board: i.e. dilation followed by erosion
        // Any new hexes that are generated by this operation will be in locations that ants can't access.

        // Get the positions of chips on the board as a flat sorted vector (i.e. raster scan the board)
        // Doesn't need to be a raster scan for this function to work, but we have the method defined already
        let mut flat_vec = self.rasterscan_board();

        // Remove the chip at our "source" from our flat vector, we don't want it to be part of our dilation
        flat_vec.retain(|&p| p != *source);

        // Get hexes that this ant/bee/spider can't access
        let forbidden_hexes = morphops::gap_closure(&self.coord, &flat_vec);

        // Are any of those hexes equal to the desired destination?
        match forbidden_hexes.iter().any(|t| t == destination) {
            true => MoveStatus::SmallGap,
            false => MoveStatus::Success,
        }
    }

    fn bee_check(
        &self,
        source: &(i8, i8, i8),
        destination: &(i8, i8, i8),
        stamina: u32,
    ) -> MoveStatus {
        // Do an ant_check plus move distance =stamina (the distance this peice can move)
        match self.ant_check(source, destination) {
            MoveStatus::SmallGap => MoveStatus::SmallGap,
            MoveStatus::Success => {
                // Check if the distance is within this animal's travel range (its stamina)
                // Simple check for bee - could replace this with "is the hex a neighbour?""
                match self.coord.hex_distance(*source, *destination) <= stamina {
                    true => MoveStatus::Success,
                    false => MoveStatus::TooFar(stamina),
                }
            }
            _ => unreachable!(), // this chip can't return other movestatus types
        }
    }

    fn spider_check(
        &self,
        source: &(i8, i8, i8),
        destination: &(i8, i8, i8),
        stamina: u32,
    ) -> MoveStatus {
        // Do an ant check first
        match self.ant_check(source, destination) {
            MoveStatus::SmallGap => MoveStatus::SmallGap,
            MoveStatus::Success => {
                // Check if the spider can reach the hex with a distance-limited floot fill
                // check if destination appears within list of visitable hexes
                let visitable = self.dist_lim_floodfill(source, stamina);
                match visitable.contains(destination) {
                    true => MoveStatus::Success,
                    false => MoveStatus::TooFar(stamina),
                }
            }
            _ => unreachable!(), // this chip can't return other movestatus types
        }
    }

    pub fn dist_lim_floodfill(&self, source: &(i8, i8, i8), stamina: u32) -> HashSet<(i8, i8, i8)> {
        // Distance-limited flood fill for movement ranges around obstacles
        // See: https://www.redblobgames.com/grids/hexagons/#distances

        // This will return all of the hexes that it is possible for this chip to visit given its stamina
        // Useful for spider.

        // We'll store visitable hexes in this empty vec. Use a vec because it has the iter_mut
        let mut visitable = HashSet::new();

        // Add starting position to the vector
        visitable.insert(*source);

        // We'll store fringes too: a list of all hexes that can be reached in k steps
        let mut fringes = HashMap::new();

        // Add the current position to fringes. It can be reached in k = 0 steps.
        fringes.insert(*source, 0);

        // Also need the position of existing chips on the board
        let obstacles = self.get_placed_positions();

        for k in 1..=stamina {
            // Get a list of hexes in fringes that have values of k-1
            let check_hexes = fringes
                .iter()
                .filter(|(p, v)| **v == k - 1)
                .map(|(p, v)| *p)
                .collect::<Vec<(i8, i8, i8)>>();

            // For each of those hexes
            for check_hex in check_hexes {
                // Get the 6 neighbours
                let neighbours = self.coord.neighbour_tiles(check_hex);

                // Collect up neighbours (n) that aren't in visited (v) and aren't blocked by an obstacle (o)
                neighbours.iter().for_each(|n| {
                    if !obstacles.contains(n) & !visitable.contains(n) {
                        visitable.insert(*n);
                        fringes.insert(*n, k);
                    }
                });
            }
        }

        visitable
    }

    // Get co-ordinates of all chips that are already placed on the board
    fn get_placed_positions(&self) -> Vec<(i8, i8, i8)> {
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
    fn get_chip(&self, position: (i8, i8, i8)) -> Option<Chip> {
        self.chips
            .iter()
            .find_map(|(c, p)| if *p == Some(position) { Some(*c) } else { None })
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

    fn count_neighbours(&self, position: (i8, i8, i8)) -> usize {
        let mut store = HashSet::new();

        // Get the co-ordinates of neighbouring hexes
        let neighbour_hexes = self.coord.neighbour_tiles(position);

        // If any of these neighbouring hex co-ordinates also appear in the board's list of chips, it means they're a neighbouring chip
        let flat_vec = self.rasterscan_board();

        // Add them to the hashset
        for elem in neighbour_hexes.iter() {
            for elem2 in flat_vec.clone().iter() {
                if (elem == elem2) & (!store.contains(elem2)) {
                    store.insert(*elem2); // add the neighbour to the hashset
                }
            }
        }

        store.len()
    }
}
