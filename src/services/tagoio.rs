use std::sync::Arc;

use anyhow::Error;
use axum::http::{HeaderMap, HeaderValue};
use reqwest::StatusCode;
use rumqttc::{Publish, QoS};
use serde_json;
use std::fmt;

use crate::{schema::RelayConfig, CONFIG_FILE};

/**
 * Get the list of relay configurations
 * To be improved later to support multiple relays
 */
pub async fn get_relay_list() -> Result<Vec<Arc<RelayConfig>>, Error> {
    if let Some(config) = &*CONFIG_FILE {
        println!("Config file loaded successfully");
        let relay = RelayConfig::new_with_defaults(None, config.clone()).unwrap();
        let relays: Vec<Arc<RelayConfig>> = vec![Arc::new(relay)];

        return Ok(relays);
    }

    Err(anyhow::anyhow!("Config file not found or invalid"))
}

#[derive(Debug)]
pub struct CustomError {
    pub status: StatusCode,
    pub body: String,
    pub message: String,
}

impl fmt::Display for CustomError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Status: {}, Body: {}, Message: {}",
            self.status, self.body, self.message
        )
    }
}

impl std::error::Error for CustomError {}

/**
 * Wrapper function to make a request to the TagoIO API
 */
async fn make_request(
    method: reqwest::Method,
    url: &str,
    headers: HeaderMap,
    body: Option<serde_json::Value>,
) -> Result<String, CustomError> {
    let client = reqwest::Client::new();
    let request = client.request(method, url).headers(headers);

    let request = if let Some(body) = body {
        request.json(&body)
    } else {
        request
    };

    let response = request.send().await.map_err(|e| CustomError {
        status: StatusCode::INTERNAL_SERVER_ERROR,
        body: String::new(),
        message: e.to_string(),
    })?;

    let status = response.status();
    let text = response.text().await.map_err(|e| CustomError {
        status,
        body: String::new(),
        message: e.to_string(),
    })?;

    if !status.is_success() {
        return Err(CustomError {
            status,
            body: text.clone(),
            message: format!("Request failed with status: {}", status),
        });
    }
    Ok(text)
}

/**
 * Forward the buffered messages to TagoIO Network
 */
pub async fn forward_buffer_messages(
    relay_cfg: &RelayConfig,
    event: &Publish,
) -> Result<(), Box<dyn std::error::Error>> {
    let endpoint = relay_cfg
        .config
        .tagoio_url
        .clone()
        .unwrap_or_else(|| "https://api.tago.io".to_string());

    let query_string = format!(
        "?authorization_token={}",
        relay_cfg.config.authorization_token
    );

    let endpoint = format!("{}/integration/network/data{}", endpoint, query_string);

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

    match make_request(reqwest::Method::POST, &endpoint, headers, Some(body)).await {
        Ok(response) => response,
        Err(e) => {
            return Err(Box::new(e));
        }
    };

    Ok(())
}

/**
 * Verify that the network token is valid
 */
pub async fn verify_network_token(relay_cfg: &RelayConfig) {
    let endpoint = relay_cfg
        .config
        .tagoio_url
        .clone()
        .unwrap_or_else(|| "https://api.tago.io".to_string());

    let endpoint = format!("{}/info", endpoint);

    let mut headers = HeaderMap::new();
    headers.insert(
        "AUTHORIZATION",
        HeaderValue::from_str(&relay_cfg.config.network_token).unwrap(),
    );

    let resp = make_request(reqwest::Method::GET, &endpoint, headers, None)
        .await
        .unwrap();

    if resp.is_empty() {
        panic!("Invalid Network Token: Check your network token and TagoIO API UR and try again");
    }
}
/**
 * Unit Test Section
 */
#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema::{ConfigFile, MQTT};
    use mockito::Matcher;
    use rumqttc::{Publish, QoS};
    use tokio;

    fn get_test_relay_config(server: &mockito::Server) -> RelayConfig {
        RelayConfig {
            id: "test_id".to_string(),
            config: ConfigFile {
                network_token: "test_network_token".to_string(),
                authorization_token: "test_authorization_token".to_string(),
                tagoio_url: Some(server.url()),
                api_port: Some("3000".to_string()),
                mqtt: MQTT {
                    client_id: Some("test_client_id".to_string()),
                    tls_enabled: false,
                    address: "localhost".to_string(),
                    port: 1883,
                    subscribe: vec!["/tago/#".to_string(), "/device/+".to_string()],
                    username: Some("test_username".to_string()),
                    password: Some("test_password".to_string()),
                    authentication_certificate_file: Some("certs/ca.crt".to_string()),
                },
            },
            profile_id: None,
            state: None,
        }
    }

    #[tokio::test]
    async fn test_forward_buffer_messages() {
        let mut server = mockito::Server::new_async().await;
        let relay_cfg = get_test_relay_config(&server);
        let event = Publish::new(
            "test/topic",
            QoS::AtLeastOnce,
            vec![104, 101, 108, 108, 111], // "hello" in bytes
        );

        let _m = server
            .mock("POST", "/integration/network/data")
            .match_query(Matcher::UrlEncoded(
                "authorization_token".into(),
                "test_authorization_token".into(),
            ))
            .match_header("AUTHORIZATION", "test_network_token")
            .match_body(Matcher::Json(serde_json::json!([{
                "variable": "payload",
                "value": "hello",
                "metadata": {
                    "topic": "test/topic",
                    "qos": 1,
                }
            }])))
            .with_status(200)
            .with_body("Success")
            .create_async()
            .await;

        let result = forward_buffer_messages(&relay_cfg, &event).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_verify_network_token() {
        let mut server = mockito::Server::new_async().await;
        let relay_cfg = get_test_relay_config(&server);
        let _m = server
            .mock("GET", "/info")
            .match_header("AUTHORIZATION", "test_network_token")
            .with_status(200)
            .create_async()
            .await;

        verify_network_token(&relay_cfg).await;
    }

    #[tokio::test]
    #[should_panic(expected = "Invalid Network Token")]
    async fn test_verify_network_token_invalid() {
        let mut server = mockito::Server::new_async().await;
        let relay_cfg = get_test_relay_config(&server);

        let _m = server
            .mock("GET", "/info")
            .match_header("AUTHORIZATION", "test_network_token")
            .with_status(401)
            .create_async()
            .await;

        verify_network_token(&relay_cfg).await;
    }
}
