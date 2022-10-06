use actix_web::web::Form;
use actix_web::{http::header::ContentType, test};
// Tests of the api
use server::api::*;
use server::models::User;
use server::*;
use server::{db::*, start_server};

use actix_session::Session;
use actix_web::http::Method;
use uuid::Uuid;

use actix_session::{storage::CookieSessionStore, SessionMiddleware};
use actix_web::{cookie::Key, web, App, HttpServer};

const SESSION_ID_KEY: &str = "session_id";
const USER_ID_KEY: &str = "user_id";

mod common;
use common::testfns::bytes_to_str;

#[actix_web::test]
async fn api_get_index() {
    // Basic test to get the response from /api/

    // Create test app
    let app =
        actix_web::test::init_service(App::new().app_data(db::create_conn_pool()).service(
            web::scope("/api").service(web::resource("/").route(web::get().to(api::index))),
        ))
        .await;

    // Generate request
    let req = test::TestRequest::get().uri("/api/").to_request();

    // Send request to app and convert response body to str
    let response = test::call_service(&app, req).await;
    let byte_result = test::read_body(response).await;
    let result = bytes_to_str(&byte_result).unwrap();

    assert_eq!(format!("Hoive-server v{}", VERSION), result);
}

#[actix_web::test]
async fn api_create_user() {
    // Try register a user at /api/register

    // Create test app
    let app = actix_web::test::init_service(
        App::new().app_data(db::create_conn_pool()).service(
            web::scope("/api")
                .service(web::resource("/register").route(web::post().to(api::register_user))),
        ),
    )
    .await;

    let pool = create_conn_pool();

    let user1 = User {
        id: String::new(),
        user_name: "piggy".to_string(),
    };

    // Generate request to register new user
    let req = test::TestRequest::post()
        .uri("/api/register")
        .set_form(user1)
        .app_data(pool.clone())
        .to_request();

    let response = test::call_service(&app, req).await;
    let result:String = test::read_body_json(response).await;

    // We won't know what Uuid is assigned, but we can make sure the response is a valid uuid
    match Uuid::parse_str(&result) {
         Ok(_) => {},
         Err(err) => panic!("{err}"),
    }
}
