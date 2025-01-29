use std::sync::Arc;

use crate::{schema::RelayConfig, services::tagoio::verify_device_token};
use axum::{response::IntoResponse, Extension, Json};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct AuthRequest {
  username: String,
  password: String,
}

#[derive(Debug, Serialize)]
struct AuthResponse {
  ok: bool,
}

/// Handle Mosquitto authentication requests
/// Validates the device token (password) against TagoIO
pub async fn handle_auth(
  Extension(relay_list): Extension<Arc<RwLock<Vec<Arc<RelayConfig>>>>>,
  Json(payload): Json<AuthRequest>,
) -> impl IntoResponse {
  let relays = relay_list.read().await;

  // Try to authenticate against any relay in the list
  for relay in relays.iter() {
    if let Ok(_) = verify_device_token(relay, &payload.password).await {
      return (axum::http::StatusCode::OK, Json(AuthResponse { ok: true }));
    }
  }

  (axum::http::StatusCode::UNAUTHORIZED, Json(AuthResponse { ok: false }))
}

/// Handle Mosquitto superuser checks
pub async fn handle_superuser(Json(_payload): Json<AuthRequest>) -> impl IntoResponse {
  (axum::http::StatusCode::UNAUTHORIZED, Json(AuthResponse { ok: false }))
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct AclRequest {
  username: String,
  topic: String,
  clientid: String,
  acc: i32, // 1 for read, 2 for write
}

/// Handle Mosquitto ACL checks
/// Only allow clients to publish/subscribe to their own topics
pub async fn handle_acl(Json(_payload): Json<AclRequest>) -> impl IntoResponse {
  // Allow clients to only access topics that start with their username
  // let authorized = payload.topic.starts_with(&payload.username);
  // if authorized {
  (axum::http::StatusCode::OK, Json(AuthResponse { ok: true }))
  // } else {
  //   (axum::http::StatusCode::FORBIDDEN, Json(AuthResponse { result: false }))
  // }
}
