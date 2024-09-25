use crate::{
  services::{
    mqttrelay::{run_mqtt_relay_connection, PublishMessage},
    tagoio::{get_relay_list, verify_network_token},
  },
  CONFIG_FILE,
};
use anyhow::Result;
use axum::{
  extract::rejection::JsonRejection,
  http::StatusCode,
  response::{IntoResponse, Response},
  routing::post,
  Extension, Json, Router,
};

use axum_server::tls_openssl::OpenSSLConfig;
use openssl::{
  pkey::PKey,
  ssl::{SslAcceptor, SslMethod, SslVerifyMode},
  x509::{store::X509StoreBuilder, X509},
};

use dotenvy_macro::dotenv;
use serde_json::json;
use std::{collections::HashMap, error::Error, net::SocketAddr, sync::Arc, time::Duration};
use tokio::{
  sync::{mpsc, RwLock},
  time::sleep,
};

/**
 * Global constants
 */
const RESTART_DELAY_SECS: u64 = 120;

#[cfg(debug_assertions)]
const HOST_ADDRESS: &str = "127.0.0.1";

#[cfg(not(debug_assertions))]
const HOST_ADDRESS: &str = "::"; // ? External IPv4/IPv6 support

fn create_ssl_acceptor(unsafe_mode: bool) -> Result<Arc<SslAcceptor>, openssl::error::ErrorStack> {
  // Certificates contents are stored in the environment variables
  let cert = dotenv!("CARGO_SERVER_SSL_CERT").as_bytes();
  let key = dotenv!("CARGO_SERVER_SSL_KEY").as_bytes();
  let ca = dotenv!("CARGO_SERVER_SSL_CA").as_bytes();

  let cert = X509::from_pem(cert)?;
  let key = PKey::private_key_from_pem(key)?;
  let ca = X509::from_pem(ca)?;

  let mut acceptor = SslAcceptor::mozilla_intermediate(SslMethod::tls())?;
  acceptor.set_private_key(&key)?;
  acceptor.set_certificate(&cert)?;
  acceptor.check_private_key()?;

  if !unsafe_mode {
    // Create a new X509Store and add the CA certificate to it
    let mut store_builder = X509StoreBuilder::new()?;
    store_builder.add_cert(ca.clone())?;
    let store = store_builder.build();

    // Set the CA store for the acceptor
    acceptor.set_cert_store(store);

    // Add the CA certificate as a client CA
    acceptor.add_client_ca(&ca)?;

    acceptor.set_verify(SslVerifyMode::PEER | SslVerifyMode::FAIL_IF_NO_PEER_CERT);
  } else {
    log::warn!(target: "security", "Running in unsafe mode: SSL Certificates verification disabled");
    acceptor.set_verify(SslVerifyMode::NONE);
  }
  Ok(Arc::new(acceptor.build()))
}

/**
 * Start the MQTT Relay service
 */
pub async fn start_relay(unsafe_mode: bool) -> Result<()> {
  // Simulate fetching relay configurations
  let relay_list = get_relay_list().await?;
  let relay_list = Arc::new(RwLock::new(relay_list));

  for relay in relay_list.read().await.iter() {
    log::info!(target: "network", "Verifying network token for relay: {}", relay.id);
    if let Err(e) = verify_network_token(relay).await {
      log::error!(target: "network", "Failed to verify network token for relay {}: {}", relay.id, e);
      std::process::exit(1);
    }
  }

  let tasks = Arc::new(RwLock::new(HashMap::new()));

  // Start the HTTP server
  let app = Router::new()
    .route("/publish", post(handle_publish))
    .layer(Extension(tasks.clone()));

  let api_port = {
    let config_file = CONFIG_FILE.read().unwrap();
    config_file.as_ref().unwrap().downlink_port.unwrap_or(3000)
  };

  let test = create_ssl_acceptor(unsafe_mode).unwrap();
  let acceptor = OpenSSLConfig::from_acceptor(test);

  // let listener = match tokio::net::TcpListener::bind(format!("{}:{}", HOST_ADDRESS, api_port)).await {
  //   Ok(listener) => listener,
  //   Err(e) => {
  //     log::error!(target: "error", "Failed to bind to port {}: {}", api_port, e);
  //     std::process::exit(1);
  //   }
  // };

  let addr = SocketAddr::from((HOST_ADDRESS.parse::<std::net::IpAddr>().unwrap(), api_port));

  tokio::spawn(async move {
    log::info!(target: "info", "Starting Publish API on: {}", addr);
    axum_server::bind_openssl(addr, acceptor)
      .serve(app.into_make_service())
      .await
      .unwrap();
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
        tasks.write().await.insert(relay_id.clone(), (task, publish_tx));
      }
    }

    tasks.write().await.retain(|_, (task, _)| !task.is_finished());

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

type TaskMap = HashMap<String, (tokio::task::JoinHandle<()>, mpsc::Sender<PublishMessage>)>;
type SharedTaskMap = Arc<RwLock<TaskMap>>;

async fn handle_publish(
  Extension(tasks): Extension<SharedTaskMap>,
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
        JsonRejection::JsonSyntaxError(_) => (StatusCode::BAD_REQUEST, "Syntax error in JSON".to_string()),
        JsonRejection::MissingJsonContentType(_) => (
          StatusCode::BAD_REQUEST,
          "Missing `Content-Type: application/json` header".to_string(),
        ),
        _ => (StatusCode::INTERNAL_SERVER_ERROR, "Unknown error".to_string()),
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

    Ok((StatusCode::OK, Json(json!({ "status": "Message published" }))))
  } else {
    Err(JsonError(axum::http::StatusCode::NOT_FOUND))
  }
}
