/// API is the middleman between the game's logic and the front-end. It converts string commands from the front
/// end (passed as Httprequests) into commands to set up and use the board and database. It also converts and passes responses
/// back (usually as jsons).
///
use actix_session::Session;
use actix_web::{error, web, Error, HttpRequest, HttpResponse};

use actix_web::Responder;
use rustrict::CensorStr;
use std::result::Result;

use rand::Rng;

pub use crate::db;
pub use crate::game;
use crate::game::movestatus::MoveStatus;
use crate::models::GameState;
pub use crate::models::{self, User};
pub use crate::schema;
use diesel::r2d2::{ConnectionManager, Pool, PooledConnection};
use diesel::SqliteConnection;
use uuid::Uuid;

const SESSION_ID_KEY: &str = "session_id";
const USER_ID_KEY: &str = "user_id";
const USER_COLOR_KEY: &str = "user_color";

use serde::Deserialize;

/// Info grabbed about session from form
#[derive(Deserialize)]
pub struct SessionInfo {
    id: Uuid,
}

#[derive(Deserialize, Debug, Clone)]
pub struct BoardAction{
    pub name: String,   // chip name
    pub rowcol: (i8,i8), // destination row,col
    pub special: Option<String>, // stores special move info (e.g. p,sourcerow,sourcecol)
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

pub async fn index() -> HttpResponse {
    HttpResponse::Ok().body(format!("Hoive-server v{}", crate::VERSION))
}

/// Register a new user with given name/team (input within path)
pub async fn register_user(
    form_input: web::Form<User>,
    session: Session,
    req: HttpRequest,
) -> Result<impl Responder, Error> {
    // First and second parts of path will be username and team
    let user_name = form_input.user_name.to_owned();

    // Check the username isn't profane
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

            // let user = models::User {
            //     id: user_id.to_string(),
            //     user_name: user_name.to_owned(),
            //     user_color: user_color.to_owned(),
            // };
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
    //session_id: web::Path<Uuid>,
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


                // Update the db and return a string
                match db::update_game_state(&game_id, second, "", "", &mut conn) {
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

// /// Pick a player to go first and update the db (happens once at the start of a game)
// async fn coin_toss(session: Session, req: HttpRequest) -> Result<HttpResponse, Error> {
//     // Select a random team to go second
//     let mut rand = rand::thread_rng();
//     let second = match rand.gen() {
//         true => "B",
//         false => "W",
//     };

//     println!("{second} is going second");
//     // Up date the db and return the player who goes second
//     let mut conn = get_db_connection(req)?;

//     if let Some(session_id) = session.get::<Uuid>(SESSION_ID_KEY)? {
//         match db::update_game_state(&session_id, second, "", false, false, &mut conn) {
//             Ok(_) => Ok(HttpResponse::Ok().body(second)),
//             Err(err) => Err(error::ErrorInternalServerError(format!(
//                 "Can't update game state of {session_id} because {err}"
//             ))),
//         }
//     } else {
//         Err(error::ErrorInternalServerError(format!(
//             "Can't find game session"
//         )))
//     }
// }

/// Get the current state of the board in a session
pub async fn game_state(session: Session, req: HttpRequest) -> Result<impl Responder, Error> {
    println!("REQ: {:?}", req);
    let mut conn = get_db_connection(req)?;
    if let Some(session_id) = session.get::<Uuid>(SESSION_ID_KEY)? {
        println!("API: board, session_id: {:?}", session_id);
        session.insert(SESSION_ID_KEY, session_id)?;

        let res = db::get_game_state(&session_id, &mut conn);
        match res {
            // This should return a json
            Ok(game_state) => Ok(web::Json(game_state)),
            _ => Err(error::ErrorInternalServerError(format!(
                "Can't find game with session id {session_id}"
            ))),
        }
    } else {
        Err(error::ErrorInternalServerError("Can't find game session"))
    }
}

/// Take some sort of action on the board
pub async fn make_action(
    form_input: web::Json<BoardAction>,
    session: Session,
    req: HttpRequest,
) -> Result<impl Responder, Error> {
 
    use crate::maths::coord::DoubleHeight;
    use crate::game::board::Board;
    use crate::maths::coord::{Cube,Coord};
    use crate::game::comps::Team;
    use crate::game::comps::convert_static_basic;
    use crate::draw;

    println!("REQ: {:?}", req);



    // See if the action is valid
    // Get the current board state...

    // This is almost a carbon copy of pub async fn game_state so it's silly
    let mut conn = get_db_connection(req)?;
    let gamey: Result<GameState,_>;

    if let Some(session_id) = session.get::<Uuid>(SESSION_ID_KEY)? {
        println!("API: board, session_id: {:?}", session_id);
        session.insert(SESSION_ID_KEY, session_id)?;

        let res = db::get_game_state(&session_id, &mut conn);
        gamey = match res {
            Ok(game_state) => Ok(game_state),
            _ => Err(error::ErrorInternalServerError(format!(
                "Can't find game with session id {session_id}"
            ))),
        };
    } else {
        gamey = Err(error::ErrorInternalServerError("Can't find game session"));
    }



    println!("Recieved {:?} as form_input.special", form_input.special);

    match &form_input.special {
        None => (), //do normal movement
        Some(value) if value == "forfeit" => {
            // Flag the opposing player as the winner
            
                    // again hacky
        let session_id = session.get::<Uuid>(SESSION_ID_KEY)?.unwrap();
        // get the not current user
        let winstring = gamey.as_ref().unwrap().last_user_id.as_ref().unwrap();

        // l-user will be the one that forfeit
        let winstring = match winstring {
            _ if winstring == "B" => "BF",
            _ if winstring == "W" => "WF",
            _ => panic!("Unrecognised player string"),
        };

        let boardo = gamey.as_ref().unwrap().board.as_ref().unwrap();

        // This doesn't allow a waiting player to see they win because it doesn't update l-user-id properly.
        // For this to work properly, we need to generate a last user id to be the opposite of this
        // Create some functions that map between B and W as string and team.
        let l_user_id = gamey.as_ref().unwrap().last_user_id.as_ref().unwrap();

        let res = db::update_game_state(
            &session_id,
            l_user_id,
            boardo, 
            winstring,
            &mut conn,
        );
        
        match res {
            Ok(_) => return Ok(web::Json(MoveStatus::Success)),
            Err(err) => return Err(error::ErrorInternalServerError(format!(
                "Problem updating gamestate because {err}"))),
        };
        

        }
        , //forfeit
        Some(_value) => (), // do specials
    }


    let board_state = gamey?.board.unwrap(); // this might die if we have an empty board

    // generate a board based on the existing state
    let board = Board::new(Cube::default());
    let mut board = board.decode_spiral(board_state);

    // Convert the input move into DoubleHeight coordinates
    let moveto = DoubleHeight::from(form_input.rowcol);

    // Convert from doubleheight to the board's co-ordinate system
    let game_hex = board.coord.mapfrom_doubleheight(moveto);

    // Parse the input string to find the team
    // better to use gamey.unwrap().last_user_id
    let active_team = match form_input.name.chars().next().unwrap().is_uppercase() {
        true => Team::Black,
        false => Team::White,
    };

    let chip_name = form_input.name.to_lowercase();

    // convert to static
    let chip_name = convert_static_basic(chip_name).unwrap();


    // Try and do the move, see what happens. If it's successful the board will update itself
    let move_status = board.move_chip(chip_name, active_team, game_hex);

    if move_status == MoveStatus::Success {
        // update the board on the server
        // for debugging, print the board
        println!("{}",draw::show_board(&board));
        // get the spiral string
        let board_str = board.encode_spiral();
        println!("Spiral string is {}", board_str);

        // better to use gamey.unwrap().last_user_id
        let l_user_id = match active_team {
            Team::Black => "B",
            Team::White => "W",
        };

        // again hacky
        let session_id = session.get::<Uuid>(SESSION_ID_KEY)?.unwrap();
        let res = db::update_game_state(
            &session_id,
            l_user_id,
            &board_str,
            "", // to do later, code to try movestatus = win 
            &mut conn,
        );
        
        match res {
            Ok(_) => Ok(web::Json(move_status)),
            Err(err) => Err(error::ErrorInternalServerError(format!(
                "Problem updating gamestate because {err}"))),
        }

    } else {
        Ok(web::Json(move_status))
    }


}


pub async fn delete_all(req: HttpRequest) -> Result<HttpResponse, Error> {
    let mut conn = get_db_connection(req).unwrap();

    db::clean_db(&mut conn);
    println!("Database cleared");

    Ok(HttpResponse::Ok().body("Cleared"))
}

