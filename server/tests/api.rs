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
            web::scope("/api").service(web::resource("/").route(web::get().to(api::index)))
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
async fn api_reguster_user() {
    // Try register a user at /api/register

    // Create test app
    let app = actix_web::test::init_service(
        App::new().app_data(db::create_conn_pool()).service(
            web::scope("/api")
                .service(web::resource("/register").route(web::post().to(api::register_user)))
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
    let result: String = test::read_body_json(response).await;

    // We won't know what Uuid is assigned, but we can make sure the response is a valid uuid
    match Uuid::parse_str(&result) {
         Ok(_) => {},
         Err(err) => panic!("{err}"),
    }
}

#[actix_web::test]
async fn api_reject_user_profanity() {
    // Try register a user with a naughty name at /api/register

    // Create test app
    let app = actix_web::test::init_service(
        App::new().app_data(db::create_conn_pool()).service(
            web::scope("/api")
                .service(web::resource("/register").route(web::post().to(api::register_user)))
        ),
    )
    .await;

    let pool = create_conn_pool();

    let naughty_user = User {
        id: String::new(),
        user_name: "piss".to_string(),
    };

    // Generate request to register new user
    let req = test::TestRequest::post()
        .uri("/api/register")
        .set_form(naughty_user)
        .app_data(pool.clone())
        .to_request();

    let response = test::call_service(&app, req).await;
    let result: String = test::read_body_json(response).await;

    assert_eq!("invalid", result);
}



#[actix_web::test]
async fn api_get_username() {
    // Try get a username for an existing user

    // Create test app
    let app = actix_web::test::init_service(
        App::new().app_data(db::create_conn_pool()).service(
            web::scope("/api")
                .service(web::resource("/register").route(web::post().to(api::register_user)))
                .service(web::resource("/user-name").route(web::post().to(api::get_username)))
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
    let user_id: String = test::read_body_json(response).await;


    // now request the username of the user we've just created
    let user_query = User {
        id: user_id,
        user_name: "".to_string(),
    };

    // Generate request to fetch username of the id we just created
    let req = test::TestRequest::post()
        .uri("/api/user-name")
        .set_form(user_query)
        .app_data(pool.clone())
        .to_request();

    let response = test::call_service(&app, req).await;
    let byte_result = test::read_body(response).await;
    let result = bytes_to_str(&byte_result).unwrap();


    assert_eq!(result, "piggy")

}




// To finish writing these you'll need to figure out how to manage sessions with cookies in tests.

// #[actix_web::test]
// async fn api_create_game() {
//     // Create a new game session, join it with one player

//     // Create test app
//     let app = actix_web::test::init_service(
//         App::new().app_data(db::create_conn_pool()).service(
//             web::scope("/api")
//             .service(web::resource("/new").route(web::get().to(api::new_game)))
//             .service(web::resource("/join").route(web::post().to(api::join)))
//         ),
//     )
//     .await;

//     let pool = create_conn_pool();

//     // We need a session that we can tie a user id to.
    
//     // Generate request
//     let req = test::TestRequest::get().uri("/api/new").app_data(pool.clone()).to_request();

//     // Send request to app and convert response body to str
//     let response = test::call_service(&app, req).await;
//     let byte_result = test::read_body(response).await;
//     let result = bytes_to_str(&byte_result).unwrap();

  
//     assert_eq!(format!("Hoive-server v{}", VERSION), result);

// }

// #[actix_web::test]
// async fn api_findjoin_game() {
//     // Find, then join a new game session

//     // Create test app
//     let app = actix_web::test::init_service(
//         App::new().app_data(db::create_conn_pool()).service(
//             web::scope("/api")
//             .service(web::resource("/new").route(web::get().to(api::new_game)))
//             .service(web::resource("/find").route(web::get().to(api::find)))
//             .service(web::resource("/join").route(web::post().to(api::join)))
//         ),
//     )
//     .await;

//     let pool = create_conn_pool();

// }


// #[actix_web::test]
// async fn api_get_game_state() {
//     // Get the game state of an existing session

//     // Create test app
//     let app = actix_web::test::init_service(
//         App::new().app_data(db::create_conn_pool()).service(
//             web::scope("/api")
//                 .service(web::resource("/new").route(web::get().to(api::new_game)))
//                 .service(
//                     web::resource("/game-state").route(web::get().to(api::game_state_json)),
//                 )
//         ),
//     )
//     .await;

//     let pool = create_conn_pool();

// }

// #[actix_web::test]
// async fn api_do_move() {
//     // Do a simple move, get the gamestate and ensure it's been updated

//     // Create test app
//     let app = actix_web::test::init_service(
//         App::new().app_data(db::create_conn_pool()).service(
//             web::scope("/api")
//                 .service(web::resource("/new").route(web::get().to(api::new_game)))
//                 .service(
//                     web::resource("/game-state").route(web::get().to(api::game_state_json)),
//                 )
//                 .service(web::resource("/do-action").route(web::post().to(api::make_action))),
//         ),
//     )
//     .await;

//     let pool = create_conn_pool();

// }

