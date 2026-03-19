use super::cache::Cache;
use super::queries;
use crate::models;
use anyhow::Result;
use bigdecimal::BigDecimal;
use num_traits::{Signed, Zero};
use sqlx::PgPool;

pub struct Repository {
    pool: PgPool,
    cache: Cache,
}

impl Repository {
    pub fn new(pool: PgPool) -> Self {
        Self {
            pool,
            cache: Cache::new(),
        }
    }
    
    /// Expose the pool for modules that need raw SQL (e.g. reorg handler).
    #[allow(unused)]
    pub fn pool(&self) -> &PgPool {
        &self.pool
    }
    
    // ── Sync state ─────────────────────────────────────────────────────────────

    pub async fn get_sync_state(&self) -> Result<Option<models::SyncState>> {
        if let Some(ss) = self.cache.get_sync_state() {
            Ok(Some(ss))
        } else {
            let row: Option<models::SyncState> = sqlx::query_as(queries::GET_SYNC_STATE)
                .fetch_optional(&self.pool)
                .await?;
            if let Some(ref ss) = row {
                self.cache.set_sync_state(ss.clone());
            }
            Ok(row)
        }
    }

    pub async fn update_sync_state(&self, last_block: i64, last_block_hash: &str) -> Result<()> {
        sqlx::query(queries::UPSERT_SYNC_STATE)
            .bind(last_block)
            .bind(last_block_hash)
            .execute(&self.pool)
            .await?;
        self.cache.update_sync_state(last_block as u64, last_block_hash);
        Ok(())
    }

    // ── Blocks ─────────────────────────────────────────────────────────────────

    pub async fn insert_block(&self, b: &models::Block) -> Result<()> {
        sqlx::query(queries::INSERT_BLOCK)
            .bind(&b.number)
            .bind(&b.hash)
            .bind(&b.parent_hash)
            .bind(&b.nonce)
            .bind(&b.miner)
            .bind(&b.state_root)
            .bind(&b.transactions_root)
            .bind(&b.receipts_root)
            .bind(&b.logs_bloom)
            .bind(&b.difficulty)
            .bind(&b.gas_limit)
            .bind(b.gas_used)
            .bind(b.timestamp)
            .bind(&b.extra_data)
            .bind(&b.mix_hash)
            .bind(&b.base_fee_per_gas)
            .bind(&b.withdrawals_root)
            .bind(&b.blob_gas_used)
            .bind(&b.excess_blob_gas)
            .bind(&b.parent_beacon_block_root)
            .bind(&b.requests_hash)
            .bind(&b.sha3_uncles)
            .bind(&b.transaction_count)
            .bind(&b.withdrawal_count)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn get_block_by_number(&self, number: i64) -> Result<Option<models::Block>> {
        let row = sqlx::query_as(queries::GET_BLOCK_BY_NUMBER)
            .bind(&number)
            .fetch_optional(&self.pool)
            .await?;
        Ok(row)
    }

    #[allow(unused)]
    pub async fn get_block_by_hash(&self, hash: &str) -> Result<Option<models::Block>> {
        let row = sqlx::query_as(queries::GET_BLOCK_BY_HASH)
            .bind(hash)
            .fetch_optional(&self.pool)
            .await?;
        Ok(row)
    }

    /// 无排序
    #[allow(unused)]
    pub async fn get_block_by_miner(&self, miner: &str) -> Result<Vec<models::Block>> {
        let row = sqlx::query_as(queries::GET_BLOCK_BY_MINER)
            .bind(miner)
            .fetch_all(&self.pool)
            .await?;
        Ok(row)
    }

    /// 有序
    #[allow(unused)]
    pub async fn get_blocks_from_number(
        &self,
        number: i64,
        limit: i64,
    ) -> Result<Vec<models::Block>> {
        let rows = sqlx::query_as(queries::GET_BLOCKS_FROM_NUMBER)
            .bind(&number)
            .bind(&limit)
            .fetch_all(&self.pool)
            .await?;
        Ok(rows)
    }

    /// 无排序
    #[allow(unused)]
    pub async fn get_blocks_from_timestamp(
        &self,
        from: i64,
        to: i64,
    ) -> Result<Vec<models::Block>> {
        let rows = sqlx::query_as(queries::GET_BLOCKS_FROM_TIMESTAMP)
            .bind(&from)
            .bind(&to)
            .fetch_all(&self.pool)
            .await?;
        Ok(rows)
    }

    /// 删除区块
    pub async fn delete_block(&self, block_number: i64) -> Result<()> {
        sqlx::query(queries::DELETE_BLOCK_BY_NUMBER)
            .bind(block_number)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    // ── Transactions ───────────────────────────────────────────────────────────

    pub async fn insert_transaction(&self, tx: &models::Transaction) -> Result<()> {
        sqlx::query(queries::INSERT_TRANSACTION)
            .bind(&tx.hash)
            .bind(&tx.block_number)
            .bind(&tx.block_hash)
            .bind(&tx.transaction_index)
            .bind(&tx.from_address)
            .bind(&tx.to_address)
            .bind(&tx.value)
            .bind(&tx.chain_id)
            .bind(&tx.nonce)
            .bind(&tx.gas_limit)
            .bind(&tx.gas_used)
            .bind(&tx.effective_gas_price)
            .bind(&tx.blob_gas_used)
            .bind(&tx.blob_gas_price)
            .bind(&tx.contract_address)
            .bind(&tx.status)
            .bind(tx.cumulative_gas_used)
            .bind(&tx.tx_type)
            .bind(&tx.input)
            .bind(&tx.gas_price)
            .bind(&tx.max_fee_per_gas)
            .bind(&tx.max_priority_fee_per_gas)
            .bind(&tx.access_list)
            .bind(&tx.authorization_list)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn get_transactions_by_number(
        &self,
        number: i64,
    ) -> Result<Vec<models::Transaction>> {
        let row = sqlx::query_as(queries::GET_TRANSACTION_BY_NUMBER)
            .bind(&number)
            .fetch_all(&self.pool)
            .await?;
        Ok(row)
    }

    #[allow(unused)]
    pub async fn get_transaction_by_hash(&self, hash: &str) -> Result<Option<models::Transaction>> {
        let row = sqlx::query_as(queries::GET_TRANSACTION_BY_HASH)
            .bind(hash)
            .fetch_optional(&self.pool)
            .await?;
        Ok(row)
    }

    #[allow(unused)]
    pub async fn get_transactions_by_address(
        &self,
        address: &str,
    ) -> Result<Vec<models::Transaction>> {
        let rows = sqlx::query_as(queries::GET_TRANSACTION_BY_ADDRESS)
            .bind(address)
            .fetch_all(&self.pool)
            .await?;
        Ok(rows)
    }

    // ── Logs ───────────────────────────────────────────────────────────────────

    pub async fn insert_log(&self, log: &models::TransactionLog) -> Result<()> {
        sqlx::query(queries::INSERT_LOG)
            .bind(&log.transaction_hash)
            .bind(&log.block_number)
            .bind(&log.log_index)
            .bind(&log.address)
            .bind(&log.topic0)
            .bind(&log.topic1)
            .bind(&log.topic2)
            .bind(&log.topic3)
            .bind(&log.data)
            .bind(&log.removed)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn get_logs_by_tx_hash(&self, hash: &str) -> Result<Vec<models::TransactionLog>> {
        let rows = sqlx::query_as(queries::GET_LOG_BY_HASH)
            .bind(hash)
            .fetch_all(&self.pool)
            .await?;
        Ok(rows)
    }

    // ── ETH Transfers ──────────────────────────────────────────────────────────

    #[allow(unused)]
    pub async fn insert_eth_transfer(&self, t: &models::EthTransfer) -> Result<()> {
        sqlx::query(queries::INSERT_ETH_TRANSFER)
            .bind(&t.transaction_hash)
            .bind(&t.block_number)
            .bind(&t.from_address)
            .bind(&t.to_address)
            .bind(&t.value)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    // ── ERC-20 ─────────────────────────────────────────────────────────────────

    #[allow(unused)]
    pub async fn insert_erc20_meta(&self, t: &models::Erc20Meta) -> Result<()> {
        sqlx::query(queries::INSERT_ERC20_META)
            .bind(&t.contract_address)
            .bind(&t.name)
            .bind(&t.symbol)
            .bind(&t.creator)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn insert_erc20_transfer(&self, t: &models::Erc20Transfer) -> Result<()> {
        sqlx::query(queries::INSERT_ERC20_TRANSFER)
            .bind(&t.transaction_hash)
            .bind(&t.block_number)
            .bind(&t.log_index)
            .bind(&t.contract_address)
            .bind(&t.from_address)
            .bind(&t.to_address)
            .bind(&t.amount)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn update_erc20_balance(
        &self,
        contract: &str,
        from: &str,
        to: &str,
        amount: &BigDecimal,
    ) -> Result<()> {
        if amount.is_zero() || amount.is_negative() {
            return Ok(());
        }
        if amount.is_negative() {
            panic!("negative erc20 amount");
        }

        // Deduct from sender (skip zero/mint address)
        if from != Self::zero_address() {
            let new_amount = amount * -1;
            sqlx::query(queries::UPSERT_ERC20_BALANCE)
                .bind(&contract)
                .bind(&from)
                .bind(&new_amount)
                .execute(&self.pool)
                .await?;
        }

        // Add to receiver (skip burn address)
        if to != Self::zero_address() {
            sqlx::query(queries::UPSERT_ERC20_BALANCE)
                .bind(&contract)
                .bind(&to)
                .bind(&amount)
                .execute(&self.pool)
                .await?;
        }

        Ok(())
    }

    #[allow(unused)]
    pub async fn get_erc20_transfers_by_contract(
        &self,
        contract: &str,
    ) -> Result<Vec<models::Erc20Transfer>> {
        let rows = sqlx::query_as(queries::GET_ERC20_TRANSFER_BY_CONTRACT)
            .bind(contract)
            .fetch_all(&self.pool)
            .await?;
        Ok(rows)
    }

    #[allow(unused)]
    pub async fn get_erc20_balance(
        &self,
        contract: &str,
        holder: &str,
    ) -> Result<Option<models::Erc20Balance>> {
        let row = sqlx::query_as(queries::GET_ERC20_BALANCE)
            .bind(contract)
            .bind(holder)
            .fetch_optional(&self.pool)
            .await?;
        Ok(row)
    }

    // ── ERC-721 ────────────────────────────────────────────────────────────────

    #[allow(unused)]
    pub async fn insert_erc721_meta(&self, t: &models::Erc721Meta) -> Result<()> {
        sqlx::query(queries::INSERT_ERC721_META)
            .bind(&t.contract_address)
            .bind(&t.name)
            .bind(&t.symbol)
            .bind(&t.creator)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn insert_erc721_transfer(&self, t: &models::Erc721Transfer) -> Result<()> {
        sqlx::query(queries::INSERT_ERC721_TRANSFER)
            .bind(&t.transaction_hash)
            .bind(&t.block_number)
            .bind(&t.log_index)
            .bind(&t.contract_address)
            .bind(&t.from_address)
            .bind(&t.to_address)
            .bind(&t.token_id)
            .bind("")
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn update_erc721_balance(
        &self,
        contract: &str,
        from: &str,
        to: &str,
        token_count: i64,
    ) -> Result<()> {
        if from != Self::zero_address() {
            sqlx::query(queries::UPSERT_ERC721_BALANCE)
                .bind(contract)
                .bind(from)
                .bind(token_count * -1)
                .execute(&self.pool)
                .await?;
        }

        if to != Self::zero_address() {
            sqlx::query(queries::UPSERT_ERC721_BALANCE)
                .bind(contract)
                .bind(to)
                .bind(token_count)
                .execute(&self.pool)
                .await?;
        }

        Ok(())
    }

    #[allow(unused)]
    pub async fn get_erc721_transfers_by_contract(
        &self,
        contract: &str,
    ) -> Result<Vec<models::Erc721Transfer>> {
        let rows = sqlx::query_as(queries::GET_ERC721_TRANSFER_BY_CONTRACT)
            .bind(contract)
            .fetch_all(&self.pool)
            .await?;
        Ok(rows)
    }

    #[allow(unused)]
    pub async fn get_erc721_balance(
        &self,
        contract: &str,
        holder: &str,
    ) -> Result<Option<models::Erc721Balance>> {
        let row = sqlx::query_as(queries::GET_ERC721_BALANCE)
            .bind(contract)
            .bind(holder)
            .fetch_optional(&self.pool)
            .await?;
        Ok(row)
    }

    pub async fn update_erc721_holder(
        &self,
        contract: &str,
        holder: &str,
        token_id: &BigDecimal,
        url: &str,
    ) -> Result<()> {
        sqlx::query(queries::UPSERT_ERC721_HOLDER)
            .bind(token_id)
            .bind(contract)
            .bind(holder)
            .bind(url)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    #[allow(unused)]
    pub async fn get_erc721_holder(
        &self,
        contract: &str,
        holder: &str,
    ) -> Result<Vec<models::Erc721Holder>> {
        let row = sqlx::query_as(queries::GET_ERC721_HOLDER)
            .bind(contract)
            .bind(holder)
            .fetch_all(&self.pool)
            .await?;
        Ok(row)
    }

    fn zero_address() -> &'static str {
        "0x0000000000000000000000000000000000000000"
    }
}
