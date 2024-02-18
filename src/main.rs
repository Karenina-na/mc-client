use crate::client::client::Client;
use crate::client::console;
use env_logger::{Builder, Target};
use lazy_static::lazy_static;
use tokio::sync::mpsc;

mod client;
mod itti;
mod util;

lazy_static! {
    static ref INIT: () = {
        std::env::set_var("RUST_LOG", "debug");
        let mut builder = Builder::from_default_env();
        builder.target(Target::Stdout);
        builder.is_test(true).init();
    };
}

#[tokio::main]
async fn main() {
    let _ = *INIT;

    let mut client = Client::new("Karenina".to_string(), 763);

    let mut itti = itti::basis::ITTI::new("127.0.0.1".to_string(), "25565".to_string(), 2048, 2048);

    let (command_tx, mut command_rx) = mpsc::channel(256); // command channel (Console -> Client)
    let (response_tx, response_rx) = mpsc::channel(256); // response channel (Client -> Console)

    // start console
    console::build_console(command_tx, response_rx);

    // start client
    client.start(&mut itti, &mut command_rx, &response_tx).await;

    // stop
    itti.stop().await;
}
