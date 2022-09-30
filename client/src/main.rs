#[tokio::main]
async fn main() {
    match client::play_games().await {
        Ok(_) => (),
        Err(err) => panic!("Problem: {err}"),
    }
}
