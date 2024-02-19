use crate::client::client::Client;
use crate::client::console;
use env_logger::{Builder, Target};
use lazy_static::lazy_static;
use log::{debug, error, warn};
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
    // init
    let mut client = Client::new("Karenina".to_string(), 763);
    let mut itti = itti::basis::ITTI::new("127.0.0.1".to_string(), "25565".to_string(), 2048, 2048);

    let (command_tx, mut command_rx) = mpsc::channel(256); // command channel (Console -> Client)
    let (response_tx, response_rx) = mpsc::channel(256); // response channel (Client -> Console)

    // start console
    match response_tx.send(vec!["reconnect".to_string()]).await {
        Ok(_) => {}
        Err(_) => {
            error!("init console failed");
            exit(0)
        }
    }
    console::build_console(command_tx, response_rx);

    loop {
        // reconnect
        match response_tx.send(vec!["reconnect".to_string()]).await {
            Ok(_) => {}
            Err(_) => {
                debug!("console quit");
                exit(0)
            }
        }
        loop {
            match command_rx.recv().await {
                Some(command) => {
                    if command.len() == 0 {
                        debug!("quit");
                        exit(0);
                    }
                    if command == vec!["reconnect"] {
                        break;
                    } else {
                        warn!("Unknown command: {:?}", command)
                    }
                }
                None => {
                    warn!("client already quit");
                    exit(0);
                }
            }
        }
        // clear channel
        loop {
            match command_rx.try_recv() {
                Ok(_) => {}
                Err(_) => {
                    break;
                }
            }
        }
        // start client
        client.start(&mut itti, &mut command_rx, &response_tx).await;
        // reset
        client.reset();
        // stop
        itti.stop().await;
    }
}
