use lazy_static::lazy_static;
use serde::Deserialize;
use validator::Validate;

#[derive(Deserialize, Validate)]
pub struct Log {
    #[validate(regex(path = "LOG_LEVEL"))]
    pub log_level: String,
}

lazy_static! {
    static ref LOG_LEVEL: regex::Regex = regex::Regex::new(r"^(debug|info|warn|error)$").unwrap();
}