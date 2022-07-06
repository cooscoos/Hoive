#[derive(Debug, Clone, Copy)]
pub enum Team {
    Black,
    White,
}

#[derive(Debug, Clone, Copy)]
pub enum Animal {
    Ant,
    Spider,
    Bee,
    Beetle,
    Grasshopper,
    Ladybird,
    Mosquito,
}

#[derive(Debug, Clone, Copy)]
pub struct Piece<'a> {
    name: &'a str,
    animal: Animal,
    team: Team,
    position: Option<(u8, u8, u8)>,
}

impl<'a> Piece<'a> {
    pub fn default(name: &'a str, animal: Animal, team: Team) -> Self {
        Piece {
            name,
            animal,
            team,
            position: None,
        }
    }
}

pub struct Player<'a> {
    hitpoints: u8,
    pieces: Vec<Piece<'a>>,
    team: Team,
}

impl<'a> Player<'a> {
    pub fn default(team: Team) -> Self {
        // let's give new players two spiders and an ant
        let pieces = vec![
            Piece::default("s1", Animal::Spider, team),
            Piece::default("s2", Animal::Spider, team),
            Piece::default("a1", Animal::Ant, team),
        ];

        Player {
            hitpoints: 6,
            pieces,
            team,
        }
    }

    // Return the peices the player has in their hand
    pub fn show_hand(self) -> Vec<Piece<'a>> {
        self.pieces
            .into_iter()
            .filter(|c| c.position.is_none())
            .collect::<Vec<Piece>>()
    }

    // Let the player place a piece
    pub fn place(&mut self, name: &str, coord: (u8, u8, u8)) {
        self.pieces
            .into_iter()
            .filter(|c| c.name == name)
            .for_each(|mut c| c.position = Some(coord));
    }
}
