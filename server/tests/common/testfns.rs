use actix_web::error::ErrorBadGateway;
use actix_web::Error;

use uuid::Uuid;

use diesel::r2d2::ConnectionManager;
use diesel::r2d2::PooledConnection;
use diesel::SqliteConnection;

use server::db::*;

use bytes::Bytes;
use std::str;

/// Update the gamestate of a live session with some garbage for tests. If clean == true then it wipes db first
pub fn test_server_garbage(
    clean: bool,
) -> Result<
    (
        Uuid,
        Uuid,
        PooledConnection<ConnectionManager<SqliteConnection>>,
    ),
    Error,
> {
    let mut con = create_conn_pool().get().unwrap();

    // Wipe the db clean
    if clean {
        clean_db(&mut con);
    }

    // Invent a user and create a session
    let user_id = Uuid::new_v4();
    let session_id = create_session(&user_id, &mut con).unwrap();

    // Update the gamestate with some garbage values
    match update_game_state(
        &session_id,
        &user_id.to_string(),
        "board_test",
        "history_test",
        &mut con,
    ) {
        Ok(_) => Ok((session_id, user_id, con)),
        Err(err) => Err(ErrorBadGateway(err)),
    }
}

// Convert bytes to str
pub fn bytes_to_str(b: &Bytes) -> Result<&str, str::Utf8Error> {
    str::from_utf8(b)
}
