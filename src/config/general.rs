use lazy_static::lazy_static;
use serde::Deserialize;
use validator::Validate;

#[derive(Deserialize, Validate)]
pub struct General {
    #[validate]
    pub account: Account,
    #[validate]
    pub auth_server: AuthServer,
    #[validate]
    pub server: Server,
    #[validate(length(min = 5, max = 5))]
    pub lang: String,
}

#[derive(Deserialize, Validate)]
pub struct Account {
    #[validate(length(min = 3, max = 20))]
    pub username: String,
    pub password: String,
}

#[derive(Deserialize, Validate)]
pub struct AuthServer {
    #[validate(regex(path = "IP_DOMAIN_REGEX"))]
    pub host: String,
    #[validate(range(min = 1, max = 65535))]
    pub port: i64,
}

#[derive(Deserialize, Validate)]
pub struct Server {
    #[validate(regex(path = "IP_DOMAIN_REGEX"))]
    pub host: String,
    #[validate(range(min = 1, max = 65535))]
    pub port: i64,
}

lazy_static! {
    static ref IP_DOMAIN_REGEX: regex::Regex = regex::Regex::new(r"^[a-zA-Z0-9.-]+(:[0-9]+)?").unwrap();
}