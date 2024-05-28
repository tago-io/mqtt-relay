use std::sync::Arc;

use axum::http::{HeaderMap, HeaderValue};
use reqwest::header::AUTHORIZATION;
use rumqttc::Publish;
use serde_json;

use crate::{schema::BridgeConfig, utils::Authentication, CONFIG_FILE};

pub async fn fetch_bridges() -> Vec<Arc<BridgeConfig>> {
    if let Some(config) = &*CONFIG_FILE {
        println!("Config file loaded successfully");
        let bridges: Vec<Arc<BridgeConfig>> = vec![Arc::new(BridgeConfig {
            id: "1".to_string(),
            address: config.config.address.clone(),
            version: "3.1.1".to_string(),
            tls: config.config.tls_enabled,
            port: config.config.port,
            certificate: None,
            profile_id: Some("1".to_string()),
            state: None,
            subscribe: config.config.subscribe.clone(),
            network_token: config.tagoio.network_token.clone(),
            authorization_token: config.tagoio.authorization.clone(),
            authentication: Authentication {
                client_id: config.config.authentication.client_id.clone(),
                username: config.config.authentication.username.clone(),
                password: config.config.authentication.password.clone(),
            },
        })];

        return bridges;
    }

    // Simulate fetching bridge configurations
    let mut bridges = Vec::new();
    for i in 1..=2 {
        let client_id = Some(format!("client_1_{}", i));
        let bridge = BridgeConfig {
            id: format!("1_{}", i),
            address: "wss://test.com".to_string(),
            version: "3.1.1".to_string(),
            tls: false,
            port: 1883,
            certificate: None,
            profile_id: Some("1".to_string()),
            state: None,
            subscribe: vec!["topic1".to_string(), "topic2".to_string()],
            network_token: "3a162597-8724-46c0-864b-1ac220a77123".to_string(),
            authorization_token: "3a162597-8724-46c0-864b-1ac220a77123".to_string(),
            authentication: Authentication {
                client_id,
                username: "Token2".to_string(),
                password: "3a162597-8724-46c0-864b-1ac220a77123".to_string(),
            },
        };
        bridges.push(Arc::new(bridge));
    }
    bridges
}

pub async fn forward_buffer_messages(
    bridge: &BridgeConfig,
    event: &Publish,
) -> Result<(), Box<dyn std::error::Error>> {
    let endpoint = "https://api.tago.io/network/publish";

    let client = reqwest::Client::new();
    let mut headers = HeaderMap::new();
    headers.insert(
        AUTHORIZATION,
        HeaderValue::from_str(&bridge.authorization_token)?,
    );
    headers.insert(
        "network-token",
        HeaderValue::from_str(&bridge.network_token)?,
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

/**
 * Mock customer and bridge configurations
 */
// pub async fn fetch_customer_settings() -> Vec<Arc<BridgeConfig>> {
//     let mut bridges = Vec::new();
//     for j in 1..=2 {
//         let client_id = format!("client_1_{}", j);
//         let bridge = BridgeConfig {
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

pub async fn fetch_bridge_settings() {
    // Implement fetching bridge settings from TagoIO
}
