use log::{debug, info, warn};
use tokio::sync::mpsc;

pub fn build_console(
    command_tx: mpsc::Sender<Vec<String>>,
    mut response_rx: mpsc::Receiver<Vec<String>>,
) {
    // console -- io channel
    tokio::spawn(async move {
        loop {
            // not connect
            let result = response_rx.try_recv();
            match result {
                // reconnect
                Ok(res) if res == vec!["reconnect"] => {
                    info!("client not connect, please input /help for more information");
                    loop {
                        // reconnect
                        let mut input = String::new();
                        match std::io::stdin().read_line(&mut input) {
                            Ok(_) => {
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
                                        return;
                                    }
                                    "/reconnect" => {
                                        command_tx
                                            .send(vec!["reconnect".to_string()])
                                            .await
                                            .unwrap();
                                        // clear channel
                                        loop {
                                            match response_rx.try_recv() {
                                                Ok(_) => {}
                                                Err(_) => {
                                                    break;
                                                }
                                            }
                                        }
                                        break;
                                    }
                                    "/help" => {
                                        // help
                                        info!("/quit: quit");
                                        info!("/reconnect: reconnect");
                                    }
                                    "" => {
                                        // empty
                                    }
                                    _ => {
                                        info!("Unknown command: {}", input);
                                    }
                                }
                            }
                            Err(e) => {
                                info!("Failed to read line: {}", e);
                                continue;
                            }
                        }
                    }
                }
                // unknown response
                Ok(res) => {
                    warn!("unknown response: {:?}", res);
                }
                // not connect
                Err(_) => {
                    debug!("client already connect");
                }
            }
            // connect
            let mut input = String::new();
            match std::io::stdin().read_line(&mut input) {
                Ok(_) => {
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
                Err(e) => {
                    info!("Failed to read line: {}", e);
                    continue;
                }
            }
        }
    });
}
