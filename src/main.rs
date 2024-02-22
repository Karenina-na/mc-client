use crate::core::client::Client;
use crate::core::console;
use crate::yggdrasil::refresh;
use config::factory::Config;
use env_logger::{Builder, Target};
use log::{debug, error, info, warn};
use std::io;
use std::process::exit;
use tokio::sync::mpsc;
use yggdrasil::authenticate;

mod config;
mod core;
mod itti;
mod util;
mod yggdrasil;

#[tokio::main]
async fn main() {
    // config
    let config = match Config::load("conf/config.toml".to_string()).await {
        Ok(config) => config,
        Err(_) => {
            init_log("error".to_string());
            error!("load config failed");
            exit(0)
        }
    };
    init_log(config.log.log_level);

    let mut client;

    // yggdrasil
    match config.general.account.password.as_str() {
        "-" => {
            // offline login
            info!(
                "You are using offline login (username: {})",
                config.general.account.username
            );
            client = Client::new(config.general.account.username, 763);
        }
        "" => {
            // interactive login
            info!(
                "You are using interactive login (username: {}), please input your password:",
                config.general.account.username
            );
            let mut password = String::new();
            match io::stdin().read_line(&mut password) {
                Ok(_) => {
                    password = password.trim().to_string();
                }
                Err(e) => {
                    error!("read line failed: {}", e);
                    exit(0);
                }
            }
            let username = config.general.account.username.clone();
            let password = password.to_string();
            let url = config.general.auth_server.host.clone();
            let name = yggdrasil_login(url, username, password).await;
            if name == "" {
                error!("login failed");
                exit(0);
            }
            client = Client::new(name.clone(), 763);
            info!(
                "login {} using {}({}) success",
                config.general.auth_server.host.clone(),
                config.general.account.username.clone(),
                name
            );
        }
        password => {
            // password login
            info!(
                "You are using password login (username: {}), password: *******",
                config.general.account.username,
            );
            let username = config.general.account.username.clone();
            let password = password.to_string();
            let url = config.general.auth_server.host.clone();
            let name = yggdrasil_login(url, username, password).await;
            if name == "" {
                error!("login failed");
                exit(0);
            }
            client = Client::new(name.clone(), 763);
            info!(
                "login {} using {}({}) success",
                config.general.auth_server.host.clone(),
                config.general.account.username.clone(),
                name
            );
        }
    };

    // itti
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

fn init_log(level: String) {
    std::env::set_var("RUST_LOG", level);
    let mut builder = Builder::from_default_env();
    builder.target(Target::Stdout).init();
}

async fn yggdrasil_login(url: String, username: String, password: String) -> String {
    // authenticate
    match authenticate::send(url.clone(), username, password, true).await {
        Ok(response) => {
            match response.error {
                Some(e) => {
                    error!("login in {} failed: {}", url, e);
                    "".to_string()
                }
                None => match (
                    response.access_token,
                    response.client_token,
                    response.available_profiles,
                    response.user,
                ) {
                    (
                        Some(access_token),
                        Some(client_token),
                        Some(available_profiles),
                        Some(user),
                    ) => {
                        debug!("login in {} success", url);
                        debug!("access_token: {}", access_token);
                        debug!("client_token: {}", client_token);
                        debug!("user: {:?}", user);
                        info!("Available profiles:");
                        for profile in &available_profiles {
                            info!("id: {}, name: {}", profile.id.clone(), profile.name.clone());
                        }
                        let mut select_name = String::new();
                        let select_id;
                        'outer: loop {
                            // chose profile
                            info!("Please chose a profile: (name)");
                            match io::stdin().read_line(&mut select_name) {
                                Ok(_) => {
                                    select_name = select_name.trim().to_string();
                                    for profile in &available_profiles {
                                        if profile.name == select_name {
                                            select_id = profile.id.clone();
                                            break 'outer;
                                        }
                                    }
                                    error!("login in {} failed: profile not found", url);
                                }
                                Err(e) => {
                                    error!("read line failed: {}", e);
                                }
                            }
                        }
                        // send refresh to select profile
                        match refresh::send(
                            url.clone(),
                            access_token,
                            client_token,
                            true,
                            select_name,
                            select_id,
                        )
                        .await
                        {
                            Ok(response) => match response.error {
                                Some(e) => {
                                    error!("login in {} failed: {}", url, e);
                                    "".to_string()
                                }
                                None => match response.selected_profile {
                                    Some(profile) => {
                                        debug!("login in {} success", url);
                                        debug!("access_token: {}", response.access_token.unwrap());
                                        debug!("client_token: {}", response.client_token.unwrap());
                                        debug!("user: {:?}", response.user.unwrap());
                                        debug!("selected profile: {:?}", profile);
                                        profile.name
                                    }
                                    None => {
                                        error!("login in {} failed: unknown error", url);
                                        "".to_string()
                                    }
                                },
                            },
                            Err(e) => {
                                error!("login in {} failed: {}", url, e);
                                "".to_string()
                            }
                        }
                    }
                    _ => {
                        error!("login in {} failed: unknown error", url);
                        "".to_string()
                    }
                },
            }
        }
        Err(e) => {
            error!("login in {} failed: {}", url, e);
            "".to_string()
        }
    }
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
