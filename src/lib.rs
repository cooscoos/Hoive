#[macro_use]
extern crate diesel;
extern crate dotenvy;

use actix_files as fs;
//use actix_session::CookieSession;
use actix_web::{web, cookie::Key, App, HttpServer};
use actix_session::{Session, SessionMiddleware, storage::RedisActorSessionStore};

pub mod api;
pub mod db;
pub mod models;
pub mod schema;
pub mod draw;
pub mod front_end;
pub mod game;
pub mod maths;
pub mod pmoore;

#[actix_web::main]
pub async fn start_server() -> std::io::Result<()> {

    //Todo: understand this https://docs.rs/actix-session/latest/actix_session/
    // The secret key would usually be read from a configuration file/environment variables.
    let secret_key = Key::generate();
    let redis_connection_string = "127.0.0.1:6379";

    HttpServer::new(move || {
        App::new()
            .app_data(db::create_conn_pool())
            .wrap(SessionMiddleware::new(
                RedisActorSessionStore::new(redis_connection_string),
                secret_key.clone()))
            .service(
                web::scope("/api")
                    .service(
                        web::resource("/register/{user_name}/{user_color}")
                            .route(web::post().to(api::register_user)),
                    )
                    .service(web::resource("/new").route(web::get().to(api::new_game)))
                    .service(web::resource("/find").route(web::get().to(api::find)))
                    .service(
                        web::resource("/join/{game_session_id}").route(web::post().to(api::join)),
                    )
                    .service(web::resource("/game-state").route(web::get().to(api::game_state)))
                    .service(
                        web::resource("/make-move/{column}").route(web::post().to(api::make_action)),
                    ),
            )
            .service(fs::Files::new("/", "./static").index_file("index.html"))
    })
    .bind("127.0.0.1:8888")?
    .run()
    .await
}