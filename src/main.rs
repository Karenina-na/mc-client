use env_logger::{Builder, Target};
use lazy_static::lazy_static;
use log::info;
use std::process::exit;
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

    // console -- io channel
    tokio::spawn(async move {
        loop {
            let mut input = String::new();
            match std::io::stdin().read_line(&mut input) {
                Ok(_) => {}
                Err(e) => {
                    info!("Failed to read line: {}", e);
                    continue;
                }
            }

            match input.trim() {
                "/quit" => {
                    // quit
                    match tx.send(input.as_bytes().to_vec()).await {
                        Ok(_) => {}
                        Err(_) => {
                            info!("client already quit");
                        }
                    }
                    break;
                }
                "/respawn" => {
                    // respawn
                    let msg = client::msg::play::respawn::new();
                    match tx.send(msg).await {
                        Ok(_) => {}
                        Err(_) => {
                            info!("client already quit");
                        }
                    }
                }
                _ => {
                    info!("Unknown command: {}", input);
                }
            }
        }
    });

    client.start(&mut itti, &mut rx).await;

    // stop
    itti.stop().await;
}
