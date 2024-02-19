use serde::Deserialize;

#[derive(Deserialize)]
#[allow(dead_code)]
pub struct General {
    pub account: Account,
    pub auth_server: AuthServer,
    pub server: Server,
}

#[derive(Deserialize)]
#[allow(dead_code)]
pub struct Account {
    pub username: String,
    pub password: String,
}

#[derive(Deserialize)]
#[allow(dead_code)]
pub struct AuthServer {
    pub host: String,
    pub port: i64,
}

#[derive(Deserialize)]
#[allow(dead_code)]
pub struct Server {
    pub host: String,
    pub port: i64,
}
