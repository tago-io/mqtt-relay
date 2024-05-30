use std::sync::Arc;

use anyhow::Error;
use axum::http::{HeaderMap, HeaderValue};
use reqwest::header::AUTHORIZATION;
use rumqttc::{Publish, QoS};
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
    let endpoint = &relay_cfg
        .config
        .tagoio_url
        .clone()
        .unwrap_or("https://api.tago.io".to_string());

    let query_string = format!(
        "?authorization_token={}",
        relay_cfg.config.authorization_token
    );

    let endpoint = format!("{}/integrations/network/data{}", endpoint, query_string);

    let client = reqwest::Client::new();
    let mut headers = HeaderMap::new();
    headers.insert(
        "AUTHORIZATION",
        HeaderValue::from_str(&relay_cfg.config.network_token)?,
    );

    let payload_str = String::from_utf8(event.payload.to_vec())?;
    let qos_number = match event.qos {
        QoS::AtMostOnce => 0,
        QoS::AtLeastOnce => 1,
        QoS::ExactlyOnce => 2,
    };

    let body = serde_json::json!([{
        "variable": "payload",
        "value": payload_str,
        "metadata": {
            "topic": event.topic.clone(),
            "qos": qos_number,
        }
    }]);

    let resp = client
        .post(endpoint)
        .headers(headers)
        .json(&body)
        .send()
        .await?
        .error_for_status()?
        .text()
        .await?;

    println!("{:#?}", resp);
    Ok(())
}

pub async fn verify_network_token(relay_cfg: &RelayConfig) {
    let endpoint = &relay_cfg
        .config
        .tagoio_url
        .clone()
        .unwrap_or("https://api.tago.io".to_string());

    let endpoint = format!("{}/info", endpoint);

    let client = reqwest::Client::new();
    let mut headers = HeaderMap::new();

    headers.insert(
        AUTHORIZATION,
        HeaderValue::from_str(&relay_cfg.config.network_token).unwrap(),
    );

    let resp = client.get(endpoint).headers(headers).send().await.unwrap();

    if !resp.status().is_success() {
        panic!("Invalid Network Token");
    }
}
