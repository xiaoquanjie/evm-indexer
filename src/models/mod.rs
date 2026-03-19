use bigdecimal::BigDecimal;
use chrono::{DateTime, Utc};
use sqlx::FromRow;

#[derive(Debug, Clone, FromRow)]
pub struct SyncState {
    #[allow(unused)]
    pub id: i32,
    pub last_block: i64,
    pub last_block_hash: String,
    #[allow(unused)]
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow)]
pub struct Block {
    pub number: i64,
    pub hash: String,
    pub parent_hash: String,
    pub nonce: Option<String>,
    pub miner: String,
    pub state_root: Option<String>,
    pub transactions_root: Option<String>,
    pub receipts_root: Option<String>,
    pub logs_bloom: Option<String>,
    pub difficulty: Option<BigDecimal>,
    pub gas_limit: i64,
    pub gas_used: i64,
    pub timestamp: i64,
    pub extra_data: Option<String>,
    pub mix_hash: Option<String>,
    pub base_fee_per_gas: Option<i64>,
    pub withdrawals_root: Option<String>,
    pub blob_gas_used: Option<i64>,
    pub excess_blob_gas: Option<i64>,
    pub parent_beacon_block_root: Option<String>,
    pub requests_hash: Option<String>,
    pub sha3_uncles: Option<String>,
    pub transaction_count: i32,
    pub withdrawal_count: i32,
    #[allow(unused)]
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow)]
pub struct Transaction {
    pub hash: String,
    pub block_number: i64,
    pub block_hash: String,
    pub transaction_index: i32,
    pub from_address: String,
    pub to_address: Option<String>,
    pub value: BigDecimal,
    pub chain_id: i64,
    pub nonce: i64,
    pub gas_limit: i64,
    pub gas_used: i64,
    pub effective_gas_price: Option<BigDecimal>,
    pub blob_gas_used: Option<i64>,
    pub blob_gas_price: Option<BigDecimal>,
    pub contract_address: Option<String>,
    pub status: Option<i16>,
    pub cumulative_gas_used: i64,
    pub tx_type: i16,
    pub input: Option<String>,
    pub gas_price: Option<BigDecimal>,
    pub max_fee_per_gas: Option<BigDecimal>,
    pub max_priority_fee_per_gas: Option<BigDecimal>,
    pub access_list: Option<String>,
    pub authorization_list: Option<String>,
    #[allow(unused)]
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow)]
pub struct TransactionLog {
    #[allow(unused)]
    pub id: i64,
    pub transaction_hash: String,
    pub block_number: i64,
    pub log_index: i32,
    pub address: String,
    pub topic0: Option<String>,
    pub topic1: Option<String>,
    pub topic2: Option<String>,
    pub topic3: Option<String>,
    pub data: Option<String>,
    pub removed: bool,
    #[allow(unused)]
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow)]
#[allow(unused)]
pub struct EthTransfer {
    pub id: i64,
    pub transaction_hash: String,
    pub block_number: i64,
    pub from_address: String,
    pub to_address: String,
    pub value: BigDecimal,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow)]
#[allow(unused)]
pub struct Erc20Meta {
    pub contract_address: String,
    pub name: String,
    pub symbol: String,
    pub creator: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow)]
pub struct Erc20Transfer {
    #[allow(unused)]
    pub id: i64,
    pub transaction_hash: String,
    pub block_number: i64,
    pub log_index: i32,
    pub contract_address: String,
    pub from_address: String,
    pub to_address: String,
    pub amount: BigDecimal,
    #[allow(unused)]
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow)]
#[allow(unused)]
pub struct Erc20Balance {
    pub contract_address: String,
    pub holder_address: String,
    pub balance: BigDecimal,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow)]
#[allow(unused)]
pub struct Erc721Meta {
    pub contract_address: String,
    pub name: String,
    pub symbol: String,
    pub creator: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow)]
pub struct Erc721Transfer {
    #[allow(unused)]
    pub id: i64,
    pub transaction_hash: String,
    pub block_number: i64,
    pub log_index: i32,
    pub contract_address: String,
    pub from_address: String,
    pub to_address: String,
    pub token_id: BigDecimal,
    #[allow(unused)]
    pub url: Option<String>,
    #[allow(unused)]
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow)]
#[allow(unused)]
pub struct Erc721Balance {
    pub contract_address: String,
    pub holder_address: String,
    pub token_count: i64,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow)]
#[allow(unused)]
pub struct Erc721Holder {
    pub token_id: BigDecimal,
    pub contract_address: String,
    pub holder_address: String,
    pub url: Option<String>,
    pub updated_at: DateTime<Utc>,
}