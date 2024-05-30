use std::error::Error;

use regex::Regex;

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
    pub api_port: Option<String>,   // Default is "3000"
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

    // pub fn is_running(&self) -> bool {
    //     matches!(self.state, Some(InitiatedState::Running))
    // }
}

impl ConfigFile {
    pub fn with_defaults(mut self) -> Self {
        if self.tagoio_url.is_none() {
            self.tagoio_url = Some("https://api.tago.io".to_string());
        }
        if self.api_port.is_none() {
            self.api_port = Some("3000".to_string());
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

        if !self.is_valid_address(&self.address) {
            panic!("Invalid MQTT address: {}", self.address);
        }

        self
    }

    fn is_valid_address(&self, address: &str) -> bool {
        let re =
            Regex::new(r"^(?:(?:[a-zA-Z0-9-]+\.)+[a-zA-Z]{2,}|(?:\d{1,3}\.){3}\d{1,3}|localhost)$")
                .unwrap();
        re.is_match(address)
    }
}

#[derive(Debug, Clone, serde::Deserialize)]
pub enum InitiatedState {
    Stopped,
    Running,
}

impl Default for InitiatedState {
    fn default() -> Self {
        InitiatedState::Stopped
    }
}

#[derive(serde::Deserialize)]
pub struct PublishRequest {
    pub topic: String,
    pub message: String,
    pub bridge_id: String,
    pub qos: u8,
    pub retain: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_relay_config_new_with_defaults() {
        let config = ConfigFile {
            network_token: "network_token".to_string(),
            authorization_token: "authorization_token".to_string(),
            tagoio_url: None,
            api_port: None,
            mqtt: MQTT {
                client_id: None,
                tls_enabled: false,
                address: "localhost".to_string(),
                port: 1883,
                subscribe: vec![],
                username: None,
                password: None,
                authentication_certificate_file: None,
            },
        };

        let relay_config = RelayConfig::new_with_defaults(None, config).unwrap();

        assert_eq!(relay_config.id, "self-hosted");
        assert_eq!(relay_config.profile_id.unwrap(), "self-hosted");
        // assert_eq!(relay_config.state.unwrap(), InitiatedState::Stopped);
        assert_eq!(
            relay_config.config.tagoio_url.unwrap(),
            "https://api.tago.io"
        );
        assert_eq!(relay_config.config.api_port.unwrap(), "3000");
        assert_eq!(relay_config.config.mqtt.client_id.unwrap(), "tagoio-relay");
        assert_eq!(
            relay_config
                .config
                .mqtt
                .authentication_certificate_file
                .unwrap(),
            "certs/ca.crt"
        );
    }

    #[test]
    fn test_config_file_with_defaults() {
        let config = ConfigFile {
            network_token: "network_token".to_string(),
            authorization_token: "authorization_token".to_string(),
            tagoio_url: None,
            api_port: None,
            mqtt: MQTT {
                client_id: None,
                tls_enabled: false,
                address: "localhost".to_string(),
                port: 1883,
                subscribe: vec![],
                username: None,
                password: None,
                authentication_certificate_file: None,
            },
        };

        let config_with_defaults = config.with_defaults();

        assert_eq!(
            config_with_defaults.tagoio_url.unwrap(),
            "https://api.tago.io"
        );
        assert_eq!(config_with_defaults.api_port.unwrap(), "3000");
        assert_eq!(config_with_defaults.mqtt.client_id.unwrap(), "tagoio-relay");
        assert_eq!(
            config_with_defaults
                .mqtt
                .authentication_certificate_file
                .unwrap(),
            "certs/ca.crt"
        );
    }

    #[test]
    fn test_mqtt_with_defaults() {
        let mqtt = MQTT {
            client_id: None,
            tls_enabled: false,
            address: "localhost".to_string(),
            port: 1883,
            subscribe: vec![],
            username: None,
            password: None,
            authentication_certificate_file: None,
        };

        let mqtt_with_defaults = mqtt.with_defaults();

        assert_eq!(mqtt_with_defaults.client_id.unwrap(), "tagoio-relay");
        assert_eq!(
            mqtt_with_defaults.authentication_certificate_file.unwrap(),
            "certs/ca.crt"
        );
    }

    #[test]
    #[should_panic(expected = "Invalid MQTT address: invalid_address")]
    fn test_invalid_mqtt_address() {
        let mqtt = MQTT {
            client_id: None,
            tls_enabled: false,
            address: "invalid_address".to_string(),
            port: 1883,
            subscribe: vec![],
            username: None,
            password: None,
            authentication_certificate_file: None,
        };

        mqtt.with_defaults();
    }

    #[test]
    fn test_is_valid_address() {
        let mqtt = MQTT {
            client_id: None,
            tls_enabled: false,
            address: "localhost".to_string(),
            port: 1883,
            subscribe: vec![],
            username: None,
            password: None,
            authentication_certificate_file: None,
        };

        assert!(mqtt.is_valid_address("localhost"));
        assert!(mqtt.is_valid_address("192.168.1.1"));
        assert!(mqtt.is_valid_address("example.com"));
        assert!(!mqtt.is_valid_address("invalid_address"));
    }
}
