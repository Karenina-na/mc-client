use console::{style, Term};
use dialoguer::{BasicHistory, FuzzySelect, Input, Sort};
use log::{debug, error, info, warn};
use prettytable::format::consts::FORMAT_BOX_CHARS;
use prettytable::{row, Table};
use tokio::sync::mpsc;

pub fn build_console(
    command_tx: mpsc::Sender<Vec<String>>,
    mut response_rx: mpsc::Receiver<Vec<String>>,
) {
    // console -- io channel
    tokio::spawn(async move {
        let mut history = BasicHistory::new().max_entries(64).no_duplicates(true);
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
            let command =
                match Input::<String>::with_theme(&dialoguer::theme::ColorfulTheme::default())
                    .with_prompt("You")
                    .default("/help".to_string())
                    .history_with(&mut history)
                    .interact_text()
                {
                    Ok(command) => command,
                    Err(_) => {
                        info!("console quit");
                        break;
                    }
                };
            match command.trim() {
                "/quit" => {
                    // quit
                    match command_tx.send(vec![]).await {
                        Ok(_) => {}
                        Err(_) => {
                            info!("client already quit");
                        }
                    }
                    println!("console {}", style("quit").red());
                    break;
                }
                "/respawn" => {
                    // respawn
                    match command_tx.send(vec!["respawn".to_string()]).await {
                        Ok(_) => {}
                        Err(_) => {
                            info!("client already quit");
                        }
                    }
                }
                "/position" => {
                    // get position
                    match command_tx.send(vec!["position".to_string()]).await {
                        Ok(_) => {}
                        Err(_) => {
                            info!("client already quit");
                        }
                    }
                    match response_rx.recv().await {
                        Some(res) => {
                            info!("position: {:?}", res);
                            println!("position: {:?}", res);
                        }
                        None => {
                            info!("client already quit");
                        }
                    }
                }
                "/server" => {
                    // get server data
                    match command_tx.send(vec!["server".to_string()]).await {
                        Ok(_) => {}
                        Err(_) => {
                            info!("client already quit");
                        }
                    }
                    match response_rx.recv().await {
                        Some(res) => {
                            info!("server data: {:?}", res);
                            println!("server data: {:?}", res);
                        }
                        None => {
                            info!("client already quit");
                        }
                    }
                }
                "/time" => {
                    // get time
                    match command_tx.send(vec!["time".to_string()]).await {
                        Ok(_) => {}
                        Err(_) => {
                            info!("client already quit");
                        }
                    }
                    match response_rx.recv().await {
                        Some(res) => {
                            info!("time: {:?}", res);
                            println!("time: {:?}", res);
                        }
                        None => {
                            info!("client already quit");
                        }
                    }
                }
                "/tps" => {
                    // get tps
                    match command_tx.send(vec!["tps".to_string()]).await {
                        Ok(_) => {}
                        Err(_) => {
                            info!("client already quit");
                        }
                    }
                    match response_rx.recv().await {
                        Some(res) => {
                            info!("tps: {:?}", res);
                            println!("tps: {:?}", res);
                        }
                        None => {
                            info!("client already quit");
                        }
                    }
                }
                "/exp" => {
                    // get exp
                    match command_tx.send(vec!["exp".to_string()]).await {
                        Ok(_) => {}
                        Err(_) => {
                            info!("client already quit");
                        }
                    }
                    match response_rx.recv().await {
                        Some(res) => {
                            info!("exp: {:?}", res);
                            println!("exp: {:?}", res);
                        }
                        None => {
                            info!("client already quit");
                        }
                    }
                }
                "/health" => {
                    // get health
                    match command_tx.send(vec!["health".to_string()]).await {
                        Ok(_) => {}
                        Err(_) => {
                            info!("client already quit");
                        }
                    }
                    match response_rx.recv().await {
                        Some(res) => {
                            info!("health: {:?}", res);
                            println!("health: {:?}", res);
                        }
                        None => {
                            info!("client already quit");
                        }
                    }
                }
                "/help" => {
                    // help
                    let mut t = Table::new();
                    t.set_format(*FORMAT_BOX_CHARS);
                    t.set_titles(row![style("Command").blue(), style("Description").white()]);
                    t.add_row(row![style("/clear").yellow(), "Clear console"]);
                    t.add_row(row![style("/quit").yellow(), "Quit console"]);
                    t.add_row(row![style("/respawn").yellow(), "Respawn"]);
                    t.add_row(row![style("/position").yellow(), "Get position"]);
                    t.add_row(row![style("/server").yellow(), "Get server data"]);
                    t.add_row(row![style("/time").yellow(), "Get time"]);
                    t.add_row(row![style("/tps").yellow(), "Get tps"]);
                    t.add_row(row![style("/exp").yellow(), "Get exp"]);
                    t.add_row(row![style("/health").yellow(), "Get health"]);
                    t.add_row(row![style("chat message").yellow(), "Send message"]);
                    t.add_row(row![style("//command").yellow(), "Send server command"]);
                    t.printstd();
                }
                "/clear" => {
                    // clear
                    match Term::stdout().clear_screen() {
                        Ok(_) => {}
                        Err(_) => {
                            info!("client already quit");
                        }
                    }
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
                                Ok(_) => {}
                                Err(_) => {
                                    info!("client already quit");
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
                            Ok(_) => {}
                            Err(_) => {
                                info!("client already quit");
                            }
                        }
                    }
                }
            }
        }
    });
}

async fn reconnect_loop(command_tx: mpsc::Sender<Vec<String>>) -> bool {
    info!("client not connect, please input /help for more information");
    println!(
        "client {}, please input {} for more information",
        style("not connect").red(),
        style("/help").cyan()
    );
    const RECONNECT_COMMANDS: [&str; 3] = ["/help", "/quit", "/connect"];
    loop {
        match FuzzySelect::with_theme(&dialoguer::theme::ColorfulTheme::default())
            .default(0)
            .items(&RECONNECT_COMMANDS)
            .max_length(5)
            .interact()
        {
            Ok(0) => {
                // help --
                let mut t = Table::new();
                t.set_format(*FORMAT_BOX_CHARS);
                t.set_titles(row![style("Command").blue(), style("Description").white()]);
                t.add_row(row![style("/quit").yellow(), "Quit console"]);
                t.add_row(row![style("/connect").yellow(), "Connect to server"]);
                t.add_row(row![style("/help").yellow(), "Show help"]);
                t.printstd();
            }
            Ok(1) => {
                // quit
                match command_tx.send(vec![]).await {
                    Ok(_) => {}
                    Err(_) => {
                        info!("client already quit");
                        println!("client already {}", style("quit").red());
                    }
                }
                info!("console quit");
                println!("console {}", style("quit").red());
                return false;
            }
            Ok(2) => {
                // connect
                command_tx
                    .send(vec!["reconnect".to_string()])
                    .await
                    .unwrap();
                break;
            }
            _ => {
                error!("unknown command")
            }
        }
        debug!("client not connect, please input /reconnect");
    }
    true
}
