use actix_web::web::Form;
use actix_web::{test::TestRequest, http::header::ContentType};
// Tests of the api
use server::db::*;
use server::models::User;
use server::api::*;


use uuid::Uuid;
use actix_web::http::Method;
use actix_session::Session;

const SESSION_ID_KEY: &str = "session_id";
const USER_ID_KEY: &str = "user_id";



#[actix_web::test]
async fn api_create_user(){

    let pool = create_conn_pool();

    // Generate a request
    let req = TestRequest::default()
        .insert_header(ContentType::json())
        .method(Method::POST)
        .app_data(pool.clone())
        .to_http_request();


    
    let user1 = User{
        id: String::new(),
        user_name: "piggy".to_string()
    };

    // Create a session on the db
    let session_id = Uuid::new_v4();

    let me = actix_web::web::Form(user1);

    // need to figure out how sessions work
    //register_user(me, session_id, req);

    



}