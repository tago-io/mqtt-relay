use crate::schema::BridgeConfig;
use rumqttc::{AsyncClient, MqttOptions, QoS, TlsConfiguration};
use std::sync::Arc;
use tokio::time::{sleep, Duration};

pub async fn run_bridge(bridge: Arc<BridgeConfig>) {
    let mut mqttoptions = MqttOptions::new(&bridge.client_id, &bridge.address, 1883);
    mqttoptions.set_keep_alive(Duration::from_secs(30));
    mqttoptions.set_max_packet_size(1024 * 1024, 1024 * 1024); // 1mb in/out

    if bridge.tls || bridge.address.starts_with("ssl") {
        if let Some(certificate) = &bridge.certificate {
            let certificate = std::fs::read_to_string(certificate).unwrap();
            mqttoptions.set_transport(rumqttc::Transport::tls_with_config(
                TlsConfiguration::Simple {
                    ca: certificate.into_bytes(),
                    alpn: None, // or Some(vec!["protocol".to_string()]) if you need ALPN
                    client_auth: None, // or Some((client_cert, client_key)) if you need client authentication
                },
            ));
        }
    }

    // TODO: This with the TLS logic and certificate seems weird af
    if let Some(username) = &bridge.username {
        if !bridge.certificate.is_none() {
            mqttoptions.set_credentials(username, bridge.password.as_deref().unwrap_or_default());
        }
    }

    // TODO: Review the CAP later
    let (client, mut eventloop) = AsyncClient::new(mqttoptions, 15);

    client
        .subscribe("tago/my_topic", QoS::AtMostOnce)
        .await
        .unwrap();

    tokio::task::spawn(async move {
        client
            .publish("hello/runmqtt", QoS::AtMostOnce, false, "Hello World")
            .await
            .unwrap();
    });

    // Attempt to connect to the MQTT broker
    match eventloop.poll().await {
        Ok(notification) => {
            println!("Received = {:?}", notification);
            if let rumqttc::Event::Incoming(rumqttc::Packet::ConnAck(_)) = notification {
                println!(
                    "Connection to MQTT broker was successful for client ID: {}",
                    bridge.client_id
                );
            }
        }
        // ! seems to be unreachable actually
        Err(e) => {
            println!(
                "Failed to connect to MQTT broker for client ID: {}. Error: {:?}",
                bridge.client_id, e
            );
            return;
        }
    }

    // Continue processing other notifications
    while let Ok(notification) = eventloop.poll().await {
        match notification {
            rumqttc::Event::Incoming(rumqttc::Packet::Publish(publish)) => {
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
    sleep(Duration::from_secs(1)).await;

    // !Reach here if timeout on connection (no control over how much time it waits for response)
    // !Reach here if connection is closed
    println!("Bridge task completed for client ID: {}", bridge.client_id);
}
