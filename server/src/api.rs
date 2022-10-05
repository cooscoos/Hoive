/// This module converts HttpRequests into commands that execute gameplay and database updates.
use std::result::Result;

// Profanity filter for usernames, and random number / uuid generation
use rand::Rng;
use rustrict::CensorStr;
use uuid::Uuid;

use actix_session::Session;
use actix_web::Responder;
use actix_web::{error, web, Error, HttpRequest, HttpResponse};
use serde::Deserialize;

pub use crate::db;
use crate::models::GameState;
pub use crate::models::{self, User};
pub use crate::schema;

use diesel::r2d2::{ConnectionManager, Pool, PooledConnection};
use diesel::SqliteConnection;

const SESSION_ID_KEY: &str = "session_id";
const USER_ID_KEY: &str = "user_id";

// Game modules
use hoive::game::{
    actions::BoardAction, board::Board, comps::Team, history::Event, movestatus::MoveStatus,
    specials,
};
use hoive::maths::coord::Coord;

/// Defines web form to parse a game session's uuid
#[derive(Deserialize)]
pub struct SessionInfo {
    id: Uuid,
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
pub async fn register_user(
    form_input: web::Form<User>,
    session: Session,
    req: HttpRequest,
) -> Result<impl Responder, Error> {
    let user_name = form_input.user_name.to_owned();

    // Refuse to register if username is profanity
    if user_name.is_inappropriate() {
        return Ok(web::Json("invalid".to_string()));
    }

    println!("REQ: {:?}", req);

    let mut conn = get_db_connection(req)?;

    match db::create_user(&user_name, &mut conn) {
        Ok(user_id) => {
            session.insert(USER_ID_KEY, user_id.to_string())?;
            println!(
                "\x1b[32;1mUsername {} registered under {}\x1b[0m\n",
                user_name, user_id
            );
            Ok(web::Json(user_id.to_string()))
        }
        Err(error) => Err(error::ErrorBadGateway(format!(
            "Cant register new user: {error}"
        ))),
    }
}

/// Get user name based on an input user id
pub async fn get_username(
    form_input: web::Form<User>,
    req: HttpRequest,
) -> Result<HttpResponse, Error> {
    println!("REQ: {:?}", req);

    let user_id = Uuid::parse_str(&form_input.id).unwrap();
    let mut conn = get_db_connection(req)?;

    match db::get_user_name(&user_id, &mut conn) {
        Ok(username) => Ok(HttpResponse::Ok().body(username)),
        Err(err) => Err(error::ErrorBadGateway(format!(
            "Cant find username for given user id because {err}"
        ))),
    }
}

/// Create a new game
pub async fn new_game(session: Session, req: HttpRequest) -> Result<impl Responder, Error> {
    println!("NEW GAME REQ: {:?}", req);

    let mut conn = get_db_connection(req)?;

    if let Some(user_id) = session.get::<Uuid>(USER_ID_KEY)? {
        match db::create_session(&user_id, &mut conn) {
            Ok(session_id) => {
                session.insert(SESSION_ID_KEY, session_id.to_string())?;
                println!("\x1b[32;1mCreated session id {}\x1b[0m\n", session_id);
                Ok(web::Json(session_id))
            }
            Err(error) => Err(error::ErrorBadGateway(format!(
                "Cant register new session: {error}"
            ))),
        }
    } else {
        Err(error::ErrorBadGateway(
            "Cant find the current user ID in this session",
        ))
    }
}

/// Find a live session without a player 2
pub async fn find(session: Session, req: HttpRequest) -> Result<impl Responder, Error> {
    println!("REQ: {:?}", req);
    let mut conn = get_db_connection(req)?;
    match db::find_live_session(&mut conn) {
        Some(game_state) => {
            session.insert(SESSION_ID_KEY, game_state.id.to_owned())?;
            Ok(web::Json(game_state.id))
        }
        None => Ok(web::Json("None".to_string())),
    }
}

/// Join a session with given session_id
pub async fn join(
    form_input: web::Form<SessionInfo>,
    session: Session,
    req: HttpRequest,
) -> Result<HttpResponse, Error> {
    println!("REQ: {:?}", req);
    let mut conn = get_db_connection(req)?;
    let session_id = form_input.id.to_owned();

    if let Some(user_2_id) = session.get::<Uuid>(USER_ID_KEY)? {
        match db::join_live_session(&session_id, &user_2_id, &mut conn) {
            Ok(0) => Err(error::ErrorNotFound(format!(
                "No waiting sessions with id {session_id}"
            ))),
            Ok(1) => {
                println!("\x1b[32;1mUser joined successfully\x1b[0m\n");

                // Get the game state so we can retrieve user ids
                let game_state = match db::get_game_state(&session_id, &mut conn) {
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
                match db::update_game_state(&session_id, &l_user, "", "", &mut conn) {
                    Ok(_) => Ok(HttpResponse::Ok().body("")),
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
    } else {
        println!("Cant find the current user ID in this session");
        Err(error::ErrorBadGateway(
            "Cant find the current user ID in this session",
        ))
    }
}

/// Retrieve gamestate of session and return as json
pub async fn game_state_json(session: Session, req: HttpRequest) -> Result<impl Responder, Error> {
    println!("REQ: {:?}", req);
    let mut conn = get_db_connection(req)?;

    // Get the game_state and wrap it in json
    match get_game_state(&session, &mut conn).await {
        Ok(game_state) => Ok(web::Json(game_state)),
        Err(err) => Err(error::ErrorInternalServerError(err)),
    }
}

/// Retrieve the game state of a session
async fn get_game_state(
    session: &Session,
    conn: &mut SqliteConnection,
) -> Result<GameState, Error> {
    if let Some(session_id) = session.get::<Uuid>(SESSION_ID_KEY)? {
        println!("API: board, session_id: {:?}", session_id);
        session.insert(SESSION_ID_KEY, session_id)?;

        let res = db::get_game_state(&session_id, conn);
        match res {
            Ok(game_state) => Ok(game_state),
            _ => Err(error::ErrorInternalServerError(format!(
                "Can't find game with session id {session_id}"
            ))),
        }
    } else {
        Err(error::ErrorInternalServerError("Can't find game session"))
    }
}

/// Allow player to take some sort of action
pub async fn make_action(
    action: web::Json<BoardAction>,
    session: Session,
    req: HttpRequest,
) -> Result<impl Responder, Error> {
    println!("REQ: {:?}", req);
    let mut conn = get_db_connection(req)?;

    if let Some(session_id) = session.get::<Uuid>(SESSION_ID_KEY)? {
        // Retrieve the game_state
        let game_state = get_game_state(&session, &mut conn).await?;

        // Find out if we have a special player action.
        let move_status = match action.special.as_ref() {
            Some(special) if special == "forfeit" => {
                // Forfeit means active player is giving up
                forfeit(game_state, &session_id, &mut conn).await?
            }
            Some(special) if special == "skip" => {
                // Try and skip the current player's turn
                skip_turn(game_state, &session_id, &mut conn).await?
            }
            Some(_) => {
                // Any other special string is a pillbug / mosquito action
                do_special(game_state, action, &session_id, &mut conn).await?
            }
            None => {
                // Otherwise it's a normal move
                do_movement(game_state, action, &session_id, &mut conn).await?
            }
        };
        Ok(web::Json(move_status))
    } else {
        Err(error::ErrorInternalServerError("Can't find game session"))
    }
}

/// Try and execute movement
async fn do_movement(
    game_state: GameState,
    action: web::Json<BoardAction>,
    session_id: &Uuid,
    conn: &mut SqliteConnection,
) -> Result<MoveStatus, Error> {
    // Generate a board based on the gamestate and find the chip name and active team
    let mut board = game_state.to_cube_board();

    let active_team = game_state.which_team()?;
    let chip_name = action.get_chip_name();
    assert!(cheat_check(&action, &active_team));

    // Convert from doubleheight to the board's co-ordinate system
    let position = board.coord.mapfrom_doubleheight(action.rowcol);

    // Try and do the move, see what happens. If it's successful the board struct will update itself
    let move_status = board.move_chip(chip_name, active_team, position);

    // Create an event to track history of moves
    let event = Event::new_by_action(&action.into_inner());

    match move_status {
        MoveStatus::Success => execute_on_db(&mut board, game_state, event, session_id, conn),
        _ => Ok(move_status),
    }
}

/// Try and execute a chip special
async fn do_special(
    game_state: GameState,
    action: web::Json<BoardAction>,
    session_id: &Uuid,
    conn: &mut SqliteConnection,
) -> Result<MoveStatus, Error> {
    // Generate a board based on the gamestate and find the chip name and active team
    let mut board = game_state.to_cube_board();

    let active_team = game_state.which_team()?;
    assert!(cheat_check(&action, &active_team));

    // Try and decode and execute the special
    let move_status = hoive::pmoore::decode_specials(
        &mut board,
        &action.get_special(),
        active_team,
        action.get_chip_name(),
        action.rowcol,
    );

    // Create an event to track history of moves
    let event = Event::new_by_action(&action.into_inner());

    // Execute it on the db if it was successful
    match move_status {
        MoveStatus::Success => execute_on_db(&mut board, game_state, event, session_id, conn),
        _ => Ok(move_status),
    }
}

/// Execute a successful action on the db
fn execute_on_db<T: Coord>(
    board: &mut Board<T>,
    game_state: GameState,
    event: Event,
    session_id: &Uuid,
    conn: &mut SqliteConnection,
) -> Result<MoveStatus, Error> {
    // Refresh all mosquito names back to m1 and update board on server
    specials::mosquito_desuck(board);
    let board_str = board.encode_spiral();

    // Get the uuid of the current user and set them as the last_user in the db
    let l_user = game_state.which_user()?;

    // Parse the event into a string and append it to the board's history
    let history = game_state.add_event(event);

    let res = db::update_game_state(session_id, &l_user, &board_str, &history, conn);

    match res {
        Ok(_) => Ok(MoveStatus::Success),
        Err(err) => Err(error::ErrorInternalServerError(format!(
            "Problem updating gamestate because {err}"
        ))),
    }
}

/// Make sure the requested move is for the active player
fn cheat_check(form_input: &web::Json<BoardAction>, active_team: &Team) -> bool {
    let team_chips = form_input.which_team();
    team_chips == *active_team
}

async fn skip_turn(
    game_state: GameState,
    session_id: &Uuid,
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

async fn forfeit(
    game_state: GameState,
    session_id: &Uuid,
    conn: &mut SqliteConnection,
) -> Result<MoveStatus, Error> {
    // The winner is the team who didn't forfeit
    let winner = !game_state.which_team()?;

    // Append F to to designate the reason for winning as a forfeit
    let win_string = format!("{}F", winner.to_string());

    // Update the last user id to the person who forfeit (the active team)
    let l_user_id = game_state.which_user()?;

    // Update db
    let res = db::update_winner(session_id, &l_user_id, &win_string, conn);

    match res {
        Ok(_) => Ok(MoveStatus::Success),
        Err(err) => Err(error::ErrorInternalServerError(format!(
            "Problem updating winner in gamestate because {err}"
        ))),
    }
}

/// For debugging only. Delete the db on the server
pub async fn delete_all(req: HttpRequest) -> Result<HttpResponse, Error> {
    let mut conn = get_db_connection(req).unwrap();

    db::clean_db(&mut conn);
    println!("Database cleared");

    Ok(HttpResponse::Ok().body("Cleared"))
}
