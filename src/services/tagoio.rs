use std::{collections::HashMap, sync::Arc, time::Duration};

use tokio::{sync::mpsc, time::sleep};

use crate::schema::{BridgeConfig, Customer};

/**
 * Mock customer and bridge configurations
 */
pub async fn fetch_customer_settings() -> HashMap<u32, Customer> {
    let mut customers: HashMap<u32, Customer> = HashMap::new();

    for i in 1..=1 {
        let mut bridges = Vec::new();
        for j in 1..=2 {
            let client_id = format!("client_{}_{}", i, j);
            let bridge = BridgeConfig {
                id: format!("{}_{}", i, j),
                address: "localhost".to_string(),
                version: "3.1.1".to_string(),
                tls: false,
                client_id,
                username: Some("Token".to_string()),
                password: Some("3a162597-8724-46c0-864b-1ac220a77123".to_string()),
                certificate: None,
            };
            bridges.push(Arc::new(bridge));
        }
        customers.insert(i, Customer { id: i, bridges });
    }
    customers
    // Implement fetching customer settings from TagoIO
}

pub async fn fetch_bridge_settings() {
    // Implement fetching bridge settings from TagoIO
}
