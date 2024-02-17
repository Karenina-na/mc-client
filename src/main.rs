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

    let mut client = client::client::Client::new("Karenina".to_string(), 763);
    let mut itti = itti::basis::ITTI::new("127.0.0.1".to_string(), "25565".to_string(), 2048, 2048);

    let (tx, mut rx) = mpsc::channel(256);

    // start console
    console::build_console(tx);

    // start client
    client.start(&mut itti, &mut rx).await;

    // stop
    itti.stop().await;
}
