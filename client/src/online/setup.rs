use reqwest::Client;
/// Set up connections to a Hoive db and generate/join games
use std::{error::Error, thread, time::Duration};
use uuid::Uuid;

use super::comms;
use hoive::draw;
use hoive::game::comps::Team;
use hoive::pmoore::get_usr_input;
use server::models::GameState;

/// Return the address of a live Hoive server based on user inputs
pub async fn join_server() -> Result<(Client, String), Box<dyn Error>> {
    // Run the set up procedure until we succeed
    let mut server_info = None;
    while server_info.is_none() {
        server_info = Some(setup().await?);
    }

    Ok(server_info.unwrap())
}

/// Run user through prompts to attempt to join a Hoive server
pub async fn setup() -> Result<(Client, String), Box<dyn Error>> {
    println!("Select a server address (leave blank for default localhost):");
    let textin = get_usr_input();
    let address = match textin {
        _ if textin.is_empty() => "localhost".to_string(), // default
        _ => textin,
    };

    println!("Select a port (leave blank for default 8080):");
    let textin = get_usr_input();
    let port = match textin {
        _ if textin.is_empty() => "8080".to_string(), // default
        _ => textin,
    };

    // Create a base url that points to the Hoive server
    let base_url = format!("http://{address}:{port}/api/");

    // Get a persistent cookie for this session so we can easily match further requests to this user
    let client = reqwest::Client::builder().cookie_store(true).build()?;

    // Test the base url connects to a valid Hoive server
    comms::check_server(&client, &base_url).await?;

    Ok((client, base_url))
}

/// Ask the user to register a user name on the database
pub async fn register_user(client: &Client, base_url: &String) -> Result<String, Box<dyn Error>> {
    // Try to register the name on the server and get user uuid
    // Invalid is returned if the user name is something profane
    let mut user_id = "invalid".to_string();
    while user_id == *"invalid" {
        println!("Enter a nickname:");
        let username = get_usr_input();
        user_id = comms::register_user(username, client, base_url).await?;
    }

    Ok(user_id)
}

/// Create or join games, and then play them
pub async fn join_game(
    client: &Client,
    base_url: &String,
) -> Result<(GameState, Team, Team), Box<dyn Error>> {
    // Check whether there are any games available on the server
    // If there are you have to join the empty one (can change later to play with friends based on uid, private flag in db)
    let free_game = comms::find_game(client, base_url).await?;

    let mut game_state: GameState;

    let mut my_team = Team::Black; // make the player's team black, it'll change to white if they're player 2
    match free_game {
        _ if free_game == *"None" => {
            println!("No games seeking a player found. Creating a new game.");
            comms::new_game(client, base_url).await?;
            println!("Created! Waiting for second player to join...");
            game_state = comms::get_gamestate(client, base_url).await?;

            // Get the updated gamestate from the server every 3 seconds and check for user_2
            while game_state.user_2.is_none() {
                game_state = comms::get_gamestate(client, base_url).await?;
                thread::sleep(Duration::from_secs(3));
            }
            // Get the other player's name
            let opponent_name = comms::get_username(
                client,
                base_url,
                game_state.user_2.as_ref().expect("There's no player 2!"),
            )
            .await?;
            // Person who created game is always user_1 (team black)
            println!(
                "Player \x1b[35;1m{opponent_name}\x1b[0m joined the game!\n\nYou are on team {}\n",
                draw::team_string(my_team)
            );
        }
        _ => {
            println!("Found an open game to join.");
            comms::join_game(client, base_url, Uuid::parse_str(&free_game).unwrap()).await?;
            // Get the gamestate of the game you've joined
            game_state = comms::get_gamestate(client, base_url).await?;
            let opponent_name = comms::get_username(
                client,
                base_url,
                game_state.user_1.as_ref().expect("There's no player 2!"),
            )
            .await?;
            // Person joining is always user_2 (team white)
            my_team = Team::White;
            println!("Joined game with player \x1b[34;1m{opponent_name}\x1b[0m successfully!\n\nYou are on team {}\n", draw::team_string(my_team))
        }
    }

    // Find out who the db has selected to go first and tell the player
    let first_team = game_state.which_team()?;
    println!("Team {} goes first", draw::team_string(first_team));

    Ok((game_state, my_team, first_team))
}
