use crate::config::buffer::Buffer;
use crate::config::general::General;
use crate::config::log::Log;
use serde::Deserialize;
use tokio::fs;
use validator::Validate;

#[derive(Deserialize, Validate)]
#[allow(dead_code)]
pub(crate) struct Config {
    #[validate]
    pub general: General,
    #[validate]
    pub buffer: Buffer,
    #[validate]
    pub log: Log,
}

impl Config {
    pub async fn load(path: String) -> Result<Config, String> {
        // toml
        match fs::read_to_string(path).await {
            Ok(toml) => match toml::from_str::<Config>(&toml) {
                Ok(config) => {
                    // validate
                    match config.validate() {
                        Ok(_) => Ok(config),
                        Err(e) => Err(format!("config.toml validate failed: {}", e)),
                    }
                }
                Err(e) => Err(format!("config.toml parse failed {}", e)),
            },
            Err(e) => Err(format!("config.toml read failed: {}", e)),
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

    #[tokio::test]
    async fn test_load_validate() {
        let path = env::current_dir().unwrap().join("conf/config.toml");
        let config = Config::load(path.to_str().unwrap().to_string())
            .await
            .unwrap();
        match config.validate() {
            Ok(_) => assert!(true),
            Err(e) => {
                println!("{}", e);
                assert!(false)
            }
        }
    }
}
