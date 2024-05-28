use crate::{schema::BridgeConfig, utils::calculate_backoff};
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

pub async fn run_bridge(bridge: Arc<BridgeConfig>, publish_rx: mpsc::Receiver<PublishMessage>) {
    println!("Running bridge task for client ID: {}", bridge.id);
    let publish_rx = Arc::new(Mutex::new(publish_rx));
    let client_id = &bridge
        .authentication
        .client_id
        .clone()
        .unwrap_or("tagoio-relay".to_string());

    let mut mqttoptions = MqttOptions::new(client_id, &bridge.address, bridge.port);
    mqttoptions.set_keep_alive(Duration::from_secs(30));
    mqttoptions.set_max_packet_size(1024 * 1024, 1024 * 1024); // 1mb in/out

    if bridge.tls || bridge.address.starts_with("ssl") {
        if let Some(certificate) = &bridge.certificate {
            mqttoptions.set_transport(rumqttc::Transport::tls_with_config(
                TlsConfiguration::Simple {
                    ca: certificate.clone().into_bytes(),
                    alpn: None, // or Some(vec!["protocol".to_string()]) if you need ALPN
                    client_auth: None, // or Some((client_cert, client_key)) if you need client authentication
                },
            ));
        }
    }

    // TODO: Review this logic for picking the right credentials (tls/certificate VS tls/username+password VS no-tls/username+password)
    if !bridge.authentication.username.is_empty() {
        if !bridge.certificate.is_none() {
            mqttoptions.set_credentials(
                &bridge.authentication.username,
                &bridge.authentication.password,
            );
        }
    }

    let mut backoff_retry_attempts = 0;
    let backoff_max_retries = 2;

    loop {
        // TODO: Review the CAP later
        let (client, mut eventloop) = AsyncClient::new(mqttoptions.clone(), 15);

        for topic in bridge.subscribe.iter() {
            client.subscribe(topic, QoS::AtMostOnce).await.unwrap();
        }

        // Attempt to connect to the MQTT broker
        match eventloop.poll().await {
            Ok(notification) => {
                println!("Received = {:?}", notification);
                if let rumqttc::Event::Incoming(rumqttc::Packet::ConnAck(_)) = notification {
                    println!(
                        "Connection to MQTT broker was successful for: {}",
                        bridge.id
                    );

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
                    "Failed to connect to MQTT broker for client ID: {}. Error: {:?}",
                    bridge.id, e
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
                        crate::services::tagoio::forward_buffer_messages(&bridge, &publish).await
                    {
                        println!(
                            "Failed to forward message to TagoIO for bridge ID: {}. Error: {:?}",
                            bridge.id, e
                        );
                    }

                    println!(
                        "Received message on topic {}: {:?}",
                        publish.topic, publish.payload
                    );
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
            println!(">>> Max retries reached. Exiting: {}", bridge.id);
            // TODO: Report to TagoIO that the bridge is down
            return;
        }
        let backoff_duration = calculate_backoff(backoff_retry_attempts);
        println!("Retrying in {:?}", backoff_duration);
        sleep(backoff_duration).await;
        backoff_retry_attempts += 1;
    }
}
