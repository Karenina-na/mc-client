use serde::Deserialize;

#[derive(Deserialize)]
#[allow(dead_code)]
pub struct Log {
    pub log_level: String,
}
