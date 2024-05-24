#[derive(Debug)]
pub struct BridgeConfig {
    pub id: String,
    pub address: String,
    pub version: String,
    pub client_id: String,
    pub tls: bool,
    pub username: Option<String>,
    pub password: Option<String>,
    pub certificate: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Customer {
    pub id: u32,
    pub bridges: Vec<std::sync::Arc<BridgeConfig>>,
}
