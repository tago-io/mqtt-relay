mod schema;
mod services;
mod utils;

use schema::BridgeConfig;
use services::bridge::run_bridge;
use services::tagoio::fetch_customer_settings;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;
use tokio::time::sleep;

use crate::schema::Customer;

// TODO: How do I setup max tasks?
#[tokio::main]
async fn main() {
    // Simulate fetching customer and bridge configurations
    // ? How are we going to receive updates? Pulling the Database ? or Redis ? or a hook ?
    let customers = fetch_customer_settings().await;
    let customers = Arc::new(Mutex::new(customers));

    let (tx, mut rx) = mpsc::channel(32);

    // Spawn a task to simulate receiving updates
    let tx_clone = tx.clone();
    tokio::task::spawn(async move {
        simulate_redis_updates(tx_clone).await;
    });

    let customers_clone = Arc::clone(&customers);
    tokio::task::spawn(async move {
        while let Some(message) = rx.recv().await {
            match message {
                UpdateMessage::UpdateCustomer(updated_customer) => {
                    let mut customers = customers_clone.lock().unwrap();
                    customers.insert(updated_customer.id, updated_customer);
                }
                UpdateMessage::RemoveCustomer(customer_id) => {
                    let mut customers = customers_clone.lock().unwrap();
                    customers.remove(&customer_id);
                }
            }
        }
    });

    let mut tasks = HashMap::new();

    loop {
        let customers = customers.lock().unwrap().clone();

        for (customer_id, customer) in customers.iter() {
            for bridge in &customer.bridges {
                let bridge_id = bridge.id.clone();
                println!("{}", bridge_id);
                if !tasks.contains_key(&bridge_id) {
                    let bridge_clone = Arc::clone(bridge);
                    let task = tokio::task::spawn(async move {
                        run_bridge(bridge_clone).await;
                    });
                    tasks.insert(bridge_id.clone(), task);
                }
            }
        }

        // Wait for all tasks to complete
        tasks.retain(|_, task| !task.is_finished());

        sleep(Duration::from_secs(5)).await;
        println!("All bridge tasks completed.");
    }
}

/**
* Mock customer and bridge configurations
*/

enum UpdateMessage {
    UpdateCustomer(Customer),
    RemoveCustomer(u32),
}

async fn simulate_redis_updates(tx: mpsc::Sender<UpdateMessage>) {
    loop {
        sleep(Duration::from_secs(2)).await;

        println!("Simulating Redis updates");

        // Simulate an update
        let mut bridges = Vec::new();
        for j in 1..=2 {
            let client_id = format!("client_2_{}", j);
            let bridge = BridgeConfig {
                id: format!("2_{}", j),
                address: "mqtt.tago.io".to_string(),
                version: "3.1.1".to_string(),
                tls: false,
                client_id,
                username: Some("Token".to_string()),
                password: Some("new_password".to_string()),
                certificate: None,
            };
            bridges.push(Arc::new(bridge));
        }
        let updated_customer = Customer { id: 1, bridges };

        tx.send(UpdateMessage::UpdateCustomer(updated_customer))
            .await
            .unwrap();
    }
}
