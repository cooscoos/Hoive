/// This module converts HttpRequests into commands that execute gameplay and database updates.
use std::result::Result;

// Profanity filter for usernames, and random number / uuid generation
use rand::Rng;

use actix_web::{error, web, Error, HttpRequest, HttpResponse};
use actix_web_actors::ws;

pub use crate::db;
use crate::models::GameState;
pub use crate::models::{self, User};
pub use crate::schema;

use diesel::r2d2::{ConnectionManager, Pool, PooledConnection};
use diesel::SqliteConnection;

// Game modules
use hoive::game::{
    actions::BoardAction, board::Board, comps::Team, history::Event, movestatus::MoveStatus,
    specials,
};
use hoive::maths::coord::Coord;
use hoive::maths::coord::Cube;

use crate::{game_server, game_session};
use actix::Addr;
use std::time::Instant;

/// Entry point for our websocket route
/// Define the username, check it for profanity, register it on the db, and in the chat.
/// Auto join the main lobby
pub async fn chat_route(
    //form_input: web::Form<User>,
    req: HttpRequest,
    stream: web::Payload,
    srv: web::Data<Addr<game_server::GameServer>>,
    pool: web::Data<Pool<ConnectionManager<SqliteConnection>>>
) -> Result<HttpResponse, Error> {

    // start the websocket
    ws::start(
        game_session::WsGameSession {
            id: 0,
            hb: Instant::now(),
            room: "main".to_owned(),
            name: None,
            active: false,
            action: BoardAction::default(),
            board: Board::<Cube>::default(),
            team: Team::Black,
            addr: srv.get_ref().clone(),
            pool: pool.get_ref().clone(),
        },
        &req,
        stream,
    )

}

/// Get a connection to the db
fn get_db_connection(
    req: HttpRequest,
) -> Result<PooledConnection<ConnectionManager<SqliteConnection>>, Error> {
    if let Some(pool) = req.app_data::<Pool<ConnectionManager<SqliteConnection>>>() {
        match pool.get() {
            Ok(conn) => Ok(conn),
            Err(error) => Err(error::ErrorBadGateway(error)), // convert error into actix-web error
        }
    } else {
        Err(error::ErrorBadGateway(
            "[api][get_db_connection] Can't get db connection",
        ))
    }
}

/// Default index page that shows the Hoive server version
pub async fn index() -> HttpResponse {
    HttpResponse::Ok().body(format!("Hoive-server v{}", crate::VERSION))
}

/// Register a new user with requested name (input via web form)
pub fn register_user(user_name: &str, session_id: usize, pool: &mut Pool<ConnectionManager<SqliteConnection>>) -> Result<bool, Error> {

    let mut conn = pool.get().unwrap();

    // Check if the db contains a user with this name already
    match db::username_available(user_name, &mut conn) {
        Ok(true) => {}
        Ok(false) => return Ok(false),
        Err(error) => {
            return Err(error::ErrorBadGateway(format!(
                "Cant access db to check for usernames: {error}"
            )))
        }
    }

    match db::create_user(user_name, &mut conn, session_id) {
        Ok(user_id) => {
            //session.insert(USER_ID_KEY, user_id.to_string())?;
            println!(
                "\x1b[32;1mUsername {} registered under {}\x1b[0m\n",
                user_name, user_id
            );
            Ok(true)
        }
        Err(error) => Err(error::ErrorBadGateway(format!(
            "Cant register new user: {error}"
        ))),
    }
}

/// Delete the user from the db
pub fn deregister_user(user_id: &usize) -> Result<(), Error> {
    // This happens infrequently enough that it's easier to ignore pool and establish a new connection
    let mut conn = db::establish_connection();

    match db::remove_user(&user_id.to_string(), &mut conn) {
        Ok(_) => Ok(()),
        Err(error) => Err(error::ErrorBadGateway(format!(
            "Can't deregister user: {error}"
        ))),
    }
}

/// Delete the game from the db
pub fn deregister_game(session_id: &str) -> Result<(), Error> {
    // This happens infrequently enough that it's easier to ignore pool and establish a new connection
    let mut conn = db::establish_connection();

    match db::remove_game(session_id, &mut conn) {
        Ok(_) => Ok(()),
        Err(error) => Err(error::ErrorBadGateway(format!(
            "Can't delete game: {error}"
        ))),
    }
}

/// Get user name based on an input user id
pub async fn get_username(user_id: &str, req: HttpRequest) -> Result<HttpResponse, Error> {

    //let mut conn = db::establish_connection();
    let mut conn = get_db_connection(req)?;

    match db::get_user_name(&user_id, &mut conn) {
        Ok(username) => Ok(HttpResponse::Ok().body(username)),
        Err(err) => Err(error::ErrorBadGateway(format!(
            "Cant find username for given user id because {err}"
        ))),
    }
}

/// Get user name based on an input user id
pub fn is_user_dead(user_id: &str, pool: &mut Pool<ConnectionManager<SqliteConnection>>) -> Result<bool, Error> {

    let mut conn = pool.get().unwrap();

    match db::is_user_dead(user_id, &mut conn) {
        Ok(result) => Ok(result),
        Err(err) => Err(error::ErrorBadGateway(format!(
            "Error finding if user is active because: {err}"
        ))),
    }
}


/// Create a new game
pub fn new_game(user_id: &usize, pool: &mut Pool<ConnectionManager<SqliteConnection>>) -> Result<String, Error> {
    
    let mut conn = pool.get().unwrap();

    match db::create_session(user_id, &mut conn) {
        Ok(session_id) => {
            println!("\x1b[32;1mCreated session id {}\x1b[0m\n", session_id);
            Ok(session_id.to_string())
        }
        Err(error) => Err(error::ErrorBadGateway(format!(
            "Cant register new session: {error}"
        ))),
    }
}

/// Find a live session without a player 2
pub fn find(pool: &mut Pool<ConnectionManager<SqliteConnection>>) -> Result<Option<GameState>, Error> {
    
    let mut conn = pool.get().unwrap();
    
    match db::find_live_session(&mut conn) {
        Some(game_state) => Ok(Some(game_state)),
        None => Ok(None),
    }
}

/// Join a session with given session_id
pub fn join(session_id: &str, user_2_id: &usize, pool: &mut Pool<ConnectionManager<SqliteConnection>>) -> Result<(), Error> {

    let mut conn = pool.get().unwrap();

    match db::join_live_session(session_id, user_2_id, &mut conn) {
        Ok(0) => Err(error::ErrorNotFound(format!(
            "No waiting sessions with id {session_id}"
        ))),
        Ok(1) => {
            println!("\x1b[32;1mUser joined successfully\x1b[0m\n");

            // Get the game state so we can retrieve user ids
            let game_state = match db::get_game_state(session_id, &mut conn) {
                Ok(value) => value,
                Err(err) => {
                    return Err(error::ErrorNotFound(format!(
                        "Could not load gamestate from {session_id} because {err}"
                    )))
                }
            };

            // Toss a coin to see who goes first
            let mut rand = rand::thread_rng();
            let l_user = match rand.gen() {
                true => game_state.user_1.unwrap(),  // User 1 is on team Black
                false => game_state.user_2.unwrap(), // User 2 is on team White
            };

            // Update the db and return ok
            match db::update_game_state(session_id, &l_user, "", "", &mut conn) {
                Ok(_) => Ok(()),
                Err(err) => Err(error::ErrorInternalServerError(format!(
                    "Can't update game state of {session_id} because {err}"
                ))),
            }
        }
        Ok(_) => Err(error::ErrorBadGateway("Multiple sessions updated")),
        Err(error) => Err(error::ErrorBadGateway(format!(
            "Cant join session: {}",
            error
        ))),
    }
}

/// Retrieve the game state of a session
pub fn get_game_state(session_id: &str, pool: &mut Pool<ConnectionManager<SqliteConnection>>) -> Result<GameState, Error> {

    let mut conn = pool.get().unwrap();

    let res = db::get_game_state(session_id, &mut conn);
    match res {
        Ok(game_state) => Ok(game_state),
        _ => Err(error::ErrorInternalServerError(format!(
            "Can't find game with session id {session_id}"
        ))),
    }
}

/// Allow player to take some sort of action
pub fn make_action(action: &BoardAction, session_id: &str, pool: &mut Pool<ConnectionManager<SqliteConnection>>) -> Result<MoveStatus, Error> {

    // Retrieve the game_state
    let game_state = get_game_state(session_id, pool)?;

    let mut conn = pool.get().unwrap();

    // Find out if we have a special player action.
    match action.special.as_ref() {
        Some(special) if special.starts_with("forfeit") => {

            // Get the forfeitting player's id
            let v: Vec<&str> = special.splitn(2, ';').collect();
            let loser_id = v[1];

            // Try and make the loser forfeit
            forfeit(game_state, session_id, loser_id, &mut conn)
        }
        Some(special) if special == "skip" => {
            // Try and skip the current player's turn
            skip_turn(game_state, session_id, &mut conn)
        }
        Some(_) => {
            // Any other special string is a pillbug / mosquito action
            do_special(game_state, action, session_id, &mut conn)
        }
        None => {
            // Otherwise it's a normal move
            do_movement(game_state, action, session_id, &mut conn)
        }
    }
}

/// Try and execute movement
fn do_movement(
    game_state: GameState,
    action: &BoardAction,
    session_id: &str,
    conn: &mut SqliteConnection,
) -> Result<MoveStatus, Error> {
    // Generate a board based on the gamestate and find the chip name and active team
    let mut board = game_state.to_cube_board();

    let active_team = game_state.which_team()?;
    let chip_name = action.get_chip_name();
    //assert!(cheat_check(&action, &active_team));

    // Convert from doubleheight to the board's co-ordinate system
    let position = board.coord.mapfrom_doubleheight(action.rowcol.unwrap());

    // Try and do the move, see what happens. If it's successful the board struct will update itself
    let move_status = board.move_chip(chip_name, active_team, position);

    // Create an event to track history of moves
    let event = Event::new_by_action(action);

    match move_status {
        MoveStatus::Success => execute_on_db(&mut board, game_state, event, session_id, conn)?,
        MoveStatus::Win(team) => {
            execute_win_on_db(team, &mut board, game_state, event, session_id, conn)?
        }
        _ => {}
    };
    Ok(move_status)
}

/// Try and execute a chip special
fn do_special(
    game_state: GameState,
    action: &BoardAction,
    session_id: &str,
    conn: &mut SqliteConnection,
) -> Result<MoveStatus, Error> {
    // Generate a board based on the gamestate and find the chip name and active team
    let mut board = game_state.to_cube_board();

    let active_team = game_state.which_team()?;

    // Try and decode and execute the special
    let move_status = hoive::pmoore::decode_specials(
        &mut board,
        &action.get_special(),
        active_team,
        action.get_chip_name(),
        action.rowcol.unwrap(),
    );

    // Create an event to track history of moves
    let event = Event::new_by_action(action);

    // Execute it on the db if it was successful
    match move_status {
        MoveStatus::Success => execute_on_db(&mut board, game_state, event, session_id, conn)?,
        MoveStatus::Win(team) => {
            execute_win_on_db(team, &mut board, game_state, event, session_id, conn)?
        }
        _ => {}
    };

    Ok(move_status)
}

/// Execute a successful action on the db
fn execute_on_db<T: Coord>(
    board: &mut Board<T>,
    game_state: GameState,
    event: Event,
    session_id: &str,
    conn: &mut SqliteConnection,
) -> Result<(), Error> {
    // Refresh all mosquito names back to m1 and update board on server
    specials::mosquito_desuck(board);
    let board_str = board.encode_spiral();

    // Get the uuid of the current user and set them as the last_user in the db
    let l_user = game_state.which_user()?;

    // Parse the event into a string and append it to the board's history
    let history = game_state.add_event(event);

    let res = db::update_game_state(session_id, &l_user, &board_str, &history, conn);

    match res {
        Ok(_) => Ok(()),
        Err(err) => Err(error::ErrorInternalServerError(format!(
            "Problem updating gamestate because {err}"
        ))),
    }
}

/// Execute a winning action on the db
fn execute_win_on_db<T: Coord>(
    winner: Option<Team>, // winning team
    board: &mut Board<T>,
    game_state: GameState,
    event: Event,
    session_id: &str,
    conn: &mut SqliteConnection,
) -> Result<(), Error> {
    let win_string = match winner {
        Some(winner_team) => {
            // Team black are user1. This is a bit jank. Need a cleverer way
            let winner_id = match winner_team {
                Team::Black => game_state
                    .user_1
                    .as_ref()
                    .unwrap()
,
                Team::White => game_state
                    .user_2
                    .as_ref()
                    .unwrap()
,
            };

            // Get the winner's name
            let winner_name = match db::get_user_name(&winner_id, conn) {
                Ok(value) => value,
                Err(err) => {
                    return Err(error::ErrorInternalServerError(format!(
                        "Problem getting winner name from user id because {err}"
                    )))
                }
            };

            // Create a winstring
            format!("{},{}", winner_team.to_string(), winner_name)
        }
        // Otherwise draw
        None => "D".to_string(),
    };

    // Refresh all mosquito names back to m1 and update board on server
    specials::mosquito_desuck(board);
    let board_str = board.encode_spiral();

    // Get the uuid of the current user and set them as the last_user in the db
    let l_user = game_state.which_user()?;

    // Parse the event into a string and append it to the board's history
    let history = game_state.add_event(event);

    // Update db
    let res =
        db::update_game_and_winner(session_id, &l_user, &board_str, &history, &win_string, conn);

    match res {
        Ok(_) => Ok(()),
        Err(err) => Err(error::ErrorInternalServerError(format!(
            "Problem updating winner in gamestate because {err}"
        ))),
    }
}

fn skip_turn(
    game_state: GameState,
    session_id: &str,
    conn: &mut SqliteConnection,
) -> Result<MoveStatus, Error> {
    // Get the board and current user
    let mut board = game_state.to_cube_board();

    let l_user = game_state.which_user()?;
    let active_team = game_state.which_team()?;

    // Parse the event into a string and append it to the board's history
    let history = game_state.add_event(Event::skip_turn(active_team));

    // Try skip the turn
    match board.try_skip_turn(active_team) {
        MoveStatus::Success => {
            // encode the board as a string (to capture the skip turn)
            let board_str = board.encode_spiral();

            // Do skip, change the active team in the db
            match db::update_game_state(session_id, &l_user, &board_str, &history, conn) {
                Ok(_) => Ok(MoveStatus::Success),
                Err(err) => Err(error::ErrorInternalServerError(err)),
            }
        }
        MoveStatus::NoSkip => Ok(MoveStatus::NoSkip),
        _ => unreachable!(),
    }
}

fn forfeit(
    game_state: GameState,
    session_id: &str,
    loser_id: &str,
    conn: &mut SqliteConnection,
) -> Result<MoveStatus, Error> {


    let winner_id = game_state.not_this_user(loser_id)?;
    let winner_team = game_state.which_team_user(&winner_id)?;

    // // The winner is the team who didn't forfeit (the inactive team)
    // let winner_team = !game_state.which_team()?;
    // let winner_id = game_state
    //     .inactive_user()?
    //     .parse::<usize>()
    //     .expect("Couldn't parse user id to usize");

    // Get the winner's name
    let winner_name = match db::get_user_name(&winner_id, conn) {
        Ok(value) => value,
        Err(err) => {
            return Err(error::ErrorInternalServerError(format!(
                "Problem getting winner name from user id because {err}"
            )))
        }
    };

    // Append F to to designate the reason for winning as a forfeit
    let win_string = format!("{},{},F", winner_team.to_string(), winner_name);

    // Update the last user id to the person who forfeit (the active team)
    let l_user_id = game_state.which_user()?;

    // Update db
    let res = db::update_winner(session_id, &l_user_id, &win_string, conn);

    match res {
        Ok(_) => Ok(MoveStatus::Win(Some(winner_team))),
        Err(err) => Err(error::ErrorInternalServerError(format!(
            "Problem updating winner in gamestate because {err}"
        ))),
    }
}

/// For debugging only. Delete the db on the server
pub fn delete_all(pool: &mut Pool<ConnectionManager<SqliteConnection>>) -> Result<HttpResponse, Error> {
    let mut conn = pool.get().unwrap();

    db::clean_db(&mut conn);
    println!("Database cleared");

    Ok(HttpResponse::Ok().body("Cleared"))
}

/// For debugging, return list of all users and game ids
pub fn get_all(pool: &mut Pool<ConnectionManager<SqliteConnection>>) -> Result<String, Error> {

    let mut conn = pool.get().unwrap();

    match db::get_all(&mut conn) {
        Ok(result) => Ok(result
            .into_iter()
            .map(|v| format!("{v}\n"))
            .collect::<String>()),
        Err(err) => Err(error::ErrorInternalServerError(format!(
            "Couldn't retrieve from db because: {}",
            err
        ))),
    }
}
