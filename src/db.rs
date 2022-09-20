use diesel::connection::SimpleConnection;
use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, Pool};

use diesel::result::QueryResult;
use diesel::SqliteConnection;

use std::result::Result;

use dotenvy::dotenv;
use std::env;
use std::time::Duration;
use uuid::Uuid;

pub use crate::models;
use crate::models::{GameState, NewGameState, User};
pub use crate::schema;

#[derive(Debug)]
pub struct ConnectionOptions {
    pub enable_wal: bool,
    pub enable_foreign_keys: bool,
    pub busy_timeout: Option<Duration>,
}

impl diesel::r2d2::CustomizeConnection<SqliteConnection, diesel::r2d2::Error>
    for ConnectionOptions
{
    fn on_acquire(&self, conn: &mut SqliteConnection) -> Result<(), diesel::r2d2::Error> {
        (|| {
            if self.enable_wal {
                conn.batch_execute("PRAGMA journal_mode = WAL; PRAGMA synchronous = NORMAL;")?;
            }
            if self.enable_foreign_keys {
                conn.batch_execute("PRAGMA foreign_keys = ON;")?;
            }
            if let Some(d) = self.busy_timeout {
                conn.batch_execute(&format!("PRAGMA busy_timeout = {};", d.as_millis()))?;
            }
            Ok(())
        })()
        .map_err(diesel::r2d2::Error::QueryError)
    }
}

/// Creates a connection pool
pub fn create_conn_pool() -> Pool<ConnectionManager<SqliteConnection>> {
    dotenv().ok();
    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    Pool::builder()
        .max_size(16)
        .connection_customizer(Box::new(ConnectionOptions {
            enable_wal: true,
            enable_foreign_keys: true,
            busy_timeout: Some(Duration::from_secs(30)),
        }))
        .build(ConnectionManager::<SqliteConnection>::new(db_url))
        .unwrap()
}

/// Establish connection to db
// fn establish_connection() -> SqliteConnection {
//     dotenv().ok();

//     let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
//     SqliteConnection::establish(&database_url)
//         .unwrap_or_else(|_| panic!("Error connecting to {}", database_url))
// }

/// Creates a new user on the db with a given name and team
pub fn create_user(name: &str, team: &str, conn: &mut SqliteConnection) -> Result<Uuid, String> {
    use super::schema::user::dsl::*;
    //let conn = &mut establish_connection();

    let uuid = Uuid::new_v4();

    let new_user = User {
        id: uuid.to_string(),
        user_name: name.to_owned(),
        user_color: team.to_owned(),
    };

    match diesel::insert_into(user).values(&new_user).execute(conn) {
        Ok(_) => Ok(uuid),
        Err(e) => Err(format!("Can't create new user because {}", e)),
    }
}

/// Creates a new game session (a new board) on the db, with a given player1
pub fn create_session(user: &Uuid, conn: &mut SqliteConnection) -> Result<Uuid, String> {
    // We have the use statement here rather than at the top of the module because in some functions we'll
    // want to use schema::user::dsl::* in functions that deal with the user part of the db, and we don't want ambiguity.
    use schema::game_state::dsl::*;
    //let conn = &mut establish_connection();

    let session_id = Uuid::new_v4();

    let new_game = NewGameState {
        id: session_id.to_string(),
        board: None,
        user_1: Some(user.to_string()),
    };

    match diesel::insert_into(game_state)
        .values(&new_game)
        .execute(conn)
    {
        Ok(_) => Ok(session_id),
        Err(e) => Err(format!("Can't create new session because {}", e)),
    }
}

/// Finds an existing game session that a second user can join
pub fn find_live_session(conn: &mut SqliteConnection) -> Option<models::GameState> {
    use schema::game_state::dsl::*;

    // Search the db for active games where there's no player 2
    let results = game_state
        .filter(user_2.is_null())
        .load::<GameState>(conn)
        .expect("Error loading gamestates");

    // Return the first (oldest) result if it exists, otherwise none
    println!("Results from db are: {:?}", results);
    results.first().cloned()
}

/// Lets a user_2 join a live session
pub fn join_live_session(
    session_id: &Uuid,
    user2_id: &Uuid,
    conn: &mut SqliteConnection,
) -> QueryResult<usize> {
    use schema::game_state::dsl::*;
    //let conn = &mut establish_connection();

    diesel::update(game_state)
        .set(user_2.eq(user2_id.to_string()))
        .filter(id.eq(session_id.to_string()))
        .execute(conn)
}

/// Gets the board from a given game session in the db
pub fn get_board(session_id: &Uuid, conn: &mut SqliteConnection) -> Result<String, String> {
    use schema::game_state::dsl::*;
    //let conn = &mut establish_connection();

    let results = game_state
        .filter(id.eq(session_id.to_string()))
        .limit(1)
        .load::<GameState>(conn)
        .expect("Error loading gamestates");

    let fetched_board = results[0].board.as_ref().unwrap();

    Ok(fetched_board.to_string())
}

/// Update the game state of a given session_id with new info on last user, board state, whether there's a winner or game over
pub fn update_game_state(
    session_id: &Uuid,
    l_user_id: &str,
    board_str: &str,
    is_winner: &str,
    conn: &mut SqliteConnection,
) -> QueryResult<usize> {
    use schema::game_state::dsl::*;

    diesel::update(game_state)
        .filter(id.eq(session_id.to_string()))
        .set((
            last_user_id.eq(l_user_id.to_string()),
            board.eq(board_str),
            winner.eq(is_winner),
        ))
        .execute(conn)
}

/// Update the winner only
pub fn update_winner(
    session_id: &Uuid,
    l_user_id: &str,
    is_winner: &str,
    conn: &mut SqliteConnection,
) -> QueryResult<usize> {
    use schema::game_state::dsl::*;

    diesel::update(game_state)
        .filter(id.eq(session_id.to_string()))
        .set((last_user_id.eq(l_user_id.to_string()), winner.eq(is_winner)))
        .execute(conn)
}

/// Get the username of a user of given id
pub fn get_user_name(user_id: &Uuid, conn: &mut SqliteConnection) -> QueryResult<String> {
    use schema::user::dsl::*;

    let result = user
        .select(user_name)
        .filter(id.eq(user_id.to_string()))
        .limit(1)
        .load::<String>(conn)
        .expect("Error getting username");

    Ok(result[0].clone())
}

/// Get the general game state of the selected session_id
pub fn get_game_state(
    session_id: &Uuid,
    conn: &mut SqliteConnection,
) -> QueryResult<models::GameState> {
    use super::schema::game_state::dsl::*;

    let res = game_state
        .filter(id.eq(session_id.to_string()))
        .limit(1)
        .load::<GameState>(conn)
        .expect("Error loading game state");
    Ok(res[0].clone())
}

/// Clear the db (wipe all gamestates and users)
pub fn clean_db(conn: &mut SqliteConnection) {
    use super::schema::game_state::dsl::*;
    use super::schema::user::dsl::*;

    diesel::delete(game_state).execute(conn).unwrap();
    diesel::delete(user).execute(conn).unwrap();
}
