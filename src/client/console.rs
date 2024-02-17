use crate::client;
use log::info;

pub fn build_console(tx: tokio::sync::mpsc::Sender<Vec<u8>>) {
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
                    match tx.send(vec![]).await {
                        Ok(_) => {
                            info!("console quit");
                        }
                        Err(_) => {
                            info!("console quit");
                            info!("client already quit");
                        }
                    }
                    break;
                }
                "/respawn" => {
                    // respawn
                    let msg = client::msg::play::respawn::new(true);
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
}
