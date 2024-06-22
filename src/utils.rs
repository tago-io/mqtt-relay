use figment::{
  providers::{Env, Format, Toml},
  Figment,
};
use home::home_dir;
use std::time::Duration;

use crate::schema::ConfigFile;

#[derive(serde::Serialize, serde::Deserialize, Default, Debug, Clone)]
struct ConfigFileResponse {
  relay: Option<ConfigFile>,
}

const DEFAULT_CONFIG: &str = include_str!("./default_config.toml");

/**
 * Get the path to the configuration file
 */
fn get_config_path(user_path: Option<String>) -> std::path::PathBuf {
  let env_config_path = if user_path.is_none() {
    std::env::var("TAGOIO__RELAY__CONFIG_PATH").ok()
  } else {
    user_path
  };

  let config_path = env_config_path
    .as_deref()
    .map(std::path::PathBuf::from)
    .unwrap_or_else(|| {
      let home_dir = home_dir().unwrap_or_else(|| {
        log::warn!(target: "info", "Home directory is not set. Using current directory as fallback.");
        std::path::PathBuf::from(".")
      });
      let config_path_str = format!("{}/.config/.tagoio-mqtt-relay.toml", home_dir.display());
      std::path::PathBuf::from(config_path_str)
    });

  if config_path.is_dir() {
    config_path.join(".tagoio-mqtt-relay.toml")
  } else {
    config_path
  }
}

/**
 * Initialize the configuration file
 */
pub fn init_config(user_path: Option<impl AsRef<str>>) {
  let config_path = get_config_path(user_path.map(|s| s.as_ref().to_string()));
  if config_path.exists() {
    log::error!(target: "error", "Configuration file already exists: {}", config_path.display());
    std::process::exit(1);
  }

  let config_dir = config_path.parent().expect("Failed to get config directory");
  if !config_dir.exists() {
    log::info!(target: "info", "Creating config directory at {}", config_dir.display());
    std::fs::create_dir_all(config_dir).expect("Failed to create config directory");
  }

  std::fs::write(&config_path, DEFAULT_CONFIG).unwrap_or_else(|err| {
    log::error!(target: "error", "Failed to create default config file: {}", err);
    std::process::exit(1);
  });

  log::info!("Configuration file created at {}", config_path.display());
}

/**
 * Fetch the configuration file
 */
pub fn fetch_config_file(user_path: Option<String>) -> Option<ConfigFile> {
  let config_path = get_config_path(user_path);
  // If the config file doesn't exist, create it
  if !config_path.exists() {
    log::error!(target: "error", "Configuration file not found.");
    std::process::exit(1);
  }

  let figment = Figment::new()
    .merge(Toml::file(&config_path))
    .merge(Env::prefixed("TAGOIO__").split("__"));

  let config: ConfigFileResponse = figment.extract().unwrap_or_else(|err| {
    log::error!(target: "error", "Failed to initialize configuration: {}", err);
    std::process::exit(1);
  });

  config.relay
}

pub fn calculate_backoff(attempt: u32) -> Duration {
  let base_delay = Duration::from_secs(5);
  let max_delay = Duration::from_secs(60);
  let delay = base_delay * 2u32.pow(attempt);
  std::cmp::min(delay, max_delay)
}

#[cfg(test)]
mod tests {
  use super::*;
  use std::path::PathBuf;

  #[test]
  fn test_get_config_path_with_user_path() {
    let user_path = Some(String::from("/custom/path/config.toml"));
    let result = get_config_path(user_path);
    assert_eq!(result, PathBuf::from("/custom/path/config.toml"));
  }

  #[test]
  fn test_get_config_path_without_user_path() {
    let user_path = None;
    let result = get_config_path(user_path);
    let home_dir = home_dir().expect("Failed to get home directory");
    let expected_path = home_dir.join(".config/.tagoio-mqtt-relay.toml");

    assert_eq!(result, expected_path);
  }
}
