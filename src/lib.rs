#[derive(Debug, Clone, Copy)]
pub enum Team {
    Black,
    White,
}

#[derive(Debug)]
pub enum Animal {
    Ant,
    Spider,
    Bee,
    Beetle,
    Grasshopper,
    Ladybird,
    Mosquito,
}


#[derive(Debug)]
pub struct Piece {
    animal: Animal,
    team: Team,
    position: Option<(u8, u8, u8)>,
}

impl Piece {
    pub fn default(animal: Animal, team: Team) -> Self {
        Piece {
            animal,
            team,
            position: None,
        }
    }
}

pub struct Player {
    hitpoints: u8,
    pieces: Vec<Piece>,
    team: Team,
}

impl Player {
    pub fn default(team: Team) -> Self {
        // let's give new players two spiders and an ant
        let pieces = vec![
            Piece::default(Animal::Spider, team),
            Piece::default(Animal::Spider, team),
            Piece::default(Animal::Ant, team),
        ];

        Player {
            hitpoints: 6,
            pieces,
            team,
        }
    }

    // Return the peices the player has in their hand
    pub fn show_hand(self) -> Vec<Piece> {
        self.pieces
            .into_iter()
            .filter(|c| c.position.is_none())
            .collect::<Vec<Piece>>()
    }

    // Let the player place a piece
    pub fn place(mut self, piece: Piece, coord: (u8,u8,u8)) {
        
        

    }
}
