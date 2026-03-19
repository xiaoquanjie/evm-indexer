use crate::process::event_decoder::{decode_transfer, u256_to_bigdecimal, TRANSFER_TOPIC};
use crate::{db, models};
use alloy::consensus::Transaction;
use alloy::network::ReceiptResponse;
use alloy::primitives::U256;
use alloy::rpc::types::eth::{
    Block, BlockTransactions, Log, Transaction as BlockTransaction, TransactionReceipt,
};
use anyhow::Result;
use std::sync::Arc;
use tracing::{debug, info};

pub struct Processor {
    repo: Arc<db::Repository>,
}

impl Processor {
    pub fn new(repo: Arc<db::Repository>) -> Self {
        Self { repo }
    }

    /// Process a full block: store block header, transactions, receipts (logs).
    pub async fn process_block(
        &self,
        block: &Block,
        receipts: &[TransactionReceipt],
    ) -> Result<()> {
        let block_number = block.header.number as i64;

        // 如果处理过了就不再处理，这是因为有些状态表无法接收重新处理，除非后面将状态表移除采取别的方式建立状态表
        if self.repo.get_block_by_number(block_number).await?.is_some() {
            return Ok(());
        }

        // ── 1. Store block header ───────────────────────────────────────────
        self.store_block(block).await?;

        // ── 2. Store transactions ───────────────────────────────────────────
        let txs = match &block.transactions {
            BlockTransactions::Full(txs) => txs,
            _ => {
                panic!(
                    "Block {} has no full transaction data, skipping txs",
                    block_number
                );
            }
        };

        if txs.len() != receipts.len() {
            panic!("error data");
        }

        for i in 0..receipts.len() {
            self.store_transaction(&txs[i], &receipts[i]).await?;
        }

        self.repo
            .update_sync_state(block_number, &format!("{:#x}", block.hash()))
            .await?;

        info!("✓ Block {} processed ({} txs)", block_number, txs.len());
        Ok(())
    }

    // ── Block header ─────────────────────────────────────────────────────────

    async fn store_block(&self, block: &Block) -> Result<()> {
        let h = &block.header;
        let insert = models::Block {
            number: h.number as i64,
            hash: format!("{:#x}", h.hash),
            parent_hash: format!("{:#x}", h.parent_hash),
            nonce: Some(format!("{:#x}", h.nonce)),
            miner: format!("{:#x}", h.beneficiary),
            state_root: Some(format!("{:#x}", h.state_root)),
            transactions_root: Some(format!("{:#x}", h.transactions_root)),
            receipts_root: Some(format!("{:#x}", h.receipts_root)),
            logs_bloom: Some(format!("{:#x}", h.logs_bloom)),
            difficulty: Some(u256_to_bigdecimal(h.difficulty)),
            gas_limit: h.gas_limit as i64,
            gas_used: h.gas_used as i64,
            timestamp: h.timestamp as i64,
            extra_data: h.extra_data.to_string().into(),
            mix_hash: Some(format!("{:#x}", h.mix_hash)),
            base_fee_per_gas: h.base_fee_per_gas.map(|u| u as i64),
            withdrawals_root: h.withdrawals_root.map(|r| format!("{:#x}", r)),
            blob_gas_used: h.blob_gas_used.map(|u| u as i64),
            excess_blob_gas: h.excess_blob_gas.map(|u| u as i64),
            parent_beacon_block_root: h.parent_beacon_block_root.map(|r| format!("{:#x}", r)),
            requests_hash: h.requests_hash.map(|r| format!("{:#x}", r)),
            sha3_uncles: Some(format!("{:#x}", h.ommers_hash)),
            transaction_count: match &block.transactions {
                BlockTransactions::Full(v) => v.len() as i32,
                BlockTransactions::Hashes(v) => v.len() as i32,
                BlockTransactions::Uncle => 0,
            },
            withdrawal_count: block
                .withdrawals
                .as_ref()
                .map(|w| w.0.len() as i32)
                .unwrap_or_default(),
            created_at: chrono::DateTime::from_timestamp(0, 0).unwrap(),
        };

        self.repo.insert_block(&insert).await
    }

    // ── Transaction & receipts ────────────────────────────────────────────────

    async fn store_transaction(
        &self,
        tx: &BlockTransaction,
        receipt: &TransactionReceipt,
    ) -> Result<()> {
        let insert = models::Transaction {
            hash: format!("{:#x}", receipt.transaction_hash),
            block_number: receipt.block_number.map(|u| u as i64).unwrap_or_default(),
            block_hash: format!("{:#x}", receipt.block_hash.as_ref().unwrap()),
            transaction_index: receipt
                .transaction_index
                .map(|u| u as i32)
                .unwrap_or_default(),
            from_address: format!("{:#x}", receipt.from),
            to_address: receipt.to.map(|u| format!("{:#x}", u)),
            value: u256_to_bigdecimal(tx.value()),
            chain_id: tx.chain_id().map(|u| u as i64).unwrap_or_default(),
            nonce: tx.nonce() as i64,
            gas_limit: tx.gas_limit() as i64,
            gas_used: receipt.gas_used as i64,
            effective_gas_price: Some(u256_to_bigdecimal(U256::from(receipt.effective_gas_price))),
            blob_gas_used: receipt.blob_gas_used.map(|u| u as i64),
            blob_gas_price: receipt
                .blob_gas_price
                .map(|u| u256_to_bigdecimal(U256::from(u))),
            contract_address: receipt.contract_address.map(|u| format!("{:#x}", u)),
            status: Some(if receipt.status() { 1 } else { 0 }),
            cumulative_gas_used: receipt.cumulative_gas_used() as i64,
            tx_type: receipt.transaction_type() as i16,
            input: if tx.input().is_empty() {
                None
            } else {
                Some(hex::encode(tx.input().as_ref()))
            },
            gas_price: tx.gas_price().map(|u| u256_to_bigdecimal(U256::from(u))),
            max_fee_per_gas: Some(u256_to_bigdecimal(U256::from(tx.max_fee_per_gas()))),
            max_priority_fee_per_gas: tx
                .max_priority_fee_per_gas()
                .map(|u| u256_to_bigdecimal(U256::from(u))),
            access_list: tx
                .access_list()
                .map(|a| serde_json::to_string(a).unwrap_or(String::new())),
            authorization_list: tx
                .authorization_list()
                .map(|a| serde_json::to_string(a).unwrap_or(String::new())),
            created_at: chrono::DateTime::from_timestamp(0, 0).unwrap(),
        };

        self.repo.insert_transaction(&insert).await?;

        // ── Process logs from receipt ──────────────────────────────────────
        for log in receipt.logs() {
            self.store_log(log).await?;
        }

        Ok(())
    }

    // ── Log processing ────────────────────────────────────────────────────────

    async fn store_log(&self, log: &Log) -> Result<()> {
        let topic_str =
            |i: usize| -> Option<String> { log.topics().get(i).map(|t| format!("{:#x}", t)) };

        let insert = models::TransactionLog {
            id: 0,
            transaction_hash: format!("{:#x}", log.transaction_hash.unwrap()),
            block_number: log.block_number.unwrap_or(0) as i64,
            log_index: log.log_index.unwrap_or(0) as i32,
            address: format!("{:#x}", log.address()),
            topic0: topic_str(0),
            topic1: topic_str(1),
            topic2: topic_str(2),
            topic3: topic_str(3),
            data: Some(format!("{:#x}", log.data().data)), //Some(hex::encode(log.data().data.as_ref())),
            removed: log.removed,
            created_at: chrono::DateTime::from_timestamp(0, 0).unwrap(),
        };

        self.repo.insert_log(&insert).await?;

        // ── Decode Transfer events ────────────────────────────────────────
        if let Some(t0) = log.topic0() {
            let sig = format!("{:#x}", t0);
            if sig.to_lowercase() == TRANSFER_TOPIC {
                if let Some(event) = decode_transfer(log.topics(), log.data().data.as_ref()) {
                    debug!(
                        "Transfer on {} | from={} to={} value={} nft={}",
                        insert.address, event.from, event.to, event.value, event.is_nft
                    );
                    if event.is_nft {
                        let insert_721 = models::Erc721Transfer {
                            id: 0,
                            transaction_hash: insert.transaction_hash,
                            block_number: insert.block_number,
                            log_index: insert.log_index,
                            contract_address: insert.address.clone(),
                            from_address: event.from.clone(),
                            to_address: event.to.clone(),
                            token_id: event.value.clone(),
                            url: None,
                            created_at: chrono::DateTime::from_timestamp(0, 0).unwrap(),
                        };
                        self.repo.insert_erc721_transfer(&insert_721).await?;
                        self.repo
                            .update_erc721_balance(&insert.address, &event.from, &event.to, 1)
                            .await?;
                        self.repo
                            .update_erc721_holder(&insert.address, &event.to, &event.value, "")
                            .await?;
                    } else {
                        let insert_20 = models::Erc20Transfer {
                            id: 0,
                            transaction_hash: insert.transaction_hash,
                            block_number: insert.block_number,
                            log_index: insert.log_index,
                            contract_address: insert.address.clone(),
                            from_address: event.from.clone(),
                            to_address: event.to.clone(),
                            amount: event.value.clone(),
                            created_at: chrono::DateTime::from_timestamp(0, 0).unwrap(),
                        };
                        self.repo.insert_erc20_transfer(&insert_20).await?;
                        self.repo
                            .update_erc20_balance(
                                &insert.address,
                                &event.from,
                                &event.to,
                                &event.value,
                            )
                            .await?;
                    }
                }
            }
        }
        Ok(())
    }
}
