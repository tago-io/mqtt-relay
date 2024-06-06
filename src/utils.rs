use std::time::Duration;
use toml_env::{initialize, Args, AutoMapEnvArgs};

use crate::schema::ConfigFile;

#[derive(serde::Serialize, serde::Deserialize, Default, Debug, Clone)]
struct ConfigFileResponse {
  relay: Option<ConfigFile>,
}

const DEFAULT_CONFIG: &str = include_str!("./default_config.toml");

pub fn init_config() {
  let env_config_path = std::env::var("CONFIG_PATH").ok();
  let config_path = env_config_path
    .as_deref()
    .map(std::path::PathBuf::from)
    .unwrap_or_else(|| {
      let home_dir = std::env::var("HOME").expect("HOME environment variable not set");
      let config_path_str = format!("{}/.config/.tagoio-mqtt-relay.toml", home_dir);
      std::path::PathBuf::from(config_path_str)
    });

  std::fs::write(&config_path, DEFAULT_CONFIG).expect("Failed to create default config file");

  println!("Configuration file created at {}", config_path.display());
}
pub fn fetch_config_file() -> Option<ConfigFile> {
  let env_config_path = std::env::var("CONFIG_PATH").ok();
  let config_path = env_config_path
    .as_deref()
    .map(std::path::PathBuf::from)
    .unwrap_or_else(|| {
      let home_dir = std::env::var("HOME").expect("HOME environment variable not set");
      let config_path_str = format!("{}/.config/.tagoio-mqtt-relay.toml", home_dir);
      std::path::PathBuf::from(config_path_str)
    });

  // If the config file doesn't exist, create it
  if !config_path.exists() {
    log::error!(target: "error", "Configuration file not found.");
    std::process::exit(1);
  }

  let config: Option<ConfigFileResponse> = initialize(Args {
    auto_map_env: Some(AutoMapEnvArgs::default()),
    // logging: Logging::StdOut,
    config_path: Some(&config_path),
    ..Args::default()
  })
  .unwrap_or_else(|err| {
    log::error!(target: "error", "Failed to initialize configuration: {}", err);
    std::process::exit(1);
  });

  config
    .unwrap_or_else(|| {
      log::error!(target: "error", "Configuration file is missing or invalid.");
      std::process::exit(1);
    })
    .relay
}

pub fn calculate_backoff(attempt: u32) -> Duration {
  let base_delay = Duration::from_secs(5);
  let max_delay = Duration::from_secs(60);
  let delay = base_delay * 2u32.pow(attempt);
  std::cmp::min(delay, max_delay)
}
