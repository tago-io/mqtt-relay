use crate::{schema::RelayConfig, utils::calculate_backoff};
use rumqttc::{
  tokio_rustls::rustls::{ClientConfig, RootCertStore},
  AsyncClient, MqttOptions, QoS, TlsConfiguration,
};
use serde::Deserialize;
use std::sync::Arc;
use tokio::{
  sync::{mpsc, Mutex, Semaphore},
  time::{sleep, Duration},
};
const BACKOFF_MAX_RETRIES: u32 = 20;

#[derive(Deserialize)]
#[allow(dead_code)]
pub struct PublishMessage {
  pub topic: String,
  pub message: String,
  pub qos: u8,
  pub retain: bool,
}

pub async fn run_mqtt_relay_connection(relay_cfg: Arc<RelayConfig>, publish_rx: mpsc::Receiver<PublishMessage>) {
  log::info!(target: "mqtt", "Running relay task for client ID: {}", relay_cfg.id);

  let publish_rx = Arc::new(Mutex::new(publish_rx));

  let mqttoptions = initialize_mqtt_options(&relay_cfg);

  let mut backoff_retry_attempts = 0;

  loop {
    let (client, mut eventloop) = AsyncClient::new(mqttoptions.clone(), 15);

    subscribe_to_topics(&client, &relay_cfg).await;

    let publish_rx_clone = Arc::clone(&publish_rx);
    let publish_task = tokio::spawn(async move {
      if let Err(e) = publish_messages(client, publish_rx_clone).await {
        log::error!(target: "mqtt", "Failed to publish messages: {:?}", e);
      }
    });

    if let Err(e) = handle_mqtt_connection(&mut eventloop).await {
      log::error!(target: "error", "Failed to connect to MQTT broker. Error details: {:?}", e.to_string());
    } else {
      log::info!(target: "mqtt", "Connected to MQTT broker successfully");
      log::info!(target: "mqtt", "Subscribed to topics: {:?}", relay_cfg.config.mqtt.subscribe);
      backoff_retry_attempts = 0;
    }

    process_incoming_messages(&mut eventloop, relay_cfg.clone()).await;

    if backoff_retry_attempts >= BACKOFF_MAX_RETRIES {
      log::error!(target: "mqtt", "Max retries reached. Exiting: {}", relay_cfg.id);
      return;
    }
    let backoff_duration = calculate_backoff(backoff_retry_attempts);
    log::warn!(target: "mqtt", "Disconnected from MQTT broker. Retrying in {:?}", backoff_duration);
    publish_task.abort();
    sleep(backoff_duration).await;
    backoff_retry_attempts += 1;
  }
}

fn initialize_mqtt_options(relay_cfg: &RelayConfig) -> MqttOptions {
  let client_id = relay_cfg
    .config
    .mqtt
    .client_id
    .clone()
    .unwrap_or("tagoio-relay".to_string());

  let username = &relay_cfg.config.mqtt.username;
  let password = &relay_cfg.config.mqtt.password;
  let ca_file = &relay_cfg.config.mqtt.broker_tls_ca;
  let crt_file = &relay_cfg.config.mqtt.broker_tls_cert;
  let key_file = &relay_cfg.config.mqtt.broker_tls_key;

  let ca_content = if let Some(ca_path) = ca_file {
    std::fs::read_to_string(ca_path).unwrap_or_else(|e| {
      log::error!(target: "mqtt", "Failed to read CA file {}: {}", ca_path, e);
      String::new()
    })
  } else {
    String::new()
  };

  let crt_content = if let Some(crt_path) = crt_file {
    std::fs::read_to_string(crt_path).unwrap_or_else(|e| {
      log::error!(target: "mqtt", "Failed to read certificate file {}: {}", crt_path, e);
      String::new()
    })
  } else {
    String::new()
  };

  let key_content = if let Some(key_path) = key_file {
    std::fs::read_to_string(key_path).unwrap_or_else(|e| {
      log::error!(target: "mqtt", "Failed to read key file {}: {}", key_path, e);
      String::new()
    })
  } else {
    String::new()
  };

  // Replace the file paths with their contents
  let ca_file = if !ca_content.is_empty() { Some(ca_content) } else { None };
  let crt_file = if !crt_content.is_empty() {
    Some(crt_content)
  } else {
    None
  };
  let key_file = if !key_content.is_empty() {
    Some(key_content)
  } else {
    None
  };

  let mut mqttoptions = MqttOptions::new(client_id, &relay_cfg.config.mqtt.address, relay_cfg.config.mqtt.port);
  mqttoptions.set_keep_alive(Duration::from_secs(30));
  mqttoptions.set_max_packet_size(1024 * 1024, 1024 * 1024); // 1mb in/out

  if relay_cfg.config.mqtt.tls_enabled || relay_cfg.config.mqtt.address.starts_with("ssl") {
    if let (Some(ca_content), Some(crt_content), Some(key_content)) = (ca_file.clone(), crt_file, key_file) {
      // All three files are provided - use client certificate authentication
      mqttoptions.set_transport(rumqttc::Transport::tls_with_config(TlsConfiguration::Simple {
        ca: ca_content.into_bytes(),
        alpn: None,
        client_auth: Some((crt_content.into_bytes(), key_content.into_bytes())),
      }));
    } else if let Some(ca_content) = ca_file {
      // Only CA is provided - use server verification only
      mqttoptions.set_transport(rumqttc::Transport::tls_with_config(TlsConfiguration::Simple {
        ca: ca_content.into_bytes(),
        alpn: None,
        client_auth: None,
      }));
    } else {
      // No certificates provided - use system root certificates
      let mut root_cert_store = RootCertStore::empty();
      root_cert_store
        .add_parsable_certificates(rustls_native_certs::load_native_certs().expect("could not load platform certs"));

      let client_config = ClientConfig::builder()
        .with_root_certificates(Arc::new(root_cert_store))
        .with_no_client_auth();

      mqttoptions.set_transport(rumqttc::Transport::tls_with_config(client_config.into()));
    }
  }

  if let Some(username) = username {
    mqttoptions.set_credentials(
      username,
      password.as_ref().expect("Password must be provided if username is set"),
    );
  }

  mqttoptions
}

async fn subscribe_to_topics(client: &AsyncClient, relay_cfg: &RelayConfig) {
  for topic in relay_cfg.config.mqtt.subscribe.iter() {
    client.subscribe(topic, QoS::AtMostOnce).await.unwrap();
  }
}

async fn handle_mqtt_connection(eventloop: &mut rumqttc::EventLoop) -> Result<(), rumqttc::ConnectionError> {
  match eventloop.poll().await {
    Ok(notification) => {
      if let rumqttc::Event::Incoming(rumqttc::Packet::ConnAck(_)) = notification {
        log::info!("Connection to MQTT broker was successful");
      }
      Ok(())
    }
    Err(e) => Err(e),
  }
}

async fn publish_messages(
  client: AsyncClient,
  publish_rx: Arc<Mutex<mpsc::Receiver<PublishMessage>>>,
) -> anyhow::Result<()> {
  while let Some(publish_message) = publish_rx.lock().await.recv().await {
    log::info!(target: "mqtt", "[API] External published received on topic {}.", publish_message.topic);
    if let Err(e) = client
      .publish(
        &publish_message.topic,
        QoS::AtLeastOnce,
        publish_message.retain,
        publish_message.message.as_bytes(),
      )
      .await
    {
      log::error!(target: "mqtt", "Failed to publish message: {:?}", e);
    }
  }
  Ok(())
}

async fn process_incoming_messages(eventloop: &mut rumqttc::EventLoop, relay_cfg: Arc<RelayConfig>) {
  // Limit concurrent requests to avoid overwhelming TagoIO or running out of file descriptors
  let semaphore = Arc::new(Semaphore::new(50));

  while let Ok(notification) = eventloop.poll().await {
    if let rumqttc::Event::Incoming(rumqttc::Packet::Publish(publish)) = notification {
      log::info!(target: "mqtt", "[Broker] Received message on topic {}", publish.topic);

      let relay_cfg = relay_cfg.clone();
      let semaphore = semaphore.clone();

      tokio::spawn(async move {
        // Acquire a permit. If the semaphore is closed, we just return.
        let _permit = match semaphore.acquire().await {
          Ok(p) => p,
          Err(_) => return,
        };

        if let Err(e) = crate::services::tagoio::forward_buffer_messages(&relay_cfg, &publish).await {
          log::error!(target: "mqtt", "Failed to forward message to TagoIO: {:?}", e.to_string());
        }
      });
    }
  }
}
