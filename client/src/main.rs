#[tokio::main]
async fn main() {
    match client::ui::play_games().await {
        Ok(_) => (),
        Err(err) => panic!("Problem: {err}"),
    }
}
