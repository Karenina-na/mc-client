use crate::core::client::Client;
use crate::core::console;
use config::factory::Config;
use env_logger::{Builder, Target};
use lazy_static::lazy_static;
use log::{debug, error, warn};
use std::process::exit;
use tokio::sync::mpsc;

mod config;
mod core;
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
    let config = match Config::load("conf/config.toml".to_string()).await {
        Ok(config) => config,
        Err(_) => {
            error!("load config failed");
            exit(0)
        }
    };

    let mut client = Client::new(config.general.account.username, 763);
    let mut itti = itti::basis::ITTI::new(
        config.general.server.host,
        config.general.server.port.to_string(),
        config.buffer.tcp_buffer.reader as i32,
        config.buffer.tcp_buffer.writer as i32,
    );

    let (command_tx, mut command_rx) =
        mpsc::channel(config.buffer.console_client_buffer.command as usize); // command channel (Console -> Client)
    let (response_tx, response_rx) =
        mpsc::channel(config.buffer.console_client_buffer.response as usize); // response channel (Client -> Console)

    // start console
    match response_tx.send(vec!["reconnect".to_string()]).await {
        Ok(_) => {}
        Err(_) => {
            error!("init console failed");
            exit(0)
        }
    }
    console::build_console(command_tx, response_rx);

    // start client
    server_loop(&mut itti, &mut client, &mut command_rx, &response_tx).await;
}

async fn server_loop(
    itti: &mut itti::basis::ITTI,
    client: &mut Client,
    command_rx: &mut mpsc::Receiver<Vec<String>>,
    response_tx: &mpsc::Sender<Vec<String>>,
) {
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
        client.start(itti, command_rx, &response_tx).await;
        // reset
        client.reset();
        // stop
        itti.stop().await;
    }
}
