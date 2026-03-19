use std::sync::Arc;

pub struct Rpc {
    rpc_config: crate::config::RpcConfig,
}

impl Rpc {
    pub fn new(rpc_config: crate::config::RpcConfig) -> Self {
        Rpc { rpc_config }
    }

    #[allow(unused)]
    pub async fn get_ws_provider(&self) -> anyhow::Result<Arc<dyn alloy::providers::Provider>> {
        let provider = alloy::providers::ProviderBuilder::new()
            .connect(&self.rpc_config.ws_url)
            .await?;
        Ok(Arc::new(provider))
    }

    pub async fn get_http_provider(&self) -> anyhow::Result<Arc<dyn alloy::providers::Provider>> {
        let provider = alloy::providers::ProviderBuilder::new()
            .connect(&self.rpc_config.http_url)
            .await?;
        Ok(Arc::new(provider))
    }
}
