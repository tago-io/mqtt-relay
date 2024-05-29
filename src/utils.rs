use std::time::Duration;
use toml_env::{initialize, Args, AutoMapEnvArgs, Logging};

use crate::schema::ConfigFile;

#[derive(serde::Serialize, serde::Deserialize, Default, Debug, Clone)]
struct ConfigFileResponse {
    relay: Option<ConfigFile>,
}

pub fn fetch_config_file() -> Option<ConfigFile> {
    let config_path = std::path::Path::new("./config.toml");

    let config: Option<ConfigFileResponse> = initialize(Args {
        auto_map_env: Some(AutoMapEnvArgs::default()),
        logging: Logging::StdOut,
        config_path: Some(&config_path),
        ..Args::default()
    })
    .unwrap()?;

    config.unwrap().relay
}

pub fn calculate_backoff(attempt: u32) -> Duration {
    let base_delay = Duration::from_secs(1);
    let max_delay = Duration::from_secs(32);
    let delay = base_delay * 2u32.pow(attempt);
    std::cmp::min(delay, max_delay)
}
