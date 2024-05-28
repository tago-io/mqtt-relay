use anyhow::{Context, Result};
use serde::{de::value::Error, Deserialize, Serialize};
use std::{str::FromStr, time::Duration};
use toml_env::{initialize, Args, AutoMapEnvArgs, Logging, TomlKeyPath};

pub fn calculate_backoff(attempt: u32) -> Duration {
    let base_delay = Duration::from_secs(1);
    let max_delay = Duration::from_secs(32);
    let delay = base_delay * 2u32.pow(attempt);
    std::cmp::min(delay, max_delay)
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ConfigFile {
    pub tagoio: TagoIO,
    pub config: Config,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TagoIO {
    pub network_token: String,
    pub authorization: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub tls_enabled: bool,
    pub address: String,
    pub port: u16,
    pub authentication: Authentication,
    pub subscribe: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Authentication {
    pub client_id: Option<String>,
    pub username: String,
    pub password: String,
}

pub fn fetch_config_file() -> Option<ConfigFile> {
    let config_path = std::path::Path::new("./config.toml");

    let config: Option<ConfigFile> = initialize(Args {
        auto_map_env: Some(AutoMapEnvArgs::default()),
        config_path: Some(&config_path),
        ..Args::default()
    })
    .unwrap()?;

    config
}
