// use actix::prelude::*;

// /// Define a message
// #[derive(Message)]
// #[rtype(result = "Result<bool, std::io::Error>")]
// struct Ping;

// // Define an actor
// struct MyActor;

// // Provide actor implementation for our actor
// impl Actor for MyActor {
//     type Context = Context<Self>;

//     fn started(&mut self, ctx: &mut Context<Self>) {
//         println!("Actor is alive.");
//     }

//     fn stopped(&mut self, ctx: &mut Context<Self>) {
//         println!("Actor is dead.");
//     }
// }

// /// Define a handler for Ping message
// impl Handler<Ping> for MyActor {
//     type Result = Result<bool, std::io::Error>;

//     fn handle(&mut self, msg: Ping, ctx: &mut Context<Self>) -> Self::Result {
//         println!("Ping received");
//         Ok(true)
//     }
// }

use actix::{Actor, StreamHandler};
use actix_web::{get, web, App, Error, HttpRequest, HttpResponse, HttpServer};
use actix_web_actors::ws;

/// Define Websocket actor
struct MyWs;

impl Actor for MyWs {
    type Context = ws::WebsocketContext<Self>;
}

/// Handler for ws::Message message
impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for MyWs {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Ping(msg)) => ctx.pong(&msg),
            Ok(ws::Message::Text(text)) => ctx.text(text),
            Ok(ws::Message::Binary(bin)) => ctx.binary(bin),
            _ => (),
        }
    }
}

#[get("/ws")]
async fn index(req: HttpRequest, stream: web::Payload) -> Result<HttpResponse, Error> {
    ws::start(MyWs, &req, stream)
}