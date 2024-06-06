mod relay;
mod schema;
mod services;
mod utils;

use clap::{Parser, Subcommand};
use schema::ConfigFile;
use utils::init_config;

use once_cell::sync::Lazy;

// const SSL_CERTIFICATE: &str = include_str!("../certs/ca.crt");
static CONFIG_FILE: Lazy<Option<ConfigFile>> = Lazy::new(|| utils::fetch_config_file());

#[derive(Parser)]
#[command(name = "tago-relay")]
#[command(about = "A CLI for managing the MQTT Relay service", long_about = None)]
struct Cli {
  #[command(subcommand)]
  command: Commands,
}

#[derive(Subcommand)]
enum Commands {
  /// Initialize the configuration file
  Init,
  /// Start the MQTT Relay service
  Start {
    /// Verbose mode (-v)
    #[arg(short, long)]
    verbose: bool,
  },
}

#[tokio::main]
async fn main() {
  let cli = Cli::parse();

  match &cli.command {
    Commands::Init => {
      init_config();
    }
    Commands::Start { verbose } => {
      if let Err(e) = relay::start_relay(*verbose).await {
        eprintln!("Error starting relay: {}", e);
      }
    }
  }
}
