// use hoive::game::{board::Board, movestatus::MoveStatus};
// use hoive::maths::coord::Coord;
// use hoive::pmoore;

// fn main() {
//     play_game();
// }

// fn play_game() {
//     // Initialise game board in cube co-ordinates
//     let coord = hoive::maths::coord::Cube::default();
//     let mut board = Board::new(coord);

//     // Say hello, tell players who goes first
//     let first = pmoore::intro();

//     // Loop game until someone wins
//     loop {
//         if let MoveStatus::Win(_) = pmoore::take_turn(&mut board, first) {
//             println!("Play again? y/n");
//             let textin = pmoore::get_usr_input();
//             match textin {
//                 _ if textin == "y" => play_game(),
//                 _ => break,
//             }
//         }
//     }
// }

#[macro_use]
extern crate diesel;
extern crate dotenvy;

mod db;
use db::*;

#[macro_use]
extern crate serde_derive;
use actix_web::{web, App, HttpRequest, HttpResponse, HttpServer, Responder};

#[derive(Debug, Serialize, Deserialize)]
struct CreatePost {
    title: String,
    body: String,
}

fn create(post: web::Json<CreatePost>, req: HttpRequest) -> impl Responder {
    println!("request: {:?}", req);
    println!("model: {:?}", post);

    let result = create_user(post.0.title.as_ref(), post.0.body.as_ref());

    HttpResponse::Ok().json(result)
}

// fn index() -> impl Responder {
//     let posts = get_gamestates();
//     HttpResponse::Ok().json(posts)
// }

fn main() {
    HttpServer::new(|| {
        App::new()
            .data(web::JsonConfig::default().limit(4096))
            //      .route("/", web::get().to(index))
            .route("/create", web::post().to(create))
    })
    .bind("127.0.0.1:8880")
    .unwrap()
    .run()
    .unwrap();
}
