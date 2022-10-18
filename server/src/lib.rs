#[macro_use]
extern crate diesel;
extern crate dotenvy;

use std::sync::{atomic::AtomicUsize, Arc};

use actix::Actor;
use actix_session::{storage::CookieSessionStore, SessionMiddleware};
use actix_web::{cookie::Key, middleware, web, App, HttpServer};

pub mod api;
pub mod chat_server;
pub mod chat_session;
pub mod db;
pub mod models;
pub mod schema;

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
    let servery = chat_server::ChatServer::new().start();

    let secret_key = get_secret_key();

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(servery.clone()))
            .app_data(db::create_conn_pool())
            .wrap(SessionMiddleware::new(
                CookieSessionStore::default(),
                secret_key.clone(),
            ))
            .service(
                web::scope("/api")
                    .service(web::resource("/").route(web::get().to(api::index)))
                    .service(web::resource("/ws").route(web::get().to(api::chat_route)))
            )
            .wrap(middleware::Logger::default())
        // To mount a nice html webiste at index, do this and remove the default index fn above
        //.service(fs::Files::new("/", "./static").index_file("index.html"))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
