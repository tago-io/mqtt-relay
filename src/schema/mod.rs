use std::error::Error;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct RelayConfig {
    pub id: String,
    pub config: ConfigFile,
    pub profile_id: Option<String>,
    pub state: Option<InitiatedState>,
}

#[derive(serde::Serialize, serde::Deserialize, Default, Debug, Clone)]
pub struct ConfigFile {
    pub network_token: String,
    pub authorization_token: String,
    pub tagoio_url: Option<String>, // Default is "https://api.tago.io"
    pub port: Option<String>,       // Default is "3000"
    pub mqtt: MQTT,
}
#[derive(serde::Serialize, serde::Deserialize, Default, Debug, Clone)]
pub struct MQTT {
    pub client_id: Option<String>, // Default is "tagoio-relay"
    pub tls_enabled: bool,
    pub address: String,
    pub port: u16,
    pub subscribe: Vec<String>,   // Default is ["/tago/#", "/device/+"]
    pub username: Option<String>, // Default is "my-username"
    pub password: Option<String>, // Default is "my-password"
    pub authentication_certificate_file: Option<String>, // Default is "certs/ca.crt"
}

impl RelayConfig {
    pub fn new_with_defaults(
        profile_id: Option<String>,
        config: ConfigFile,
    ) -> Result<Self, Box<dyn Error>> {
        // Ensure that profile_id and state are not None
        let id = "self-hosted".to_string();
        let profile_id = Some(profile_id.unwrap_or_else(|| "self-hosted".to_string()));
        let state: Option<InitiatedState> = Some(InitiatedState::Stopped);

        Ok(RelayConfig {
            id,
            config: config.with_defaults(),
            profile_id,
            state,
        })
    }

    pub fn is_running(&self) -> bool {
        matches!(self.state, Some(InitiatedState::Running))
    }
}

impl ConfigFile {
    pub fn with_defaults(mut self) -> Self {
        if self.tagoio_url.is_none() {
            self.tagoio_url = Some("https://api.tago.io".to_string());
        }
        if self.port.is_none() {
            self.port = Some("3000".to_string());
        }
        self.mqtt = self.mqtt.with_defaults();
        self
    }
}

impl MQTT {
    pub fn with_defaults(mut self) -> Self {
        if self.client_id.is_none() {
            self.client_id = Some("tagoio-relay".to_string());
        }
        if self.authentication_certificate_file.is_none() {
            self.authentication_certificate_file = Some("certs/ca.crt".to_string());
        }
        self
    }
}

#[derive(Debug, Clone, Deserialize)]
pub enum InitiatedState {
    Stopped,
    Running,
}

impl Default for InitiatedState {
    fn default() -> Self {
        InitiatedState::Stopped
    }
}

#[derive(Deserialize)]
pub struct PublishRequest {
    pub topic: String,
    pub message: String,
    pub bridge_id: String,
    pub qos: u8,
    pub retain: bool,
}
