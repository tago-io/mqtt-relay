use serde::Deserialize;

use crate::utils::Authentication;

#[derive(Debug, Clone)]
pub struct BridgeConfig {
    pub id: String,
    pub address: String,
    pub version: String,
    pub port: u16,
    pub tls: bool,
    pub certificate: Option<String>,
    pub profile_id: Option<String>,
    pub state: Option<InitiatedState>,
    pub subscribe: Vec<String>,
    pub authentication: Authentication,
    pub network_token: String,
    pub authorization_token: String,
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
