/// API is the middleman between the game's logic and the front-end. It converts string commands from the front
/// end into commands the board understands, and converts responses from the board into human-readable strings.
///
use actix_session::Session;
use actix_web::{error, web, Error, HttpRequest, HttpResponse};

use serde_json::json;
use std::result::Result;


pub use crate::db;
pub use crate::game;
pub use crate::models;
pub use crate::schema;
use diesel::r2d2::{ConnectionManager, Pool, PooledConnection};
use diesel::SqliteConnection;
use uuid::Uuid;

const SESSION_ID_KEY: &str = "session_id";
const USER_ID_KEY: &str = "user_id";
const USER_COLOR_KEY: &str = "user_color";

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

/// Register a new user with given name/team (input within path)
pub async fn register_user(
    path: web::Path<(String, String)>,
    session: Session,
    req: HttpRequest,
) -> Result<HttpResponse, Error> {

    // First and second parts of path will be username and team
    let user_name = &path.0;
    let user_color = &path.1;
    println!("REQ: {:?}", req);
    println!("User Name: {:?}", user_name);
    println!("User Color: {:?}", user_color);

    let mut conn = get_db_connection(req)?;

    match db::create_user(&user_name, &user_color, &mut conn) {
        Ok(user_id) => {
            session.insert(USER_ID_KEY, user_id.to_string())?;
            session.insert(USER_COLOR_KEY, user_color.clone())?;
            
            let user = models::User {
                id: user_id.to_string(),
                user_name: user_name.to_owned(),
                user_color: user_color.to_owned(),
            };
            // This used to be a json and i think it needs to be.
            Ok(HttpResponse::Ok().body(format!("user id: {}",user_id)))
        }
        Err(error) => Err(error::ErrorBadGateway(format!("Cant register new user: {error}"))),
    }
}


/// Create a new game
pub async fn new_game(session: Session, req: HttpRequest) -> Result<HttpResponse, Error> {
    println!("NEW GAME REQ: {:?}", req);

    let mut conn = get_db_connection(req)?;

    if let Some(user_id) = session.get::<Uuid>(USER_ID_KEY)? {
        match db::create_session(&user_id, &mut conn) {
            Ok(session_id) => {
                session.insert(SESSION_ID_KEY, session_id.to_string())?;
                Ok(HttpResponse::Ok().body(format!("SESSION_ID_KEY: {}",session_id)))
            }
            Err(error) => Err(error::ErrorBadGateway(format!("Cant register new session: {error}"))),
            }
        }
    else {
        Err(error::ErrorBadGateway("Cant find the current user ID in this session"))
    }
}

/// Find a live session without a player 2
pub async fn find(session: Session, req: HttpRequest) -> Result<HttpResponse, Error> {
    println!("REQ: {:?}", req);
    let mut conn = get_db_connection(req)?;
    match db::find_live_session(&mut conn) {
        Some(game_state) => {
            session.insert(SESSION_ID_KEY, game_state.id.to_owned())?;
            Ok(HttpResponse::Ok().body(game_state.id))
        }
        None => Err(error::ErrorNotFound("No live sessions with a single player found")),
    }
}


/// Join a session with given session_id
pub async fn join(
    session_id: web::Path<Uuid>,
    session: Session,
    req: HttpRequest,
) -> Result<HttpResponse, Error> {
    println!("REQ: {:?}", req);
    let mut conn = get_db_connection(req)?;
    let game_id = session_id.into_inner();
    if let Some(user_2_id) = session.get::<Uuid>(USER_ID_KEY)? {
        match db::join_live_session(&game_id, &user_2_id, &mut conn) {
            Ok(0) => Err(error::ErrorNotFound(format!("No waiting sessions with id {game_id}"))),
            Ok(1) => Ok(HttpResponse::Ok().body("Ok")),
            Ok(_) => Err(error::ErrorBadGateway("Multiple sessions updated")),
            Err(error) => Err(error::ErrorBadGateway(format!("Cant join session: {}", error))),
        }
    } else {
        Err(error::ErrorBadGateway("Cant find the current user ID in this session"))
    }
}

/// Get the current state of the board in a session
pub async fn game_state(session: Session, req: HttpRequest) -> Result<HttpResponse, Error> {
    println!("REQ: {:?}", req);
    let mut conn = get_db_connection(req)?;
    if let Some(session_id) = session.get::<Uuid>(SESSION_ID_KEY)? {
        println!("API: board, session_id: {:?}", session_id);
        session.insert(SESSION_ID_KEY, session_id)?;

        // let id = session_id.into_inner();
        let res = db::get_game_state(&session_id, &mut conn);
        match res {
            Ok(game_state) => Ok(HttpResponse::Ok().body(format!("{:?}",game_state))),
            _ => Err(error::ErrorInternalServerError(format!("Can't find game with session id {session_id}"))),
        }
    } else {
        Err(error::ErrorInternalServerError("Can't find game session"))
    }
}

/// Take some sort of action on the board
pub async fn make_action(
    path: web::Path<u32>,
    session: Session,
    req: HttpRequest,
) -> Result<HttpResponse, Error> {
    // For now the thing passed is a number, but we'll later pass a string command like "bq1,0,-2" or "bp1,s,from,to" etc
    let column = path.into_inner();
    println!("REQ: {:?}", req);
    let conn = get_db_connection(req)?;
    if let (Some(session_id), Some(user_id)) = (
        session.get::<Uuid>(SESSION_ID_KEY)?,
        session.get::<Uuid>(USER_ID_KEY)?,
    ) {
        // THIS is where we get a result out. Their version of game is probably my pmoore, sort of.
        // pmoore is kind of doing two jobs at the moment which is bad (he's the front end and the logic)
        //let res = game::user_move(session_id, user_id, column as usize, conn.deref());
        let res:Result<&str,&str> = Ok("placeholder");

        match res {
            Ok(game_state) => {
                println!("API make_move returns: {:?}", game_state);
                Ok(HttpResponse::Ok().json(game_state))
            }
            Err(msg) => Err(error::ErrorInternalServerError(msg)),
        }
    } else {
        Err(error::ErrorInternalServerError("[user_move] No session info"))
    }
}

// use crate::game::{board::Board, movestatus::MoveStatus};
// use crate::maths::coord::Coord;
// use crate::maths::coord::Cube;

// /// Start a new game, create a db respond with how it went
// fn new_game() {
//     // Initialise game board in cube co-ordinates
//     let coord = Cube::default();
//     let mut board = Board::new(coord);
// }

// We need a way of storing a board as a string in an sqlitedb
// need a table called gamestate which has:
// session id, a board (string representing board), user1, user2, current-player, ended (bool)

// Then have the option to find an existing session without a user2 and join it as a player

// let encoded = board.encode_spiral();
// println!("The spiral string is:\n {}", encoded);
// let newboard = board.decode_spiral(encoded);
// println!("SPIRAL BOARD\n{}", draw::show_board(&newboard));
