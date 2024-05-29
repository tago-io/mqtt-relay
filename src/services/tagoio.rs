use std::sync::Arc;

use anyhow::Error;
use axum::http::{HeaderMap, HeaderValue};
use reqwest::header::AUTHORIZATION;
use rumqttc::Publish;
use serde_json;

use crate::{schema::RelayConfig, CONFIG_FILE};

pub async fn get_relay_list() -> Result<Vec<Arc<RelayConfig>>, Error> {
    if let Some(config) = &*CONFIG_FILE {
        println!("Config file loaded successfully");
        let relay = RelayConfig::new_with_defaults(None, config.clone()).unwrap();
        let relays: Vec<Arc<RelayConfig>> = vec![Arc::new(relay)];

        return Ok(relays);
    }

    Err(anyhow::anyhow!("Config file not found or invalid"))
}

pub async fn forward_buffer_messages(
    relay_cfg: &RelayConfig,
    event: &Publish,
) -> Result<(), Box<dyn std::error::Error>> {
    let endpoint = "https://api.tago.io/network/publish";

    let client = reqwest::Client::new();
    let mut headers = HeaderMap::new();
    headers.insert(
        AUTHORIZATION,
        HeaderValue::from_str(&relay_cfg.config.authorization_token)?,
    );
    headers.insert(
        "network-token",
        HeaderValue::from_str(&relay_cfg.config.network_token)?,
    );

    let body = serde_json::json!({
        "topic": event.topic,
        "message": String::from_utf8(event.payload.to_vec())?,
    });

    let resp = client
        .post(endpoint)
        .headers(headers)
        .json(&body)
        .send()
        .await?
        .text()
        .await?;

    println!("{:#?}", resp);
    Ok(())
}

pub async fn verify_network_token(relay_cfg: &RelayConfig) -> Result<(), Error> {
    let endpoint = "https://api.tago.io/info";

    let client = reqwest::Client::new();
    let mut headers = HeaderMap::new();

    headers.insert(
        AUTHORIZATION,
        HeaderValue::from_str(&relay_cfg.config.network_token)?,
    );

    let resp = client.get(endpoint).headers(headers).send().await?;

    if resp.status().is_success() {
        Ok(())
    } else {
        Err(anyhow::anyhow!("Invalid token"))?
    }
}
// pub async fn fetch_customer_settings() -> Vec<Arc<RelayConfig>> {
//     let mut bridges = Vec::new();
//     for j in 1..=2 {
//         let client_id = format!("client_1_{}", j);
//         let bridge = RelayConfig {
//             id: format!("{}", j),
//             address: "localhost".to_string(),
//             version: "3.1.1".to_string(),
//             tls: false,
//             client_id,
//             username: Some("Token".to_string()),
//             password: Some("3a162597-8724-46c0-864b-1ac220a77123".to_string()),
//             certificate: None,
//             customer_id: "1".to_string(),
//         };
//         bridges.push(Arc::new(bridge));
//     }
//     bridges
//     // Implement fetching customer settings from TagoIO
// }
