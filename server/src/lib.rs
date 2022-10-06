#[macro_use]
extern crate diesel;
extern crate dotenvy;

use actix_session::{storage::CookieSessionStore, SessionMiddleware};
use actix_web::{cookie::Key, web, App, HttpServer};

pub mod api;
pub mod db;
pub mod models;
pub mod schema;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

fn get_secret_key() -> Key {
    Key::generate()
}

#[actix_web::main]
pub async fn start_server() -> std::io::Result<()> {
    let secret_key = get_secret_key();

    HttpServer::new(move || {
        App::new()
            .app_data(db::create_conn_pool())
            .wrap(SessionMiddleware::new(
                CookieSessionStore::default(),
                secret_key.clone(),
            ))
            .service(
                web::scope("/api")
                    .service(web::resource("/").route(web::get().to(api::index)))
                    .service(web::resource("/register").route(web::post().to(api::register_user)))
                    .service(web::resource("/user-name").route(web::post().to(api::get_username)))
                    .service(web::resource("/new").route(web::get().to(api::new_game)))
                    .service(web::resource("/find").route(web::get().to(api::find)))
                    .service(web::resource("/join").route(web::post().to(api::join)))
                    .service(
                        web::resource("/game-state").route(web::get().to(api::game_state_json)),
                    )
                    .service(web::resource("/wipe").route(web::get().to(api::delete_all)))
                    .service(web::resource("/do-action").route(web::post().to(api::make_action))),
            )
        // To mount a nice html webiste at index, do this and remove the default index fn above
        //.service(fs::Files::new("/", "./static").index_file("index.html"))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
