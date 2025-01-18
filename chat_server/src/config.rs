use std::{env, fs::File, path::PathBuf};

use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub auth: AuthConfig,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ServerConfig {
    pub port: u16,
    pub db_url: String,
    pub base_dir: PathBuf,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthConfig {
    pub sk: String,
    pub pk: String,
}

impl AppConfig {
    pub fn load() -> Result<Self> {
        // read from ./app.yml or /etc/config/app.yml or from env CHAT_CONFIG
        let ret = match (
            File::open("app.yml"),
            File::open("/etc/config/app.yml"),
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
