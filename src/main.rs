mod relay;
mod schema;
mod services;
mod utils;

use clap::{Parser, Subcommand};
use schema::ConfigFile;
use utils::init_config;

use once_cell::sync::Lazy;
use std::sync::RwLock;

// const SSL_CERTIFICATE: &str = include_str!("../certs/ca.crt");
static CONFIG_FILE: Lazy<RwLock<Option<ConfigFile>>> = Lazy::new(|| RwLock::new(None));

#[derive(Parser)]
#[command(name = "tago-relay")]
#[command(about = "A CLI for managing the MQTT Relay service", long_about = None)]
struct Cli {
  #[command(subcommand)]
  command: Commands,
}

#[derive(Subcommand)]
enum Commands {
  #[command(
    about = "Initialize the configuration file",
    long_about = "Initialize the configuration file for the MQTT Relay service.\n\n\
                  This command creates a `config.toml` file containing the Relay parameters.\n\n\
                  Examples:\n\
                  - Initialize with default path:\n\
                    tago-relay init\n\
                  - Initialize with custom path:\n\
                    tago-relay init --config-path /path/to/config"
  )]
  Init {
    /// Path to the configuration file
    #[arg(short, long)]
    config_path: Option<String>,
  },
  #[command(
    about = "Start the MQTT Relay service",
    long_about = "Start the MQTT Relay service to create a bridge between TagoIO and customer brokers.\n\n\
                  This command starts the Relay, which is a client that will connect to the user Broker and send data to the TagoIO platform.\n\n\
                  The `config_path` option sets the path for the `config.toml` file. If not passed, it defaults to the environment variable `RELAY_CONFIG_PATH` or `$HOME/.config/tagoio-mqtt-relay.toml`.\n\n\
                  The `verbose` option accepts a string with types of logs: `info`, `error`, `mqtt`, `network`. Defaults to `info,error`.\n\n\
                  Examples:\n\
                  - Start with default configuration:\n\
                    tago-relay start\n\
                  - Start with verbose mode:\n\
                    tago-relay start --verbose info,mqtt\n\
                  - Start with custom configuration path:\n\
                    tago-relay start --config-path /path/to/config.toml"
  )]
  Start {
    /// Verbose mode (-v)
    #[arg(short, long)]
    verbose: Option<String>,

    /// Path to the configuration file
    #[arg(short, long)]
    config_path: Option<String>,
  },
}

#[tokio::main]
async fn main() {
  let cli = Cli::parse();

  match &cli.command {
    Commands::Init { config_path } => {
      init_config(config_path.as_deref());
    }
    Commands::Start { verbose, config_path } => {
      let log_level: String = verbose
        .as_ref()
        .map(|v| v.to_string())
        .unwrap_or_else(|| "error,info".to_string());

      env_logger::init_from_env(env_logger::Env::new().default_filter_or(log_level));

      let config = utils::fetch_config_file(config_path.clone());
      if let Some(config) = config {
        *CONFIG_FILE.write().unwrap() = Some(config);
      } else {
        log::error!("Failed to load configuration file.");
        std::process::exit(1);
      }

      if let Err(e) = relay::start_relay().await {
        log::error!("Error starting relay: {}", e);
      }
    }
  }
}
