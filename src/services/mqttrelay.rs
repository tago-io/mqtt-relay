use crate::{schema::RelayConfig, utils::calculate_backoff};
use rumqttc::{AsyncClient, MqttOptions, QoS, TlsConfiguration};
use serde::Deserialize;
use std::sync::Arc;
use tokio::{
    sync::{mpsc, Mutex},
    time::{sleep, Duration},
};

#[derive(Deserialize)]
pub struct PublishMessage {
    pub topic: String,
    pub message: String,
    pub qos: u8,
    pub retain: bool,
}

pub async fn run_mqtt_relay_connection(
    relay_cfg: Arc<RelayConfig>,
    publish_rx: mpsc::Receiver<PublishMessage>,
) {
    println!("Running bridge task for client ID: {}", relay_cfg.id);

    let publish_rx = Arc::new(Mutex::new(publish_rx));

    // Desconstruct the relay configuration
    let client_id = &relay_cfg
        .config
        .mqtt
        .client_id
        .clone()
        .unwrap_or("tagoio-relay".to_string());

    let username = &relay_cfg.config.mqtt.username;
    let password = &relay_cfg.config.mqtt.password;
    let certificate_file = &relay_cfg.config.mqtt.authentication_certificate_file;

    // Start the MQTT client
    let mut mqttoptions = MqttOptions::new(
        client_id,
        &relay_cfg.config.mqtt.address,
        relay_cfg.config.mqtt.port,
    );
    mqttoptions.set_keep_alive(Duration::from_secs(30));
    mqttoptions.set_max_packet_size(1024 * 1024, 1024 * 1024); // 1mb in/out

    if relay_cfg.config.mqtt.tls_enabled || relay_cfg.config.mqtt.address.starts_with("ssl") {
        if let Some(certificate) = &certificate_file {
            mqttoptions.set_transport(rumqttc::Transport::tls_with_config(
                TlsConfiguration::Simple {
                    ca: certificate.clone().into_bytes(),
                    alpn: None, // or Some(vec!["protocol".to_string()]) if you need ALPN
                    client_auth: None, // or Some((client_cert, client_key)) if you need client authentication
                },
            ));
        }
    }

    if username.is_some() {
        if !certificate_file.is_none() {
            mqttoptions.set_credentials(
                username.as_ref().unwrap(),
                password
                    .as_ref()
                    .expect("Password must be provided if username is set"),
            );
        }
    }

    let mut backoff_retry_attempts = 0;
    let backoff_max_retries = 15;

    loop {
        // TODO: Review the CAP later
        let (client, mut eventloop) = AsyncClient::new(mqttoptions.clone(), 15);

        for topic in relay_cfg.config.mqtt.subscribe.iter() {
            client.subscribe(topic, QoS::AtMostOnce).await.unwrap();
        }

        // Attempt to connect to the MQTT broker
        match eventloop.poll().await {
            Ok(notification) => {
                println!("Received = {:?}", notification);
                if let rumqttc::Event::Incoming(rumqttc::Packet::ConnAck(_)) = notification {
                    println!("Connection to MQTT broker was successful");

                    // Reset backoff attempts
                    backoff_retry_attempts = 0;
                }
            }
            Err(e) => {
                // TODO: Critical error handling such as TLS errors should be handled differently
                // if let rumqttc::ConnectionError::Tls(_) = e {
                //     println!(
                //         "Uncategorized error encountered for client ID: {}. Dropping error: {:?}",
                //         bridge.client_id, e
                //     );
                //     // TODO: Report to TagoIO that the bridge is down
                //     return;
                // }

                println!(
                    "Action Required: Failed to connect to MQTT broker. Error details: {:?}",
                    e.to_string()
                );
            }
        }

        let publish_rx_clone = Arc::clone(&publish_rx);
        tokio::spawn(async move {
            while let Some(publish_message) = publish_rx_clone.lock().await.recv().await {
                client
                    .publish(
                        &publish_message.topic,
                        QoS::AtLeastOnce,
                        publish_message.retain,
                        publish_message.message.as_bytes(),
                    )
                    .await
                    .unwrap();
            }
        });

        // Continue processing other notifications
        while let Ok(notification) = eventloop.poll().await {
            match notification {
                rumqttc::Event::Incoming(rumqttc::Packet::Publish(publish)) => {
                    if let Err(e) =
                        crate::services::tagoio::forward_buffer_messages(&relay_cfg, &publish).await
                    {
                        println!("Failed to forward message to TagoIO: {:?}", e.to_string());
                    }

                    // println!(
                    //     "Received message on topic {}: {:?}",
                    //     publish.topic, publish.payload
                    // );
                }
                _ => {
                    println!("Received = {:?}", notification);
                }
            }
        }
        // Simulate running task
        sleep(Duration::from_secs(2)).await;

        // !Reach here if connection is closed
        // !Reach here if timeout on connection (no control over how much time it waits for response)

        // Exponential Backoff

        if backoff_retry_attempts >= backoff_max_retries {
            eprintln!("Warning: Max retries reached. Exiting: {}", relay_cfg.id);
            return;
        }
        let backoff_duration = calculate_backoff(backoff_retry_attempts);
        println!("Retrying in {:?}", backoff_duration);
        sleep(backoff_duration).await;
        backoff_retry_attempts += 1;
    }
}
