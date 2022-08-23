pub mod models;
pub mod schema;

use diesel::prelude::*;
use dotenv::dotenv;
use uuid::Uuid;
use std::env;

use models::*;
use schema::game_state;
use schema::game_state::dsl::*;

/// Establishes a connection to the sqlite database
fn establish_connection() -> SqliteConnection {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");
    SqliteConnection::establish(&database_url)
        .unwrap_or_else(|_| panic!("Error connecting to {}", 
                                   database_url))
}

/// Create a new user on the db, assigning it a new user id
pub fn create_new_user(name: &str, team: &str, conn: &SqliteConnection) -> Result<Uuid,String> {
    use super::schema::user::dsl::*;

    // Create new user from a generated id
    let new_user_id = Uuid::new_v4();

    let new_user = models::User {
        id: new_user_id.to_string(),
        user_name: name.to_owned(),
        user_color: team.to_owned(),
    };

    // Try insert into db
    match diesel::insert_into(user).values(&new_user).execute(conn) {
        Ok(_) => Ok(new_user_id),
        Err(err) => Err(format!("Can't create new user because: {err}")),
    }

    

}

// Probably not required, but is good toy example
pub fn get_posts() -> Vec<GameState> {
    let connection = establish_connection();
    game_state
        .limit(5)
        .load::<GameState>(&connection)
        .expect("Error loading gamestate")
}