use Hoive::{Animal, Piece, Player, Team};

fn main() {
    // Start with a 1 player game where we just place pieces correctly
    // Add in a second player to place pieces correctly
    // Then do movement

    // initialise a player
    let P1 = Player::default(Team::Black);

    // show the player's hand
    println!("{:?}", P1.show_hand());

    // let them place a piece at 0,0,0, use a command to show hand
    

}
