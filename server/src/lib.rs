#[macro_use]
extern crate diesel;
extern crate dotenvy;

use std::{
    sync::{
        atomic::{AtomicUsize},
        Arc,
    }
};

use actix_session::{storage::CookieSessionStore, SessionMiddleware};
use actix_web::{middleware, cookie::Key, web, App, HttpServer};
use actix::Actor;

pub mod api;
pub mod db;
pub mod models;
pub mod schema;
pub mod chat_session;
pub mod chat_server;


pub const VERSION: &str = env!("CARGO_PKG_VERSION");

fn get_secret_key() -> Key {
    Key::generate()
}


#[actix_web::main]
pub async fn start_server() -> std::io::Result<()> {

    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
    log::info!("starting HTTP server at http://localhost:8080");

    // set up applications state
    // keep a count of the number of visitors
    //let app_state = Arc::new(AtomicUsize::new(0));
    // start chat server actor
    //let servery = chat_server::ChatServer::new().start();

    let secret_key = get_secret_key();

    HttpServer::new(move || {
        App::new()
        //.app_data(web::Data::new(servery.clone()))
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
                    //.service(web::resource("/ws").route(web::get().to(api::chat_route)))
                    .service(web::resource("/wipe").route(web::get().to(api::delete_all)))
                    .service(web::resource("/do-action").route(web::post().to(api::make_action)))
            )
            .wrap(middleware::Logger::default())
        // To mount a nice html webiste at index, do this and remove the default index fn above
        //.service(fs::Files::new("/", "./static").index_file("index.html"))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
