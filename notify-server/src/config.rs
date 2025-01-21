use std::{env, fs::File};

use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct NotifyConfig {
    pub server: ServerConfig,
    pub auth: AuthConfig,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ServerConfig {
    pub port: u16,
    pub db_url: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthConfig {
    pub pk: String,
}

impl NotifyConfig {
    pub fn load() -> Result<Self> {
        // read from ./notify.yml or /etc/config/notify.yml or from env CHAT_CONFIG
        let ret = match (
            File::open("notify.yml"),
            File::open("/etc/config/notify.yml"),
            env::var("CHAT_CONFIG"),
        ) {
            (Ok(file), _, _) => serde_yaml::from_reader(file),
            (_, Ok(file), _) => serde_yaml::from_reader(file),
            (_, _, Ok(file)) => serde_yaml::from_str(&file),
            _ => bail!("Config file not found"),
        };

        Ok(ret?)
    }
}
