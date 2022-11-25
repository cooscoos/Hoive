/// Functions that communicate with a Hoive server, sending requests and getting responses
use std::error::Error;

use reqwest::{Client, StatusCode};

use hoive::game::actions::BoardAction;
use hoive::game::movestatus::MoveStatus;
use server::models::GameState;
use uuid::Uuid;

/// Check base_url to make sure it's an active Hoive server of same version as client.
pub async fn check_server(client: &Client, base_url: &str) -> Result<(), Box<dyn Error>> {
    // The Hoive client version string
    let client_version = format!("Hoive-server v{}", crate::VERSION);

    // Try and get a response from the server
    let body = client.get(base_url).send().await?;

    // The server version string
    let server_version = body.text().await?;

    match client_version == server_version {
        true => Ok(()),
        false => Err("server and client versions don't match.".into()),
    }
}

/// send a game action request to the server, return movestatus
pub async fn send_action(
    action: BoardAction,
    client: &Client,
    base_url: &String,
) -> Result<MoveStatus, Box<dyn Error>> {
    let url = format!("{base_url}do-action");
    let body = client.post(&url).json(&action).send().await?;
    let move_status = body.json::<MoveStatus>().await?;

    Ok(move_status)
}

/// Register a new user on the server's db
pub async fn register_user(
    username: String,
    client: &Client,
    base_url: &String,
) -> Result<String, Box<dyn Error>> {
    // id and user_color are unknown but the server's db has these fields so we need to include them
    let params = [
        ("id", String::new()),
        ("user_name", username),
        ("user_color", String::new()),
    ];
    let url = format!("{base_url}register");
    let body = client.post(&url).form(&params).send().await?;

    match body.status() {
        StatusCode::OK => {
            let user_id = body.json::<String>().await?;

            // Server will return "invalid" if the username doesn't pass a profanity check
            match user_id == *"invalid" {
                true => println!("Invalid username, try again."),
                false => println!("You have been granted user_id: {}", user_id),
            }
            Ok(user_id)
        }
        _ => panic!("Problem accessing url {} because {}", &url, body.status()),
    }
}

/// Get username based on input user id
pub async fn get_username(
    client: &Client,
    base_url: &String,
    user_id: &str,
) -> Result<String, Box<dyn Error>> {
    // username and user_color are unknown but the server's db has these fields so we need to include them
    let params = [
        ("id", user_id.to_owned()),
        ("user_name", String::new()),
        ("user_color", String::new()),
    ];
    let url = format!("{base_url}user-name");
    let body = client.post(&url).form(&params).send().await?;

    Ok(body.text().await?)
}

/// Ask server to find a game which does not have a second player and return session id
pub async fn find_game(client: &Client, base_url: &String) -> Result<String, Box<dyn Error>> {
    let url = format!("{base_url}find");
    let body = client.get(&url).send().await?;

    match body.status() {
        StatusCode::OK => {
            let session_id = body.json::<String>().await?;
            Ok(session_id)
        }
        _ => panic!("Problem accessing url {} because {}", &url, body.status()),
    }
}

/// Request a new game be created on the server
pub async fn new_game(client: &Client, base_url: &String) -> Result<String, Box<dyn Error>> {
    let url = format!("{base_url}new");
    let body = client.get(&url).send().await?;

    match body.status() {
        StatusCode::OK => {
            let session_id = body.json::<String>().await?;
            println!("You have been granted session_id: {}", session_id);
            Ok(session_id)
        }
        _ => panic!("Problem accessing url {} because {}", &url, body.status()),
    }
}

/// Join a game of given session id
pub async fn join_game(
    client: &Client,
    base_url: &String,
    session_id: Uuid,
) -> Result<(), Box<dyn Error>> {
    let params = [("id", session_id)];
    let url = format!("{base_url}join");

    let body = client.post(&url).form(&params).send().await?;

    match body.status() {
        StatusCode::OK => {
            println!("Joined success: {}", session_id);
            Ok(())
        }
        _ => {
            println!("Body: {:?}", &body);
            panic!("Problem accessing url {} because {}", &url, body.status());
        }
    }
}

/// Get the gamestate of the current session
pub async fn get_gamestate(
    client: &Client,
    base_url: &String,
) -> Result<GameState, Box<dyn Error>> {
    let url = format!("{base_url}game-state");
    let game_state = client.get(&url).send().await?.json::<GameState>().await?;
    Ok(game_state)
}

/// Ask the db to pick a random player to go first
pub async fn get_coin_toss(client: &Client, base_url: &String) -> Result<String, Box<dyn Error>> {
    let url = format!("{base_url}{}", "coin-toss");
    let response_string = client.get(&url).send().await?.text().await?;
    Ok(response_string)
}

/// Ask the server's db to wipe itself
pub async fn wipe_db(client: &Client, base_url: &String) -> Result<(), Box<dyn Error>> {
    let url = format!("{base_url}wipe");
    client.get(&url).send().await?;
    Ok(())
}
