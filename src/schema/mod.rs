use crate::services::tagoio::verify_network_token;

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct RelayConfig {
  pub id: String,
  pub config: ConfigFile,
  pub profile_id: Option<String>,
  pub network_id: Option<String>,
}

#[derive(serde::Serialize, serde::Deserialize, Default, Debug, Clone)]
pub struct ConfigFile {
  pub network_token: String,
  pub authorization_token: String,
  pub tagoio_url: Option<String>, // Default is "https://api.tago.io"
  pub downlink_port: Option<u16>, // Default is "3000"
  pub mqtt: Mqtt,
}

#[derive(serde::Serialize, serde::Deserialize, Default, Debug, Clone)]
pub struct Mqtt {
  pub client_id: Option<String>, // Default is "tagoio-relay"
  pub tls_enabled: bool,
  pub address: String,
  pub port: u16,
  pub subscribe: Vec<String>,          // Default is ["/tago/#", "/device/+"]
  pub username: Option<String>,        // Default is "my-username"
  pub password: Option<String>,        // Default is "my-password"
  pub broker_tls_ca: Option<String>,   // Default is "certs/ca.crt"
  pub broker_tls_cert: Option<String>, // Default is "certs/client.crt"
  pub broker_tls_key: Option<String>,  // Default is "certs/client.key"
}

impl RelayConfig {
  pub fn new_with_defaults(profile_id: Option<String>, config: ConfigFile) -> anyhow::Result<Self> {
    // Ensure that profile_id and state are not None
    let id = "self-hosted".to_string();
    let profile_id = Some(profile_id.unwrap_or_else(|| "self-hosted".to_string()));

    Ok(RelayConfig {
      id,
      config: config.with_defaults()?,
      profile_id,
      network_id: None,
    })
  }

  pub async fn verify(&mut self) -> anyhow::Result<()> {
    log::info!(target: "network", "Verifying network token for relay: {}", self.id);
    match verify_network_token(self).await {
      Ok(verified_id) => {
        self.network_id = Some(verified_id);
        Ok(())
      }
      Err(e) => Err(e.into()),
    }
  }
}

impl ConfigFile {
  pub fn with_defaults(mut self) -> anyhow::Result<Self> {
    if self.tagoio_url.is_none() {
      self.tagoio_url = Some("https://api.tago.io".to_string());
    }
    if self.downlink_port.is_none() {
      self.downlink_port = Option::from(3000);
    }
    self.mqtt = self.mqtt.with_defaults()?;
    Ok(self)
  }
}

impl Mqtt {
  pub fn with_defaults(mut self) -> anyhow::Result<Self> {
    if self.client_id.is_none() {
      self.client_id = Some("tagoio-relay".to_string());
    }
    Ok(self)
  }
}

#[derive(serde::Deserialize)]
#[allow(dead_code)]
pub struct PublishRequest {
  pub topic: String,
  pub message: String,
  pub relay_id: String,
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
      downlink_port: None,
      mqtt: Mqtt {
        client_id: None,
        tls_enabled: false,
        address: "localhost".to_string(),
        port: 1883,
        subscribe: vec![],
        username: None,
        password: None,
        broker_tls_ca: None,
        broker_tls_cert: None,
        broker_tls_key: None,
      },
    };

    let relay_config = RelayConfig::new_with_defaults(None, config).unwrap();

    assert_eq!(relay_config.id, "self-hosted");
    assert_eq!(relay_config.profile_id.unwrap(), "self-hosted");
    // assert_eq!(relay_config.state.unwrap(), InitiatedState::Stopped);
    assert_eq!(relay_config.config.tagoio_url.unwrap(), "https://api.tago.io");
    assert_eq!(relay_config.config.downlink_port.unwrap(), 3000);
    assert_eq!(relay_config.config.mqtt.client_id.unwrap(), "tagoio-relay");
    // assert_eq!(
    //   relay_config.config.mqtt.authentication_certificate_file.unwrap(),
    //   "certs/ca.crt"
    // );
  }

  #[test]
  fn test_config_file_with_defaults() {
    let config = ConfigFile {
      network_token: "network_token".to_string(),
      authorization_token: "authorization_token".to_string(),
      tagoio_url: None,
      downlink_port: None,
      mqtt: Mqtt {
        client_id: None,
        tls_enabled: false,
        address: "localhost".to_string(),
        port: 1883,
        subscribe: vec![],
        username: None,
        password: None,
        broker_tls_ca: None,
        broker_tls_cert: None,
        broker_tls_key: None,
      },
    };

    let config_with_defaults = config.with_defaults().unwrap();

    assert_eq!(config_with_defaults.tagoio_url.unwrap(), "https://api.tago.io");
    assert_eq!(config_with_defaults.downlink_port.unwrap(), 3000);
    assert_eq!(config_with_defaults.mqtt.client_id.unwrap(), "tagoio-relay");
  }

  #[test]
  fn test_mqtt_with_defaults() {
    let mqtt = Mqtt {
      client_id: None,
      tls_enabled: false,
      address: "localhost".to_string(),
      port: 1883,
      subscribe: vec!["/tago/#".to_string(), "/device/+".to_string()],
      username: None,
      password: None,
      broker_tls_ca: None,
      broker_tls_cert: None,
      broker_tls_key: None,
    };

    let mqtt_with_defaults = mqtt.with_defaults().unwrap();

    assert_eq!(mqtt_with_defaults.client_id.unwrap(), "tagoio-relay");
  }

  // #[test]
  // fn test_is_valid_address() {
  //   let mqtt = MQTT {
  //     client_id: None,
  //     tls_enabled: false,
  //     address: "localhost".to_string(),
  //     port: 1883,
  //     subscribe: vec![],
  //     username: None,
  //     password: None,
  //     authentication_certificate_file: None,
  //   };

  //   assert!(mqtt.is_valid_address("localhost"));
  //   assert!(mqtt.is_valid_address("192.168.1.1"));
  //   assert!(mqtt.is_valid_address("example.com"));
  //   assert!(!mqtt.is_valid_address("invalid_address"));
  // }
}
