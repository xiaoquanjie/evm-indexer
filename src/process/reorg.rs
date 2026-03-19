use crate::process::event_decoder::{decode_transaction_log, TRANSFER_TOPIC};
use crate::{db};
use alloy::eips::BlockNumberOrTag;
use alloy::providers::Provider;
use alloy::rpc::types::eth::Block;
use anyhow::{anyhow, Result};
use std::sync::Arc;
use tracing::{debug, info};

pub struct Reorg {
    repo: Arc<db::Repository>,
    provider: Arc<dyn Provider>,
}

impl Reorg {
    pub fn new(repo: Arc<db::Repository>, provider: Arc<dyn Provider>) -> Self {
        Self { repo, provider }
    }

    /// 探测是否发生了reorg区块重组，new_block_number必须是数据库当前状态的下一个应处理区块，否则返回err
    /// 如果发生区块重组，则返回所有该重组的区块数据
    pub async fn detect_reorg(&self, new_block: &Block) -> Result<Vec<Block>> {
        let new_block_number = new_block.number();
        let new_parent_hash = format!("{:#x}", new_block.header.parent_hash);

        if let Some(sync_state) = self.repo.get_sync_state().await? {
            if sync_state.last_block + 1 != new_block_number as i64 {
                // 应该是下一个区块才对
                return Err(anyhow!("wrong new block number"));
            }
            if sync_state.last_block_hash.to_lowercase() == new_parent_hash.to_lowercase() {
                // 是下一个区块
                return Ok(vec![]);
            }
        } else {
            if new_block_number == 0 && !new_parent_hash.is_empty() {
                // 应该是初始区块才对
                return Err(anyhow!("wrong new block number or new parent hash"));
            }
            return Ok(vec![]);
        }

        if new_block_number == 0 {
            return Ok(vec![]);
        }

        // 往数据库里倒查需要重组的区块
        let mut blocks = vec![];
        let mut prev_block_number = new_block_number - 1;

        loop {
            if prev_block_number == 0 {
                break;
            }

            // 获取上一个区块, 返回的值是none的话说明rpc错误或者区块id计算错误
            let block = self
                .provider
                .get_block_by_number(BlockNumberOrTag::Number(prev_block_number))
                .await?
                .expect(&format!("error prev_block_number:{}", prev_block_number));

            // 此block不一致了
            let parent_hash = format!("{:#x}", block.header.parent_hash);
            blocks.push(block);

            // 获取数据库里的区块, 返回的值是空的话，说明数据库数据缺失了, 这里先简单的认为是数据库到头了
            let Some(prev_block) = self
                .repo
                .get_block_by_number(prev_block_number as i64 - 1)
                .await?
            else {
                break;
            };

            if prev_block.hash.to_lowercase() == parent_hash.to_lowercase() {
                // 结束
                break;
            }
            prev_block_number -= 1;
        }

        Ok(blocks)
    }

    /// 删除跟这个区块有关的数据库数据
    pub async fn handle_reorg(&self, block: &Block) -> Result<()> {
        // 获取所有的交易
        let mut transactions = self
            .repo
            .get_transactions_by_number(block.number() as i64)
            .await?;
        // 根据index倒排序交易
        transactions.sort_by(|v1, v2| v2.transaction_index.cmp(&v1.transaction_index));

        for t in transactions.iter() {
            // 获取所有的日志
            let mut logs = self.repo.get_logs_by_tx_hash(&t.hash).await?;
            // 根据index倒排序日志
            logs.sort_by(|v1, v2| v2.log_index.cmp(&v1.log_index));

            // 处理日志
            for log in logs {
                let Some(t0) = log.topic0.clone() else {
                    continue;
                };
                if t0.to_lowercase() != TRANSFER_TOPIC {
                    continue;
                }
                let Some(event) = decode_transaction_log(&log) else {
                    continue;
                };
                debug!(
                    "Reorg on {} | from={} to={} value={} nft={}",
                    log.address, event.from, event.to, event.value, event.is_nft
                );

                if event.is_nft {
                    // erc721
                    self.repo
                        .update_erc721_holder(&log.address, &event.from, &event.value, "")
                        .await?;
                    self.repo
                        .update_erc721_balance(&log.address, &event.to, &event.from, 1)
                        .await?;
                } else {
                    self.repo
                        .update_erc20_balance(&log.address, &event.to, &event.from, &event.value)
                        .await?;
                }
            }
        }

        // 删除数据
        let number = block.number() as i64;
        self.repo.delete_block(number).await?;

        if number != 0 {
            self.repo
                .update_sync_state(number - 1, &format!("{:#x}", block.header.parent_hash))
                .await?;
        }

        info!("✓ Block {} reorg ({} txs)", number, transactions.len());
        Ok(())
    }
}
