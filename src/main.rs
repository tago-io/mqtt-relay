mod schema;
mod services;
mod utils;

use anyhow::Result;
use axum::response::IntoResponse;
use axum::routing::post;
use axum::{self, Json};
use axum::{Extension, Router};
use lazy_static::lazy_static;
use schema::BridgeConfig;
use services::bridge::{run_bridge, PublishMessage};
use services::tagoio::fetch_bridges;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio;
use tokio::sync::{mpsc, RwLock};
use tokio::time::sleep;
use utils::ConfigFile;

use crate::utils::Authentication;

lazy_static! {
    static ref CONFIG_FILE: Option<ConfigFile> = utils::fetch_config_file();
}

#[derive(serde::Deserialize)]
struct PublishRequest {
    topic: String,
    message: String,
    bridge_id: String,
    qos: u8,
    retain: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Simulate fetching bridge configurations
    let bridges = fetch_bridges().await;
    let bridges = Arc::new(Mutex::new(bridges));

    let (tx, mut rx) = mpsc::channel(32);

    // Spawn a task to simulate receiving updates
    // let tx_clone = tx.clone();
    // tokio::task::spawn(async move {
    //     simulate_redis_updates(tx_clone).await;
    // });

    let bridges_clone = Arc::clone(&bridges);
    tokio::task::spawn(async move {
        while let Some(message) = rx.recv().await {
            match message {
                UpdateMessage::UpdateBridge(updated_bridge) => {
                    let mut bridges = bridges_clone.lock().unwrap();
                    if let Some(pos) = bridges.iter().position(|b| b.id == updated_bridge.id) {
                        bridges[pos] = Arc::new(updated_bridge);
                    } else {
                        bridges.push(Arc::new(updated_bridge));
                    }
                }
                UpdateMessage::RemoveBridge(bridge_id) => {
                    let mut bridges = bridges_clone.lock().unwrap();
                    bridges.retain(|b| b.id != bridge_id);
                }
            }
        }
    });

    let tasks = Arc::new(RwLock::new(HashMap::new()));

    // Start the HTTP server
    let app = Router::new()
        .route("/publish", post(handle_publish))
        .layer(Extension(tasks.clone()));
    let server = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();

    tokio::spawn(async move {
        axum::serve(server, app).await.unwrap();
    });

    // Start the bridge tasks
    loop {
        let bridges = bridges.lock().unwrap().clone();

        for bridge in &bridges {
            let bridge_id = bridge.id.clone();
            if !tasks.read().await.contains_key(&bridge_id) {
                let bridge_clone = Arc::clone(bridge);
                let (publish_tx, publish_rx) = mpsc::channel(32);
                let task = tokio::task::spawn(async move {
                    run_bridge(bridge_clone, publish_rx).await;
                });
                tasks
                    .write()
                    .await
                    .insert(bridge_id.clone(), (task, publish_tx));
            }
        }

        // ! Task reinitiate since we are not removing the task from the bridge list.
        tasks
            .write()
            .await
            .retain(|_, (task, _)| !task.is_finished());

        sleep(Duration::from_secs(5)).await;
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
    if let Some((_, publish_tx)) = tasks.get(&payload.bridge_id) {
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

/**
* Mock bridge configurations
*/

enum UpdateMessage {
    UpdateBridge(BridgeConfig),
    RemoveBridge(String),
}

async fn simulate_redis_updates(tx: mpsc::Sender<UpdateMessage>) {
    // ! Rewrite with logic to fetch updates from Redis?
    loop {
        sleep(Duration::from_secs(20)).await;

        println!("Simulating Redis updates");

        // Simulate an update
        let mut bridges = Vec::new();
        for j in 1..=2 {
            let client_id = Some(format!("client_2_{}", j));
            let bridge = BridgeConfig {
                id: format!("2_{}", j),
                address: "mqtt.tago.io".to_string(),
                version: "3.1.1".to_string(),
                tls: false,
                port: 1883,
                certificate: None,
                profile_id: Some("1".to_string()),
                state: None,
                subscribe: vec!["tago/my_topic".to_string()],
                network_token: "3a162597-8724-46c0-864b-1ac220a77123".to_string(),
                authorization_token: "3a162597-8724-46c0-864b-1ac220a77123".to_string(),
                authentication: Authentication {
                    client_id,
                    username: "Token2".to_string(),
                    password: "3a162597-8724-46c0-864b-1ac220a77123".to_string(),
                },
            };
            bridges.push(bridge);
        }

        for bridge in bridges {
            tx.send(UpdateMessage::UpdateBridge(bridge)).await.unwrap();
        }
    }
}
