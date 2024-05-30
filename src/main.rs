mod schema;
mod services;
mod utils;

use anyhow::Result;
use axum::extract::rejection::JsonRejection;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::post;
use axum::{self, Json};
use axum::{Extension, Router};
use schema::ConfigFile;
use serde_json::json;
use services::mqttrelay::{run_mqtt_relay_connection, PublishMessage};
use services::tagoio::{get_relay_list, verify_network_token};
use std::collections::HashMap;
use std::error::Error;
use std::sync::Arc;
use std::time::Duration;
use tokio;
use tokio::sync::{mpsc, RwLock};
use tokio::time::sleep;

use once_cell::sync::Lazy;

const RESTART_DELAY_SECS: u64 = 120;
static CONFIG_FILE: Lazy<Option<ConfigFile>> = Lazy::new(|| utils::fetch_config_file());

#[tokio::main]
async fn main() -> Result<()> {
    // Simulate fetching relay configurations
    let relay_list = get_relay_list().await?;
    let relay_list = Arc::new(RwLock::new(relay_list));

    for relay in relay_list.read().await.iter() {
        verify_network_token(relay).await;
    }

    let tasks = Arc::new(RwLock::new(HashMap::new()));

    // Start the HTTP server
    let app = Router::new()
        .route("/publish", post(handle_publish))
        .layer(Extension(tasks.clone()));

    let api_port = CONFIG_FILE
        .as_ref()
        .unwrap()
        .api_port
        .clone()
        .unwrap_or("3000".to_string());

    println!("Starting API on port {}", api_port);
    let server = match tokio::net::TcpListener::bind(format!("0.0.0.0:{}", api_port)).await {
        Ok(listener) => listener,
        Err(e) => {
            eprintln!("Failed to bind to port {}: {}", api_port, e);
            return Err(anyhow::anyhow!("Failed to bind to port"));
        }
    };

    tokio::spawn(async move {
        println!("Listening on: {}", server.local_addr().unwrap());
        axum::serve(server, app).await.unwrap();
    });

    // Start the relay tasks
    loop {
        let relay_list = relay_list.read().await.clone();

        for relay in &relay_list {
            let relay_id = relay.id.clone();
            if !tasks.read().await.contains_key(&relay_id) {
                let relay_clone = Arc::clone(relay);
                let (publish_tx, publish_rx) = mpsc::channel(32);
                let task = tokio::task::spawn(async move {
                    run_mqtt_relay_connection(relay_clone, publish_rx).await;
                });
                tasks
                    .write()
                    .await
                    .insert(relay_id.clone(), (task, publish_tx));
            }
        }

        tasks
            .write()
            .await
            .retain(|_, (task, _)| !task.is_finished());

        // Relay will be restarted after 120 seconds
        sleep(Duration::from_secs(RESTART_DELAY_SECS)).await;
    }
}

#[derive(serde::Deserialize)]
struct PublishRequest {
    topic: String,
    message: String,
    relay_id: Option<String>,
    qos: u8,
    retain: bool,
}

/**
* Handle incoming publish requests from the HTTP server
*/
struct JsonError(axum::http::StatusCode);

impl IntoResponse for JsonError {
    fn into_response(self) -> Response {
        let body = Json(json!({
            "error": self.0.canonical_reason().unwrap_or("Unknown error")
        }));
        (self.0, body).into_response()
    }
}

async fn handle_publish(
    Extension(tasks): Extension<
        Arc<RwLock<HashMap<String, (tokio::task::JoinHandle<()>, mpsc::Sender<PublishMessage>)>>>,
    >,
    payload: Result<Json<PublishRequest>, JsonRejection>,
) -> Result<impl IntoResponse, JsonError> {
    let payload = match payload {
        Ok(payload) => payload,
        Err(rejection) => {
            let (status, error_message) = match rejection {
                JsonRejection::JsonDataError(err) => {
                    let detailed_error = format!("Invalid JSON data: {}", err.source().unwrap());
                    (StatusCode::UNPROCESSABLE_ENTITY, detailed_error)
                }
                JsonRejection::JsonSyntaxError(_) => {
                    (StatusCode::BAD_REQUEST, "Syntax error in JSON".to_string())
                }
                JsonRejection::MissingJsonContentType(_) => (
                    StatusCode::BAD_REQUEST,
                    "Missing `Content-Type: application/json` header".to_string(),
                ),
                _ => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Unknown error".to_string(),
                ),
            };
            return Ok((status, Json(json!({ "error": error_message }))));
        }
    };

    let tasks = tasks.read().await;
    let relay_id = if payload.relay_id.is_none() {
        if let Some(first_relay_id) = tasks.keys().next() {
            first_relay_id.clone()
        } else {
            return Err(JsonError(axum::http::StatusCode::NOT_FOUND));
        }
    } else {
        payload.relay_id.clone().unwrap()
    };

    if let Some((_, publish_tx)) = tasks.get(&relay_id) {
        let message = PublishMessage {
            topic: payload.topic.clone(),
            message: payload.message.clone(),
            qos: payload.qos,
            retain: payload.retain,
        };

        publish_tx
            .send(message)
            .await
            .map_err(|_| JsonError(axum::http::StatusCode::INTERNAL_SERVER_ERROR))?;

        Ok((
            StatusCode::OK,
            Json(json!({ "status": "Message published" })),
        ))
    } else {
        Err(JsonError(axum::http::StatusCode::NOT_FOUND))
    }
}
