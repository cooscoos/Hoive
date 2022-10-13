#[tokio::main]
//#[actix_web::main]
async fn main() {
    match client::play_games().await {
        Ok(_) => (),
        Err(err) => panic!("Problem: {err}"),
    }
}