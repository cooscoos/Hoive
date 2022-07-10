use std::{collections::HashMap, hash::Hash, collections::HashSet};


#[derive(Hash, Eq, PartialEq, Debug, Clone, Copy)]
pub enum Team {
    Black,
    White,
}

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

#[derive(Hash, Eq, PartialEq, Debug, Clone, Copy)]
pub struct Chip {
    name: &'static str,
    animal: Animal,
    team: Team,
}

impl Chip {
    pub fn default(name: &'static str, animal: Animal, team: Team) -> Self {
        Chip { name, animal, team }
    }
}

#[derive(Debug, Clone)]
pub struct Player {
    hitpoints: u8,
    team: Team,
}

impl Player {
    pub fn default(team: Team) -> Self {
        Player { hitpoints: 6, team }
    }
}

pub struct Board {
    pub chips: HashMap<Chip, Option<(i8, i8, i8)>>,
    turns: u32, // tracks the number of turns that have elapsed
}

impl Board {
    // Initialise all of the board pieces with position = None (in player hand)
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

    // Move a chip of given name / team, to a new position
    pub fn move_chip(&mut self, name: &'static str, team: Team, position: (i8, i8, i8)) {
        let animal = Board::get_animal(name); // Get the chip's animal based on its name
        let chip_select = Chip::default(name, animal, team); // Select that chip

        // Current position of chip tells us if we're placing chip from player's hand, or relocating it on the board
        match self.chips.clone().get(&chip_select) {
            Some(p) => {
                match p {
                    Some(current_position) => {
                        self.relocate_chip(chip_select, position, current_position)
                    } // chip has a position, so we're relocating it
                    None => {
                        // chip's position == None (player hand), so we're placing it
                        self.place_chip(chip_select, team, position);
                    }
                }
            }
            None => panic!("Chip does not exist"),
        }
        self.turns += 1;
    }

    // Figure out what animal a chip is based on the first char in its name
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
                    _ => panic!("Unknown chip"),
                }
            }
            None => panic!("Invalid chip name"),
        }
        animal
    }

    // Move chip from player's hand to the board at selected position
    fn place_chip(&mut self, chip: Chip, team: Team, new_position: (i8, i8, i8)) {
        // Two constraints for placement of new chip:
        // Constraint 1) it can't be placed on top of another chip, and;
        // Constraint 2) it must have at least one neighbour (after turn 1)
        // Constraint 3) its neighbours must be the same team (after turn 2)

        // Any chips already on board at given position?
        let constraint1 = self.get_placed().iter().any(|p| *p == new_position);

        let neighbour_hex = Board::neighbour_tiles(new_position);

        // Do we have at least one neighbour?
        let constraint2 = !neighbour_hex
            .into_iter()
            .map(|p| self.get_team(p))
            .any(|t| t.is_some());

        // Are all chips neighbouring given position not on the same team as you?
        let constraint3 = !neighbour_hex
            .into_iter()
            .map(|p| self.get_team(p))
            .filter(|t| t.is_some())
            .all(|t| t.unwrap() == team);

        if constraint1 {
            println!("Can't place chip in occupied position.");
        } else if self.turns >= 1 && constraint2 {
            println!("Can't place chip middle of nowhere");
        } else if self.turns >= 2 && constraint3 {
            println!("Can't place chip next to other team");
        } else {
            self.chips.insert(chip, Some(new_position)); // Overwrite the chip's position in the HashMap
        }
    }

    fn relocate_chip(
        &mut self,
        chip: Chip,
        new_position: (i8, i8, i8),
        current_position: &(i8, i8, i8),
    ) {
        // Constraints for a relocation:
        // Constraint 1) it cannot break the hive in two
        // Constraint 2) it must end up adjacent to another tile (or on top of one if beetle)


        // Constraint1

        let constraint1 = self.constraint1(current_position);
        
        // Constraint2
        let neighbour_hex = Board::neighbour_tiles(new_position);

        // Do we have at least one neighbour in new desired position?
        let constraint2 = !neighbour_hex
            .into_iter()
            .map(|p| self.get_team(p))
            .any(|t| t.is_some());

        if constraint1 {
            println!("This move would break the hive in two");
        } else if constraint2 {
            println!("Can't move chip middle of nowhere");
        }
    }

    fn constraint1(&self, current_position: &(i8, i8, i8)) -> bool {
        // create an empty hash set to store the locations
        let mut store: HashSet<(i8,i8,i8)> = HashSet::new();

        // Get the positions of chips on the  board as a flat sorted vector (raster scan)
        let mut flat_vec = self.rasterscan_board();

        
        // Remove the current_position from the flat vector to simulate the move
        flat_vec.retain(|&p| p!=*current_position);

        // for each element in flat_vec


        for position in flat_vec.clone() {
            // find the neighbours
            let neighbour_check = Board::neighbour_tiles(position);

            // convert to vector
            let neighbour_vec = neighbour_check.into_iter().collect::<Vec<(i8,i8,i8)>>();

   
            // if there are neighbours that appear in flat_vec then add them to the store hashset
            for elem in neighbour_vec.iter() {
                for elem2 in flat_vec.clone().iter() {
                    if elem == elem2 {
                        store.insert(*elem2);
                    }
                }
            }

            // remove the point we just checked
            flat_vec.retain(|&p| p!=position);
            
        }

        // the total elements in the store hashset should be N-2 if the hive has not broken in two
        println!("initial size was {}, and store len is {}",self.rasterscan_board().len(),store.len());
       
        store.len() != self.rasterscan_board().len() - 2

        
    }

    // Raster scans all chips on the board and returns their positions as a flat vector
    fn rasterscan_board(&self) -> Vec<(i8,i8,i8)> {
        // flatten existing HashMap into a vector for the chips that are on the board only (p.is_some()), (a,r,c)
        let mut flat_vec = self.chips.iter().filter(|(_,p)| p.is_some()).map(|(_,p)| p.unwrap()).collect::<Vec<(i8,i8,i8)>>();

        // I think this does it, sorting (a,r,c) as by r descending first, then a descending, then c ascending
        flat_vec.sort_by(|(a1,r1,c1),(a2,r2,c2)| (r2,a2,c1).partial_cmp(&(r1,a1,c2)).unwrap());
        flat_vec
    }

    // get co-ordinates of all chips that are already placed on the board
    fn get_placed(&self) -> Vec<(i8, i8, i8)> {
        self.chips
            .values()
            .filter(|p| p.is_some())
            .map(|p| p.unwrap())
            .collect()
    }

    // get HECS co-ordinates of the 6 neighbouring tiles
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

    // get the Team of chip at given position
    fn get_team(&self, position: (i8, i8, i8)) -> Option<Team> {
        self.chips.iter().find_map(|(c, p)| {
            if *p == Some(position) {
                Some(c.team)
            } else {
                None
            }
        })
    }

    // Return the chip at given location
    fn get_chip(&self, position: (i8, i8, i8)) -> Option<Chip> {
        self.chips
            .iter()
            .find_map(|(c, p)| if *p == Some(position) { Some(*c) } else { None })
    }
}
