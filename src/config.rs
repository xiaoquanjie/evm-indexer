use anyhow::Result;
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct AppConfig {
    pub rpc: RpcConfig,
    pub database: DatabaseConfig,
    pub indexer: IndexerConfig,
}

#[derive(Debug, Deserialize, Clone)]
pub struct RpcConfig {
    /// HTTP RPC endpoint for historical block scanning
    pub http_url: String,
    /// WebSocket endpoint for live block subscription
    #[allow(unused)]
    pub ws_url: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct DatabaseConfig {
    pub url: String,
    #[serde(default = "default_max_connections")]
    pub max_connections: u32,
}

#[derive(Debug, Deserialize, Clone)]
pub struct IndexerConfig {
    /// Block number to start scanning from
    pub start_block: u64,
    /// How many blocks to fetch concurrently during historical scan
    #[serde(default = "default_concurrent_blocks")]
    #[allow(unused)]
    pub concurrent_blocks: usize,
    /// Number of confirmations before a block is considered final
    #[serde(default = "default_confirmations")]
    #[allow(unused)]
    pub confirmations: u64,
}

impl AppConfig {
    pub fn load() -> Result<Self> {
        let cfg = config::Config::builder()
            .add_source(config::File::with_name("config").required(false))
            .add_source(config::Environment::with_prefix("EVM_INDEXER").separator("__"))
            .build()?;

        Ok(cfg.try_deserialize()?)
    }
}

fn default_max_connections() -> u32 {
    10
}

fn default_concurrent_blocks() -> usize {
    10
}

fn default_confirmations() -> u64 {
    12
}
