// Series of tests for manipulating the db

use actix_web::Error;
use actix_web::error::ErrorBadGateway;

use uuid::Uuid;

use diesel::SqliteConnection;
use diesel::r2d2::PooledConnection;
use diesel::r2d2::ConnectionManager;

use server::db::*;
use server::models::GameState;

/// Update the gamestate of a live session with some garbage for tests. If clean == true then it wipes db first
fn test_server_garbage(clean: bool) -> Result<(Uuid, Uuid, PooledConnection<ConnectionManager<SqliteConnection>>), Error> {

    let mut con = create_conn_pool().get().unwrap();

    // Wipe the db clean
    if clean {
        clean_db(&mut con);
    }

    // Invent a user and create a session
    let user_id = Uuid::new_v4();
    let session_id = create_session(&user_id, &mut con).unwrap();

    // Update the gamestate with some garbage values
    match update_game_state(&session_id, &user_id.to_string(), "board_test", "history_test", &mut con) {
        Ok(_) => Ok((session_id, user_id, con)),
        Err(err) => Err(ErrorBadGateway(err)),
    }
}


#[test]
fn server_get_board() {

    // Test getting a board

    // Update the game_state with some garbage test values
    let (session_id,_, mut con) = match test_server_garbage(true) {
        Ok(vals) => vals,
        Err(err) => panic!("Problem {err}"),
    };

    // Get the board string, it should read "board_test"
    let board = get_board(&session_id, &mut con).unwrap();
    assert_eq!(board, "board_test");
}

#[test]
fn server_get_gamestate() {

    // Test getting gamestate

    // Update the game_state with some garbage test values
    let (session_id, user_id, mut con) = match test_server_garbage(true) {
        Ok(vals) => vals,
        Err(err) => panic!("Problem {err}"),
    };

    // Get the game_state string
    let gamestate = get_game_state(&session_id, &mut con).unwrap();

    // The retrieved value should be identical to this
    let expected = GameState{
        id: session_id.to_string(),
        board: Some("board_test".to_string()),
        user_1: Some(user_id.to_string()),
        user_2: None,
        winner: None,
        last_user_id: Some(user_id.to_string()),
        history:Some("history_test".to_string()),
    };

    assert_eq!(gamestate, expected);
}

#[test]
fn server_get_username() {

    // Test getting a username

    let mut con = create_conn_pool().get().unwrap();

    // Create a user with name piggy
    let user_id = match create_user("piggy", &mut con) {
        Ok(value) => value,
        Err(err) => panic!("Error: {err}"),
    };

    // Fetch the username, it should read "piggy"
    let username = get_user_name(&user_id, &mut con).unwrap();
    assert_eq!(username, "piggy");
}

#[test]
fn server_find_session() {

    // Test finding a live session

    // Create a sole new session on db
    let (session_id, _, mut con) = match test_server_garbage(true) {
        Ok(vals) => vals,
        Err(err) => panic!("Problem {err}"),
    };

    // Find a new session, get the gamestate
    let found_session = find_live_session(&mut con).unwrap();

    assert_eq!(session_id.to_string(), found_session.id);
}

#[test]
fn server_find_multi_session() {

    // Test finding one of many live sessions

    // Create a two new sessions on db
    let (session_id_1, _, mut con) = match test_server_garbage(true) {
        Ok(vals) => vals,
        Err(err) => panic!("Problem {err}"),
    };

    // Create a second session without wiping the db
    let _result = test_server_garbage(false);

    // Find a new session, get the gamestate, this should return the older of the two sessions
    let found_session = find_live_session(&mut con).unwrap();

    assert_eq!(session_id_1.to_string(), found_session.id);
}

#[test]
fn server_update_winner() {

    // Test updating the winner of a game

    // Create a a new sessions on db
    let (session_id, _, mut con) = match test_server_garbage(true) {
        Ok(vals) => vals,
        Err(err) => panic!("Problem {err}"),
    };

    // Define a winner string
    let _result = update_winner(&session_id, "1", "BF", &mut con);

    // Get the game_state and make sure it's been updated
    let gamestate = get_game_state(&session_id, &mut con).unwrap();

    assert_eq!("BF", gamestate.winner.unwrap());
}