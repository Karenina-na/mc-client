use crate::core::client::Client;
use crate::yggdrasil::refresh;
use config::factory::Config;
use console::style;
use dialoguer::{FuzzySelect, Password};
use env_logger::{Builder, Target};
use log::{debug, error, info, warn};
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
            println!(
                "You are using offline login (username: {})",
                style(config.general.account.username.clone()).yellow(),
            );
            client = Client::new(config.general.account.username, 763, config.general.lang);
        }
        "" => {
            // interactive login
            info!(
                "You are using interactive login (email: {}), please input your password:",
                config.general.account.username
            );
            let password = match Password::new()
                .with_prompt(format!(
                    "You are using interactive login (email: {}), please input your password",
                    style(config.general.account.username.clone()).yellow()
                ))
                .interact()
            {
                Ok(p) => p,
                Err(e) => {
                    error!("read line failed: {}", e);
                    exit(0);
                }
            };
            let username = config.general.account.username.clone();
            let url = config.general.auth_server.host.clone();
            let name = yggdrasil_login(url, username, password).await;
            if name == "" {
                error!("login failed");
                exit(0);
            }
            client = Client::new(name.clone(), 763, config.general.lang);
            info!(
                "login {} using {}({}) success",
                config.general.auth_server.host.clone(),
                config.general.account.username.clone(),
                name
            );
            println!(
                "login {} using {}({}) {}",
                style(config.general.auth_server.host.clone()).cyan(),
                style(config.general.account.username.clone()).yellow(),
                style(name).blue(),
                style("success").green(),
            );
        }
        password => {
            // password login
            info!(
                "You are using password login (username: {}), password: *******",
                config.general.account.username,
            );
            println!(
                "You are using password login (username: {}), password: *******",
                style(config.general.account.username.clone()).yellow(),
            );
            let username = config.general.account.username.clone();
            let password = password.to_string();
            let url = config.general.auth_server.host.clone();
            let name = yggdrasil_login(url, username, password).await;
            if name == "" {
                error!("login failed");
                exit(0);
            }
            client = Client::new(name.clone(), 763, config.general.lang);
            info!(
                "login {} using {}({}) success",
                config.general.auth_server.host.clone(),
                config.general.account.username.clone(),
                name
            );
            println!(
                "login {} using {}({}) {}",
                style(config.general.auth_server.host.clone()).cyan(),
                style(config.general.account.username.clone()).yellow(),
                style(name).blue(),
                style("success").green(),
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
    let (msg_tx, msg_rx) = mpsc::channel(16); // message channel (Client -> Display)

    // start console
    match response_tx.send(vec!["reconnect".to_string()]).await {
        Ok(_) => {}
        Err(_) => {
            error!("init console failed");
            exit(0)
        }
    }
    core::console::build_console(command_tx, response_rx);
    core::display::build_display(msg_rx);

    // start client
    server_loop(
        &mut itti,
        &mut client,
        &mut command_rx,
        &response_tx,
        &msg_tx,
    )
    .await;
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
                        println!("Available profiles: ");
                        for profile in &available_profiles {
                            info!("id: {}, name: {}", profile.id.clone(), profile.name.clone());
                            println!(
                                "id: {}, name: {}",
                                style(profile.id.clone()).blue(),
                                style(profile.name.clone()).green()
                            );
                        }
                        let select_name;
                        let select_id;
                        loop {
                            match FuzzySelect::with_theme(
                                &dialoguer::theme::ColorfulTheme::default(),
                            )
                            .with_prompt(format!(
                                "Please chose a profile to login in {}",
                                style(url.clone()).cyan()
                            ))
                            .items(
                                &available_profiles
                                    .iter()
                                    .map(|p| p.name.clone())
                                    .collect::<Vec<String>>(),
                            )
                            .interact()
                            {
                                Ok(i) => {
                                    select_name = available_profiles[i].name.clone();
                                    select_id = available_profiles[i].id.clone();
                                    break;
                                }
                                Err(_) => {
                                    error!(
                                        "login in {} failed: profile not found",
                                        style(url.clone()).cyan(),
                                    );
                                }
                            };
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
    msg_tx: &mpsc::Sender<Vec<String>>,
) {
    'outer: loop {
        // reconnect
        match response_tx.send(vec!["reconnect".to_string()]).await {
            Ok(_) => {}
            Err(_) => {
                debug!("console quit");
                break;
            }
        }
        loop {
            match command_rx.recv().await {
                Some(command) => {
                    if command.len() == 0 {
                        debug!("quit");

                        break 'outer;
                    }
                    if command == vec!["reconnect"] {
                        break;
                    } else {
                        warn!("Unknown command: {:?}", command)
                    }
                }
                None => {
                    warn!("client already quit");

                    break 'outer;
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
        client.start(itti, command_rx, &response_tx, &msg_tx).await;
        // reset
        client.reset();
        // stop
        itti.stop().await;
    }

    println!("bye!");
}
