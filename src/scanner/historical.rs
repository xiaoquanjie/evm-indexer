use alloy::eips::BlockId;
use alloy::network::primitives::BlockTransactionsKind;
use alloy::providers::Provider;
use alloy::rpc::types::eth::BlockNumberOrTag;
use alloy::rpc::types::{Block, TransactionReceipt};
use anyhow::{anyhow, Result};
use std::sync::Arc;

/// 获取历史区块数据
pub struct HistoricalScanner {
    provider: Arc<dyn Provider>,
}

impl HistoricalScanner {
    pub fn new(provider: Arc<dyn Provider>) -> Self {
        Self { provider }
    }

    pub fn get_provider(&self) -> Arc<dyn Provider> {
        self.provider.clone()
    }

    pub async fn get_block(&self, block_id: u64) -> Result<Option<Block>> {
        let block = self
            .provider
            .get_block_by_number(BlockNumberOrTag::Number(block_id))
            .kind(BlockTransactionsKind::Full)
            .await?;
        Ok(block)
    }

    pub async fn get_receipts(&self, block_id: u64) -> Result<Vec<TransactionReceipt>> {
        let data = self
            .provider
            .get_block_receipts(BlockId::Number(BlockNumberOrTag::Number(block_id)))
            .await?;
        data.ok_or(anyhow!("error: get block receipts"))
    }

    pub async fn get_block_number(&self) -> Result<u64> {
        let n = self.provider.get_block_number().await?;
        Ok(n as u64)
    }
}
