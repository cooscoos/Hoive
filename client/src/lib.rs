// Version of the client, should track with the version of the server.
//pub const VERSION: &str = "0.1.0";
const VERSION: &str = env!("CARGO_PKG_VERSION");

pub mod comms;
pub mod ui;
