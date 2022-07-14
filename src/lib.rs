use std::{collections::HashMap, collections::HashSet, hash::Hash};

// enum to keep track of team identities
#[derive(Hash, Eq, PartialEq, Debug, Clone, Copy)]
pub enum Team {
    Black,
    White,
}

// enum to keep track of animal identities
#[derive(Hash, Eq, PartialEq, Debug, Clone, Copy)]
pub enum Animal {
    Ant,
    Spider,
    Bee,
    Beetle,
    Grasshopper,
    Ladybird,
    Mosquito,
}

// A "move" is defined as either a:
// i) Placement: a new chip moves from player's hand to the board, i.e.: position = None --> position = Some(a, r, c);
// ii) Relocation: a chip already on the board moves to somewhere else on board, i.e.: position = Some(a, r, c) --> position = Some(a', r', c').

// Enum to return status of whether a move (i.e. new placement or relocation of a chip) was legal
pub enum MoveStatus {
    Success, // The placement/relocation of the chip was legal, the move was executed
    // Following statuses are returned when move can't be executed because the target space...:
    Occupied,     // is already occupied
    Unconnected,  // is in the middle of nowhere
    BadNeighbour, // is next to opposing team
    HiveSplit,    // would split the hive in two
}

#[derive(Hash, Eq, PartialEq, Debug, Clone, Copy)]
pub struct Chip {
    name: &'static str, // names help us distinguish between e.g. multiple black team spiders
    animal: Animal,
    team: Team,
}

impl Chip {
    // Create new chip
    pub fn default(name: &'static str, animal: Animal, team: Team) -> Self {
        Chip { name, animal, team }
    }
}

// Player struct to keep track of hitpoints (number of bee edges that are untouched) and team
// It's currently unused, and might end up being superfluous, but will keep for now
#[derive(Debug, Clone)]
pub struct Player {
    hitpoints: u8,
    team: Team,
}

impl Player {
    // Create new player
    pub fn default(team: Team) -> Self {
        Player { hitpoints: 6, team }
    }
}

pub struct Board {
    pub chips: HashMap<Chip, Option<(i8, i8, i8)>>,
    turns: u32, // tracks number of turns that have elapsed
}

impl Board {
    // At new game, initialise all of the chips for each team with position = None (in player hand)
    pub fn default() -> Self {
        let chips: HashMap<Chip, Option<(i8, i8, i8)>> = HashMap::from([
            // Black team's chips
            (Chip::default("s1", Animal::Spider, Team::Black), None),
            (Chip::default("s2", Animal::Spider, Team::Black), None),
            (Chip::default("s3", Animal::Spider, Team::Black), None),
            // White team's chips
            (Chip::default("s1", Animal::Spider, Team::White), None),
            (Chip::default("s2", Animal::Spider, Team::White), None),
            (Chip::default("s3", Animal::Spider, Team::White), None),
            (Chip::default("s4", Animal::Spider, Team::White), None),
        ]);
        Board { chips, turns: 0 }
    }

    // List all chips belonging to a given team. If team == None, then show both teams' chips
    pub fn list_chips(&self, team: Option<Team>) -> Vec<Chip> {
        let chip_iter = self.chips.clone().into_iter().map(|(c, _)| c);

        match team {
            Some(team) => chip_iter.filter(|c| c.team == team).collect::<Vec<Chip>>(),
            None => chip_iter.collect::<Vec<Chip>>(),
        }
    }

    // For now, this guy handles the MoveStatus enum and provides some printscreen feedback
    pub fn try_move(&mut self, name: &'static str, team: Team, position: (i8, i8, i8)) {
        match self.move_chip(name, team, position) {
            MoveStatus::Success => {
                println!("Chip move was successful."); 
                self.turns += 1;
                // TODO: and then we need to code some logic to switch the active player
            }
            MoveStatus::BadNeighbour => println!("Can't place chip next to other team."),
            MoveStatus::HiveSplit => println!("BAD BEE. This move would break my hive in twain."),
            MoveStatus::Occupied => println!("Can't place chip in occupied position."),
            MoveStatus::Unconnected => println!("Can't move chip to middle of nowhere."),
        }
    }

    // Try move a chip of given name / team, to a new position. Return MoveStatus to tell the main loop how successful the attempt was.
    fn move_chip(
        &mut self,
        name: &'static str,
        team: Team,
        position: (i8, i8, i8),
    ) -> MoveStatus {
        let animal = Board::get_animal(name); // Get the chip's animal based on its name
        let chip_select = Chip::default(name, animal, team); // Select the chip

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

    // Figure out what animal a chip is based on the first char in its name
    // This is clunky and I don't like it. It sort of renders the Animal enum as pointless, but it works for now.
    fn get_animal(name: &str) -> Animal {
        let animal;
        match name.chars().next() {
            Some(character) => {
                animal = match character {
                    'a' => Animal::Ant,
                    's' => Animal::Spider,
                    'q' => Animal::Bee,
                    'b' => Animal::Beetle,
                    'g' => Animal::Grasshopper,
                    'l' => Animal::Ladybird,
                    'm' => Animal::Mosquito,
                    _ => panic!("The chip's name field doesn't correspond to a known animal."),
                }
            }
            None => panic!("Chip has an invalid name field."),
        }
        animal
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
        let neighbour_hex = Board::neighbour_tiles(destination);

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
        // Two constraints for a relocation:
        // Constraint 1) chip relocate cannot break the hive in two;
        // Constraint 2) chip must end up adjacent to another tile (or on top of one if its a beetle, but we'll worry about this later)

        // Does moving the chip away from current position cause the hive to split?
        let constraint1 = self.hive_break_check(current_position);

        // Get hexes that neighbour the desired destination hex (a',r',c')
        let neighbour_hex = Board::neighbour_tiles(destination);

        // Do we have at least one neighbour at the destination?
        let constraint2 = !neighbour_hex
            .into_iter()
            .map(|p| self.get_team(p))
            .any(|t| t.is_some());

        if constraint1 {
            MoveStatus::HiveSplit
        } else if constraint2 {
            MoveStatus::Unconnected
        } else {
            self.chips.insert(chip, Some(destination)); // Overwrite the chip's position in the HashMap
            MoveStatus::Success
        }
    }

    // Check if moving a chip out of the current position splits the hive
    fn hive_break_check(&self, current_position: &(i8, i8, i8)) -> bool {
        // To achieve this, we need to do some connected component labelling.
        // A "one-component-at-a-time" algorithm is one of the simplest ways to find connected components in a grid.
        // More info: https://en.wikipedia.org/wiki/Connected-component_labeling?oldformat=true#Pseudocode_for_the_one-component-at-a-time_algorithm

        // Create an empty hash set to store the locations of all chips on the board that neighbour at least one other chip
        // Why use a hash set? Because hash sets can't store duplicates: even if a chip neighbours several other chips, it only appears once in the hash set record
        let mut store: HashSet<(i8, i8, i8)> = HashSet::new();

        // Get the positions of chips on the board as a flat sorted vector (i.e. raster scan the board)
        let mut flat_vec = self.rasterscan_board();

        // Remove the chip at our "current_position" from our flat vector. This simulates moving the chip somewhere else.
        // I've thought about it for  whole 3 minutes, and I think there is no situation in which a hive can be broken and reconnected on the same turn.
        // If I'm wrong then we'll need to pass this function a destination hex and add it in
        flat_vec.retain(|&p| p != *current_position);

        // For each element in the raster scan
        for position in flat_vec.clone() {
            // Get the co-ordinates of neighbouring hexes as a vector
            let neighbour_hexes = Board::neighbour_tiles(position);
            let neighbour_vec = neighbour_hexes.into_iter().collect::<Vec<(i8, i8, i8)>>();

            // If any of these neighbouring hex co-ordinates also appear in the remaining elements of the raster scan, it means they're a neighbouring chip
            // We'll store all neighbouring chip co-ordinates in that "store" hashset that we defined earlier
            for elem in neighbour_vec.iter() {
                for elem2 in flat_vec.clone().iter() {
                    if elem == elem2 {
                        store.insert(*elem2);
                    }
                }
            }

            // We're done with checking this chip's neighbours, so delete it from the raster scan queue and move on to the next element
            flat_vec.retain(|&p| p != position);
        }

        // The total elements in the final store hashset should be N-2 if the Hive has not broken in two
        // println!(
        //    "initial size was {}, and store len is {}",
        //    self.rasterscan_board().len(),
        //    store.len()
        //);

        store.len() != self.rasterscan_board().len() - 2
    }

    // Raster scan all chips on the board and returns their positions as a flat vector
    fn rasterscan_board(&self) -> Vec<(i8, i8, i8)> {
        // Flatten the board's HashMap into a vector that only counts chips on the board (i.e. p.is_some())
        let mut flat_vec = self
            .chips
            .iter()
            .filter(|(_, p)| p.is_some())
            .map(|(_, p)| p.unwrap())
            .collect::<Vec<(i8, i8, i8)>>();

        // Sort the hex co-ordinates (a,r,c) by:
        // r descending first
        // then a descending
        // then c ascending
        // This is equivalent to a raster scan in a HECS co-ordinate system
        flat_vec
            .sort_by(|(a1, r1, c1), (a2, r2, c2)| (r2, a2, c1).partial_cmp(&(r1, a1, c2)).unwrap());

        flat_vec
    }

    // Get co-ordinates of all chips that are already placed on the board
    fn get_placed(&self) -> Vec<(i8, i8, i8)> {
        self.chips
            .values()
            .filter(|p| p.is_some())
            .map(|p| p.unwrap())
            .collect()
    }

    // Get HECS co-ordinates of the 6 neighbouring tiles
    fn neighbour_tiles(position: (i8, i8, i8)) -> [(i8, i8, i8); 6] {
        let (a, r, c) = position;

        [
            (1 - a, r - (1 - a), c - (1 - a)),
            (1 - a, r - (1 - a), c + a),
            (a, r, c - 1),
            (a, r, c + 1),
            (1 - a, r + a, c - (1 - a)),
            (1 - a, r + a, c + a),
        ]
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

    // Return the info on the Chip that is at a given location
    fn get_chip(&self, position: (i8, i8, i8)) -> Option<Chip> {
        self.chips
            .iter()
            .find_map(|(c, p)| if *p == Some(position) { Some(*c) } else { None })
    }
}
