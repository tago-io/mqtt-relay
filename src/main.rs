mod schema;
mod services;
mod utils;

use anyhow::Result;
use axum::response::IntoResponse;
use axum::routing::post;
use axum::{self, Json};
use axum::{Extension, Router};
use schema::ConfigFile;
use services::mqttrelay::{run_mqtt_relay_connection, PublishMessage};
use services::tagoio::get_relay_list;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio;
use tokio::sync::{mpsc, RwLock};
use tokio::time::sleep;

use once_cell::sync::Lazy;

static CONFIG_FILE: Lazy<Option<ConfigFile>> = Lazy::new(|| utils::fetch_config_file());

#[derive(serde::Deserialize)]
struct PublishRequest {
    topic: String,
    message: String,
    relay_id: String,
    qos: u8,
    retain: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Simulate fetching relay configurations
    let relay_list = get_relay_list().await?;
    let relay_list = Arc::new(Mutex::new(relay_list));

    let tasks = Arc::new(RwLock::new(HashMap::new()));

    // Start the HTTP server
    let app = Router::new()
        .route("/publish", post(handle_publish))
        .layer(Extension(tasks.clone()));

    let server = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", "3000"))
        .await
        .unwrap();

    tokio::spawn(async move {
        axum::serve(server, app).await.unwrap();
    });

    // Start the relay tasks
    loop {
        let relay_list = relay_list.lock().unwrap().clone();

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

        // Relay will be restarted after 60 seconds
        sleep(Duration::from_secs(60)).await;
    }
}

/**
* Handle incoming publish requests from the HTTP server
*/
async fn handle_publish(
    Extension(tasks): Extension<
        Arc<RwLock<HashMap<String, (tokio::task::JoinHandle<()>, mpsc::Sender<PublishMessage>)>>>,
    >,
    Json(payload): Json<PublishRequest>,
) -> Result<impl IntoResponse, axum::http::StatusCode> {
    let tasks = tasks.read().await;
    if let Some((_, publish_tx)) = tasks.get(&payload.relay_id) {
        let message = PublishMessage {
            topic: payload.topic,
            message: payload.message,
            qos: payload.qos,
            retain: payload.retain,
        };

        publish_tx
            .send(message)
            .await
            .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?;

        Ok(axum::http::StatusCode::OK)
    } else {
        Err(axum::http::StatusCode::NOT_FOUND)
    }
}
