use std::time::Duration;
use toml_env::{initialize, Args, AutoMapEnvArgs};

use crate::schema::ConfigFile;

#[derive(serde::Serialize, serde::Deserialize, Default, Debug, Clone)]
struct ConfigFileResponse {
  relay: Option<ConfigFile>,
}

pub fn fetch_config_file() -> Option<ConfigFile> {
  let env_config_path = std::env::var("CONFIG_PATH").ok();
  let config_path = env_config_path
    .as_deref()
    .map(std::path::Path::new)
    .unwrap_or_else(|| std::path::Path::new("./config.toml"));

  let config: Option<ConfigFileResponse> = initialize(Args {
    auto_map_env: Some(AutoMapEnvArgs::default()),
    // logging: Logging::StdOut,
    config_path: Some(&config_path),
    ..Args::default()
  })
  .unwrap_or_else(|err| {
    eprintln!("Failed to initialize configuration: {}", err);
    std::process::exit(1);
  });

  config
    .unwrap_or_else(|| {
      eprintln!("Configuration file is missing or invalid.");
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
