use log::info;
use tokio::sync::mpsc;

pub fn build_console(
    command_tx: mpsc::Sender<Vec<String>>,
    mut response_rx: mpsc::Receiver<Vec<String>>,
) {
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
                    match command_tx.send(vec![]).await {
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
                // mc command
                "/respawn" => {
                    // respawn
                    match command_tx.send(vec!["respawn".to_string()]).await {
                        Ok(_) => {
                            info!("respawn");
                        }
                        Err(_) => {
                            info!("client already quit");
                        }
                    }
                }
                // console
                "/getPosition" => {
                    // get position
                    match command_tx.send(vec!["getPosition".to_string()]).await {
                        Ok(_) => {
                            info!("get position");
                        }
                        Err(_) => {
                            info!("client already quit");
                        }
                    }
                    match response_rx.recv().await {
                        Some(res) => {
                            info!("position: {:?}", res);
                        }
                        None => {
                            info!("client already quit");
                        }
                    }
                }
                // server data
                "/getServerData" => {
                    // get server data
                    match command_tx.send(vec!["getServerData".to_string()]).await {
                        Ok(_) => {
                            info!("get server data");
                        }
                        Err(_) => {
                            info!("client already quit");
                        }
                    }
                    match response_rx.recv().await {
                        Some(res) => {
                            info!("server data: {:?}", res);
                        }
                        None => {
                            info!("client already quit");
                        }
                    }
                }
                "" => {
                    // empty
                }
                "/help" => {
                    // help
                    info!("/quit: quit");
                    info!("/respawn: respawn");
                    info!("/getPosition: get position");
                    info!("/getServerData: get server data");
                }
                _ => {
                    info!("Unknown command: {}", input);
                }
            }
        }
    });
}
