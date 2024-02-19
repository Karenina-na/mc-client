use crate::config::buffer::Buffer;
use crate::config::general::General;
use crate::config::log::Log;
use serde::Deserialize;
use tokio::fs;

#[derive(Deserialize)]
#[allow(dead_code)]
pub(crate) struct Config {
    pub general: General,
    pub buffer: Buffer,
    pub log: Log,
}

impl Config {
    pub async fn load(path: String) -> Result<Config, String> {
        // toml解析
        match fs::read_to_string(path).await {
            Ok(toml) => match toml::from_str(&toml) {
                Ok(config) => Ok(config),
                Err(e) => Err(format!("config.toml parse failed {}", e.to_string())),
            },
            Err(e) => Err(format!("config.toml read failed: {}", e.to_string())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[tokio::test]
    async fn test_load() {
        let path = env::current_dir().unwrap().join("conf/config.toml");
        let config = Config::load(path.to_str().unwrap().to_string()).await;
        match config {
            Ok(_) => assert!(true),
            Err(e) => {
                println!("{}", e);
                assert!(false)
            }
        }
    }
}
