use std::collections::HashMap;

#[derive(Hash, Eq, PartialEq, Debug, Clone, Copy)]
pub enum Team {
    Black,
    White,
    Ghost, // Ghost is used for the first turn to overcome adjacent tile rules
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

    pub fn relocate(&mut self, new_position: (u8, u8, u8)) {
        //self.position = Some(new_position);
    }
}

#[derive(Debug, Clone)]
pub struct Player {
    hitpoints: u8,
    team: Team,
}

impl Player {
    pub fn default(team: Team) -> Self {
        // let's give new players two spiders and an ant
        let pieces = vec![
            Chip::default("s1", Animal::Spider, team),
            Chip::default("s2", Animal::Spider, team),
            Chip::default("a1", Animal::Ant, team),
        ];

        Player { hitpoints: 6, team }
    }

    // Return the peices the player has in their hand
    //pub fn show_hand(&self) -> Vec<Piece<'a>> {
    //    self.pieces
    //        .clone()
    //        .into_iter()
    //        .filter(|c| c.position.is_none())
    //        .collect::<Vec<Piece>>()
    //}

    // Show all the pieces the player owns (on board and hand)
    //pub fn show_all(&self) -> Vec<Piece<'a>> {
    //    self.pieces.clone()
    //}

    // Let the player place a piece
    //pub fn place(&mut self, name: &str, new_position: (u8, u8, u8)) {
    //    self.pieces.iter_mut().filter(|c| c.name==name).for_each(|c| c.relocate(new_position))
    //}
}

pub struct Board {
    pub chips: HashMap<Chip, Option<(u8, u8, u8)>>,
}

impl Board {
    // Initialise all of the board pieces with position = None (in player hand)
    pub fn default() -> Self {
        let chips: HashMap<Chip, Option<(u8, u8, u8)>> = HashMap::from([
            // Black team's chips
            (Chip::default("s1", Animal::Spider, Team::Black), None),
            (Chip::default("s2", Animal::Spider, Team::Black), None),
            // White team's chips
            (Chip::default("s1", Animal::Spider, Team::White), None),
            // The ghost chip, place adjacent to (0,0,0) in the first turn, and then deleted
            (
                Chip::default("ghost", Animal::Spider, Team::Ghost),
                Some((0, 0, 1)),
            ),
        ]);

        Board { chips }
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
    pub fn move_chip(&mut self, name: &'static str, team: Team, position: (u8, u8, u8)) {
        // Get the chip's animal based on its name
        let animal = Board::get_animal(name);

        // Select the right chip
        let chip_select = Chip::default(name, animal, team);

        // Use position of chip to decide whether we're placing the chip from the player's hand, or relocating it on the board
        match self.chips.get(&chip_select) {
            Some(p) => {
                match p {
                    Some(_) => self.relocate_chip(chip_select, position), // chip already has position, so we're relocating it
                    None => {
                        self.place_chip(chip_select, position);
                    } // chip's position == None (player hand), so we're placing it
                }
            }
            None => panic!("Chip does not exist"),
        }
    }

    fn get_animal(name: &str) -> Animal {
        // Figure out what animal the chip is based on the first char in its name
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

    // Place a given chip on the board at given position
    fn place_chip(&mut self, chip: Chip, position: (u8, u8, u8)) {
        // Two constraints for placement of new chip:
        // 1) can't be placed on top of another tile, and;
        // 2) must be adjacent to another tile that is either ghost, or own colour

        



        self.chips.insert(chip, Some(position)); // Overwrite the position in the HashMap
    }

    fn relocate_chip(&mut self, chip: Chip, position: (u8, u8, u8)) {
        // Constraints for a relocation:
        // constraint 1: it must end up adjacent to another tile (or on top of one if beetle)
        // constraint 2: it cannot break the hive in two
    }

    // get HECS co-ordinates of the 6 neighbouring tiles
    fn get_neighbours(position: (u8, u8, u8)) -> [(u8, u8, u8); 6] {
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
}
