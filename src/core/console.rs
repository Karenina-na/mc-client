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
                    if reconnect_loop(command_tx.clone()).await {
                        // clear channel
                        loop {
                            match response_rx.try_recv() {
                                Ok(_) => {}
                                Err(_) => {
                                    break;
                                }
                            }
                        }
                    } else {
                        // quit
                        break;
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
                                    debug!("respawn");
                                }
                                Err(_) => {
                                    info!("client already quit");
                                }
                            }
                        }
                        // position
                        "/position" => {
                            // get position
                            match command_tx.send(vec!["position".to_string()]).await {
                                Ok(_) => {
                                    debug!("get position");
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
                        "/server" => {
                            // get server data
                            match command_tx.send(vec!["server".to_string()]).await {
                                Ok(_) => {
                                    debug!("get server data");
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
                        // time
                        "/time" => {
                            // get time
                            match command_tx.send(vec!["time".to_string()]).await {
                                Ok(_) => {
                                    debug!("get time");
                                }
                                Err(_) => {
                                    info!("client already quit");
                                }
                            }
                            match response_rx.recv().await {
                                Some(res) => {
                                    info!("time: {:?}", res);
                                }
                                None => {
                                    info!("client already quit");
                                }
                            }
                        }
                        // tps
                        "/tps" => {
                            // get tps
                            match command_tx.send(vec!["tps".to_string()]).await {
                                Ok(_) => {
                                    debug!("get tps");
                                }
                                Err(_) => {
                                    info!("client already quit");
                                }
                            }
                            match response_rx.recv().await {
                                Some(res) => {
                                    info!("tps: {:?}", res);
                                }
                                None => {
                                    info!("client already quit");
                                }
                            }
                        }
                        // exp
                        "/exp" => {
                            // get exp
                            match command_tx.send(vec!["exp".to_string()]).await {
                                Ok(_) => {
                                    debug!("get exp");
                                }
                                Err(_) => {
                                    info!("client already quit");
                                }
                            }
                            match response_rx.recv().await {
                                Some(res) => {
                                    info!("exp: {:?}", res);
                                }
                                None => {
                                    info!("client already quit");
                                }
                            }
                        }
                        // health
                        "/health" => {
                            // get health
                            match command_tx.send(vec!["health".to_string()]).await {
                                Ok(_) => {
                                    debug!("get health");
                                }
                                Err(_) => {
                                    info!("client already quit");
                                }
                            }
                            match response_rx.recv().await {
                                Some(res) => {
                                    info!("health: {:?}", res);
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
                            info!("/position: get position");
                            info!("/server: get server data");
                            info!("/time: get time");
                            info!("/tps: get tps");
                            info!("/exp: get exp");
                            info!("/health: get health");
                            info!("chat message: send message");
                            info!("//command: send command");
                        }
                        msg => {
                            if msg.starts_with('/') {
                                // 两个//
                                if msg.starts_with("//") {
                                    // send message
                                    match command_tx
                                        .send(vec!["command".to_string(), msg[2..].to_string()])
                                        .await
                                    {
                                        Ok(_) => {
                                            debug!("send command: {}", msg);
                                        }
                                        Err(_) => {
                                            debug!("client already quit");
                                        }
                                    }
                                } else {
                                    info!("Unknown command: {}", msg);
                                }
                            } else {
                                // send message
                                match command_tx
                                    .send(vec!["chat".to_string(), msg.to_string()])
                                    .await
                                {
                                    Ok(_) => {
                                        debug!("send message: {}", msg);
                                    }
                                    Err(_) => {
                                        debug!("client already quit");
                                    }
                                }
                            }
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

async fn reconnect_loop(command_tx: mpsc::Sender<Vec<String>>) -> bool {
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
                        return false;
                    }
                    "/connect" => {
                        command_tx
                            .send(vec!["reconnect".to_string()])
                            .await
                            .unwrap();
                        break;
                    }
                    "/help" => {
                        // help
                        info!("/quit: quit");
                        info!("/connect: connect");
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
        debug!("client not connect, please input /reconnect");
    }
    true
}
