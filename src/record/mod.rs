use std::sync::Arc;
use tracing::info;
use crate::config::AppConfig;
use crate::db::Repository;
use crate::process::processor::Processor;
use crate::process::reorg::Reorg;
use crate::scanner::historical::HistoricalScanner;

/// 将一些特定的历史数据入库
pub struct Record {
    config: AppConfig,
    repo: Arc<Repository>,
    scanner: HistoricalScanner,
}

impl Record {
    pub fn new(config: AppConfig, repo: Arc<Repository>, scanner: HistoricalScanner) -> Self {
        Record {
            config,
            repo,
            scanner,
        }
    }

    pub async fn run(&self) -> anyhow::Result<u64> {
        let reorg = Reorg::new(self.repo.clone(), self.scanner.get_provider());
        let processor = Processor::new(self.repo.clone());
        let mut new_block_id;

        loop {
            new_block_id = match self.repo.get_sync_state().await? {
                None => self.config.indexer.start_block,
                Some(ss) => ss.last_block as u64 + 1,
            };

            loop {
                let available_number = self.scanner.get_block_number().await?;
                if new_block_id > available_number {
                    tokio::time::sleep(tokio::time::Duration::from_secs(7)).await;
                    continue;
                }
                break;
            }

            let Some(new_block) = self.scanner.get_block(new_block_id).await?
            else {
                break;
            };

            let detect = reorg.detect_reorg(&new_block).await?;
            if detect.is_empty() {
                let receipts = self.scanner.get_receipts(new_block_id).await?;
                processor.process_block(&new_block, &receipts).await?;
            } else {
                for d in detect {
                    reorg.handle_reorg(&d).await?;
                }
            }
        }

        info!("Historical scan complete, last block: #{}", new_block_id);
        Ok(new_block_id)
    }
}