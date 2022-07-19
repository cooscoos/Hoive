// Components of a game: the two teams and chips/tiles

// Enum for two teams
#[derive(Hash, Eq, PartialEq, Debug, Clone, Copy)]
pub enum Team {
    Black,
    White,
}

// Probably a better way of doing this, but this works.
// Tell me who the other team are
pub fn other_team(team: Team) -> Team {
    match team {
        Team::Black => Team::White,
        Team::White => Team::Black,
    }
}

// Struct for tiles/chips in a game
#[derive(Hash, Eq, PartialEq, Debug, Clone, Copy)]
pub struct Chip {
    pub name: &'static str, // identifies a unique animal/number, e.g. a2, s1 = ant 2, spider 1
    pub team: Team,         // black or white chip
}

impl Chip {
    // Create new chip
    pub fn new(name: &'static str, team: Team) -> Self {
        Chip { name, team }
    }
}

// Player struct to keep track of hitpoints (number of bee edges that are untouched) and team
// It's currently unused, and will likely end up being superfluous
// As bee edge touches can be tracked by the Board struct
// #[derive(Debug, Clone)]
// pub struct Player {
//     _hitpoints: u8,
//     _team: Team,
// }

// impl Player {
//     // Create new player
//     pub fn default(team: Team) -> Self {
//         Player {
//             _hitpoints: 6,
//             _team: team,
//         }
//     }
// }
