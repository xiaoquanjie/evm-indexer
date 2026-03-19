// ─── Sync state ───────────────────────────────────────────────────────────────

pub const GET_SYNC_STATE: &str = "SELECT * FROM sync_state WHERE id = 1";

pub const UPSERT_SYNC_STATE: &str = r#"
INSERT INTO sync_state (
    last_block,
    last_block_hash
) VALUES ($1, $2) ON CONFLICT (id)
DO UPDATE SET
    last_block = $1,
    last_block_hash = $2
"#;

// ─── Blocks ───────────────────────────────────────────────────────────────────

pub const INSERT_BLOCK: &str = r#"
INSERT INTO blocks (
    number,
    hash,
    parent_hash,
    nonce,
    miner,
    state_root,
    transactions_root,
    receipts_root,
    logs_bloom,
    difficulty,
    gas_limit,
    gas_used,
    timestamp,
    extra_data,
    mix_hash,
    base_fee_per_gas,
    withdrawals_root,
    blob_gas_used,
    excess_blob_gas,
    parent_beacon_block_root,
    requests_hash,
    sha3_uncles,
    transaction_count,
    withdrawal_count
) VALUES (
    $1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16,$17,$18,$19,$20,$21,$22,$23,$24
) ON CONFLICT (number) DO NOTHING
"#;

pub const GET_BLOCK_BY_NUMBER: &str = r#"
SELECT * FROM blocks WHERE number = $1
"#;

#[allow(unused)]
pub const GET_BLOCK_BY_HASH: &str = r#"
SELECT * FROM blocks WHERE hash = $1
"#;

#[allow(unused)]
pub const GET_BLOCK_BY_MINER: &str = r#"
SELECT * FROM blocks WHERE miner = $1"#;

#[allow(unused)]
pub const GET_BLOCKS_FROM_NUMBER: &str = r#"
SELECT * FROM blocks WHERE number >= $1 ORDER BY number LIMIT $2
"#;

pub const GET_BLOCKS_FROM_TIMESTAMP: &str = r#"
SELECT * FROM blocks WHERE timestamp >= $1 AND timestamp <= $2
"#;

pub const DELETE_BLOCK_BY_NUMBER: &str = r#"
DELETE FROM blocks WHERE number = $1
"#;

// ─── Transactions ─────────────────────────────────────────────────────────────

pub const INSERT_TRANSACTION: &str = r#"
INSERT INTO transactions (
    hash,
    block_number,
    block_hash,
    transaction_index,
    from_address,
    to_address,
    value,
    chain_id,
    nonce,
    gas_limit,
    gas_used,
    effective_gas_price,
    blob_gas_used,
    blob_gas_price,
    contract_address,
    status,
    cumulative_gas_used,
    tx_type,
    input,
    gas_price,
    max_fee_per_gas,
    max_priority_fee_per_gas,
    access_list,
    authorization_list
) VALUES (
    $1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16,$17,$18,$19,$20,$21,$22,$23,$24
) ON CONFLICT (hash) DO NOTHING
"#;

pub const GET_TRANSACTION_BY_NUMBER: &str = r#"
SELECT * FROM transactions WHERE block_number = $1
"#;

#[allow(unused)]
pub const GET_TRANSACTION_BY_HASH: &str = r#"
SELECT * FROM transactions WHERE hash = $1
"#;

#[allow(unused)]
pub const GET_TRANSACTION_BY_ADDRESS: &str = r#"
SELECT * FROM transactions WHERE from_address = $1 or to_address = $1
"#;

#[allow(unused)]
pub const FETCH_TRANSACTIONS_HASH: &str = r#"
SELECT hash, transaction_index FROM transactions WHERE block_number = $1
"#;

#[allow(unused)]
pub const DELETE_TRANSACTION_BY_HASH: &str = r#"
DELETE FROM transactions WHERE hash = $1
"#;

// ─── Logs ─────────────────────────────────────────────────────────────────────

pub const INSERT_LOG: &str = r#"
INSERT INTO transaction_logs (
    transaction_hash,
    block_number,
    log_index,
    address,
    topic0,
    topic1,
    topic2,
    topic3,
    data,
    removed
) VALUES (
    $1,$2,$3,$4,$5,$6,$7,$8,$9,$10
) ON CONFLICT (transaction_hash, log_index) DO NOTHING
"#;

pub const GET_LOG_BY_HASH: &str = r#"
SELECT * FROM transaction_logs WHERE transaction_hash = $1
"#;

// ─── ETH Transfers ────────────────────────────────────────────────────────────

#[allow(unused)]
pub const INSERT_ETH_TRANSFER: &str = r#"
INSERT INTO eth_transfers (
    transaction_hash,
    block_number,
    from_address,
    to_address,
    value
) VALUES ($1,$2,$3,$4,$5)
"#;

// ─── ERC-20 ───────────────────────────────────────────────────────────────────

pub const INSERT_ERC20_META: &str = r#"
INSERT INTO erc20_meta (
    contract_address,
    name,
    symbol,
    creator
) VALUES ($1, $2, $3, $4)
"#;

#[allow(unused)]
pub const INSERT_ERC20_TRANSFER: &str = r#"
INSERT INTO erc20_transfers (
    transaction_hash,
    block_number,
    log_index,
    contract_address,
    from_address,
    to_address,
    amount
) VALUES ($1,$2,$3,$4,$5,$6,$7)
ON CONFLICT (transaction_hash, log_index) DO NOTHING
"#;

#[allow(unused)]
pub const GET_ERC20_TRANSFER_BY_CONTRACT: &str = r#"
SELECT * FROM erc20_transfers WHERE contract_address = $1
"#;

#[allow(unused)]
pub const UPSERT_ERC20_BALANCE: &str = r#"
INSERT INTO erc20_balances (
    contract_address,
    holder_address,
    balance
) VALUES ($1, $2, $3)
ON CONFLICT (contract_address, holder_address)
DO UPDATE SET
    balance = erc20_balances.balance + $3,
    updated_at = NOW()
"#;

#[allow(unused)]
pub const GET_ERC20_BALANCE: &str = r#"
SELECT * FROM erc20_balances WHERE contract_address = $1 AND holder_address = $2
"#;

// ─── ERC-721 ──────────────────────────────────────────────────────────────────

#[allow(unused)]
pub const INSERT_ERC721_META: &str = r#"
INSERT INTO erc721_meta (
    contract_address,
    name,
    symbol,
    creator
) VALUES ($1, $2, $3, $4)
"#;

#[allow(unused)]
pub const INSERT_ERC721_TRANSFER: &str = r#"
INSERT INTO erc721_transfers (
    transaction_hash, 
    block_number, 
    log_index,
    contract_address, 
    from_address, 
    to_address, 
    token_id,
    url
) VALUES ($1,$2,$3,$4,$5,$6,$7,$8)
ON CONFLICT (transaction_hash, log_index) DO NOTHING
"#;

#[allow(unused)]
pub const GET_ERC721_TRANSFER_BY_CONTRACT: &str = r#"
SELECT * FROM erc721_transfers WHERE contract_address = $1
"#;

pub const UPSERT_ERC721_BALANCE: &str = r#"
INSERT INTO erc721_balances (
    contract_address, 
    holder_address, 
    token_count
) VALUES ($1, $2, $3)
ON CONFLICT (contract_address, holder_address)
DO UPDATE SET
    token_count = erc721_balances.token_count + $3,
    updated_at = NOW()
"#;

#[allow(unused)]
pub const GET_ERC721_BALANCE: &str = r#"
SELECT * FROM erc721_balances WHERE contract_address = $1 AND holder_address = $2
"#;

pub const UPSERT_ERC721_HOLDER: &str = r#"
INSERT INTO erc721_holders (
    token_id,
    contract_address,
    holder_address,
    url
) VALUES ($1, $2, $3, $4)
on CONFLICT (contract_address, token_id) 
DO UPDATE SET
    holder_address = $3,
    updated_at = NOW()
"#;

#[allow(unused)]
pub const GET_ERC721_HOLDER: &str = r#"
SELECT * FROM erc721_holders WHERE contract_address = $1 AND holder_address = $2
"#;