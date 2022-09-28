/// API is the middleman between the game's logic and the front-end. It converts string commands from the front
/// end (passed as Httprequests) into commands to set up and use the board and database. It also converts and passes responses
/// back (usually as jsons).
///

use std::result::Result;
use std::str::FromStr;

use rustrict::CensorStr;
use rand::Rng;
use uuid::Uuid;
use serde::Deserialize;

use actix_session::Session;
use actix_web::{error, web, Error, HttpRequest, HttpResponse};
use actix_web::Responder;

pub use crate::db;
use crate::models::GameState;
pub use crate::models::{self, User};
pub use crate::schema;
use diesel::r2d2::{ConnectionManager, Pool, PooledConnection};
use diesel::SqliteConnection;

const SESSION_ID_KEY: &str = "session_id";
const USER_ID_KEY: &str = "user_id";
const USER_COLOR_KEY: &str = "user_color";

use hoive::game::{comps::Team, movestatus::MoveStatus, actions::BoardAction, board::Board, specials};
use hoive::maths::coord::{Coord, Cube};


/// Info grabbed about session from form
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

/// Register a new user with requested name (input by a web form)
pub async fn register_user(
    form_input: web::Form<User>,
    session: Session,
    req: HttpRequest,
) -> Result<impl Responder, Error> {

    let user_name = form_input.user_name.to_owned();

    // Check the username isn't profanity
    if user_name.is_inappropriate() {
        return Ok(web::Json("invalid".to_string()));
    }

    let user_color = "red".to_string();
    println!("REQ: {:?}", req);
    println!("User Name: {:?}", user_name);
    println!("User Color: {:?}", user_color);

    let mut conn = get_db_connection(req)?;

    match db::create_user(&user_name, &user_color, &mut conn) {
        Ok(user_id) => {
            session.insert(USER_ID_KEY, user_id.to_string())?;
            session.insert(USER_COLOR_KEY, user_color.clone())?;

            println!("{}", user_id);
            Ok(web::Json(user_id.to_string()))
        }
        Err(error) => Err(error::ErrorBadGateway(format!(
            "Cant register new user: {error}"
        ))),
    }
}

/// Get a user name based on an input user id
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
                println!("Created session id {}", session_id);
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
    let game_id = session_id;

    match session.get::<Uuid>(USER_ID_KEY) {
        Ok(value) => println!("{:?}", value),
        Err(err) => println!("error: {}", err),
    }

    if let Some(user_2_id) = session.get::<Uuid>(USER_ID_KEY)? {
        match db::join_live_session(&game_id, &user_2_id, &mut conn) {
            Ok(0) => Err(error::ErrorNotFound(format!(
                "No waiting sessions with id {game_id}"
            ))),
            Ok(1) => {
                println!("User joined successfully");

                // Toss a coin to see who goes first
                let mut rand = rand::thread_rng();
                let second = match rand.gen() {
                    true => "B",
                    false => "W",
                };

                let teamy = Team::from_str(second).unwrap();

                // Update the db and return a string
                match db::update_game_state(&game_id, teamy, "", &mut conn) {
                    Ok(_) => Ok(HttpResponse::Ok().body(second)),
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

/// Retrieve and then return the state of a board in a session
pub async fn game_state(session: Session, req: HttpRequest) -> Result<impl Responder, Error> {
    println!("REQ: {:?}", req);

    let mut conn = get_db_connection(req)?;
    // Get the game_state and wrap it in json
    match retrieve_game_state(&session, &mut conn).await {
        Ok(game_state) => Ok(web::Json(game_state)),
        Err(err) => Err(error::ErrorInternalServerError(err)),
    }
}

/// Try retrieve the game state of a session
async fn retrieve_game_state(
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

/// Returns who the db thinks is the current player
fn current_player(game_state: &GameState) -> Result<Team, Error> {
    // This will return the previous player
    let string_version = match &game_state.last_user_id {
        Some(value) => value,
        None => panic!("Tried to use a team query function before teams initialised"),
    };

    match Team::from_str(string_version.as_str()) {
        Ok(value) => Ok(!value), // return not previous player for current player
        Err(err) => Err(error::ErrorInternalServerError(err)),
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
        // Retrieve the game_state and current player
        let game_state = retrieve_game_state(&session, &mut conn).await?;
        let active_team = current_player(&game_state)?;

        // Find out if we have a special player action
        match action.special.as_ref() {
            Some(special) if special == "forfeit" => {
                // Forfeit means active player is giving up
                match forfeit(active_team, &session_id, &mut conn).await {
                    Ok(_) => Ok(web::Json(MoveStatus::Success)),
                    Err(err) => Err(error::ErrorInternalServerError(format!(
                        "Problem updating gamestate because {err}"
                    ))),
                }
            }
            Some(special) if special == "skip" => {
                // Try and skip the current player's turn
                match skip_turn(game_state, active_team, &session_id, &mut conn).await {
                    Ok(move_status) => Ok(web::Json(move_status)),
                    Err(err) => Err(error::ErrorInternalServerError(format!(
                        "Problem updating gamestate because {err}"
                    ))),
                }
            }
            Some(_) => {
                // Any other special string is a pillbug / mosquito action
                match do_special(game_state, action, &session_id, &mut conn).await {
                    Ok(move_status) => Ok(web::Json(move_status)),
                    Err(err) => Err(error::ErrorInternalServerError(format!(
                        "Problem updating gamestate because {err}"
                    ))),
                }
            }
            None => {
                // Otherwise it's a normal move
                match do_movement(game_state, action, &session_id, &mut conn).await {
                    Ok(move_status) => Ok(web::Json(move_status)),
                    Err(err) => Err(error::ErrorInternalServerError(format!(
                        "Problem updating gamestate because {err}"
                    ))),
                }
            }
        }
    } else {
        return Err(error::ErrorInternalServerError("Can't find game session"));
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
    let active_team = current_player(&game_state)?;
    let chip_name = action.get_chip_name();
    assert!(cheat_check(&action, &active_team));

    // Convert from doubleheight to the board's co-ordinate system
    let position = board.coord.mapfrom_doubleheight(action.rowcol);

    // Try and do the move, see what happens. If it's successful the board struct will update itself
    let move_status = board.move_chip(chip_name, active_team, position);

    match move_status {
        MoveStatus::Success => {
            // Refresh all mosquito names back to m1 and update board on server
            specials::mosquito_desuck(&mut board);
            let board_str = board.encode_spiral();
            let res = db::update_game_state(&session_id, active_team, &board_str, conn);

            match res {
                Ok(_) => Ok(move_status),
                Err(err) => Err(error::ErrorInternalServerError(format!(
                    "Problem updating gamestate because {err}"
                ))),
            }
        }
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
    let active_team = current_player(&game_state)?;
    assert!(cheat_check(&action, &active_team));

    // Try and decode and execute the special
    let move_status = hoive::pmoore::decode_specials(
        &mut board,
        &action.get_special(),
        active_team,
        action.get_chip_name(),
        action.rowcol,
    );

    match move_status {
        MoveStatus::Success => {
            // Refresh all mosquito names back to m1 and update db
            specials::mosquito_desuck(&mut board);
            let board_str = board.encode_spiral();

            let res = db::update_game_state(&session_id, active_team, &board_str, conn);

            match res {
                Ok(_) => return Ok(move_status),
                Err(err) => {
                    return Err(error::ErrorInternalServerError(format!(
                        "Problem updating gamestate because {err}"
                    )))
                }
            }
        }
        _ => return Ok(move_status),
    }
}

/// Make sure the requested move is for the active player
/// Will need to do some more thorough checks later such as making sure the playerid matches
fn cheat_check(form_input: &web::Json<BoardAction>, active_team: &Team) -> bool {
    let chip_name = form_input.name.as_str();

    // Black chips get passed as uppercase, white as lowercase
    let team_chips = match chip_name.chars().next().unwrap().is_uppercase() {
        true => Team::Black,
        false => Team::White,
    };

    team_chips == *active_team
}

async fn skip_turn(
    game_state: GameState,
    active_team: Team,
    session_id: &Uuid,
    conn: &mut SqliteConnection,
) -> Result<MoveStatus, Error> {
    // Get the board
    let board_state = game_state.board.unwrap(); // this might die if we have an empty board

    // generate a board in Cube coords based on the existing state
    let board = Board::new(Cube::default());
    let mut board = board.decode_spiral(board_state);

    match board.try_skip_turn(active_team) {
        MoveStatus::Success => {
            // Do skip, change the active team in the db
            match db::update_active_team(session_id, &active_team.to_string(), conn) {
                Ok(_) => Ok(MoveStatus::Success),
                Err(err) => return Err(error::ErrorInternalServerError(err)),
            }
        }
        MoveStatus::NoSkip => Ok(MoveStatus::NoSkip),
        _ => unreachable!(),
    }
}

async fn forfeit(
    active_team: Team,
    session_id: &Uuid,
    conn: &mut SqliteConnection,
) -> Result<(), Error> {
    // The winner is the team who didn't forfeit
    let winner = !active_team;

    // Append F to to designate the reason for winning as a forfeit
    let win_string = format!("{}F", winner.to_string());

    // Update the last user id to the person who forfeit (the active team)
    let l_user_id = active_team.to_string();

    // Update db
    let res = db::update_winner(&session_id, &l_user_id, &win_string, conn);

    match res {
        Ok(_) => return Ok(()),
        Err(err) => {
            return Err(error::ErrorInternalServerError(format!(
                "Problem updating winner in gamestate because {err}"
            )))
        }
    };
}

pub async fn delete_all(req: HttpRequest) -> Result<HttpResponse, Error> {
    let mut conn = get_db_connection(req).unwrap();

    db::clean_db(&mut conn);
    println!("Database cleared");

    Ok(HttpResponse::Ok().body("Cleared"))
}
