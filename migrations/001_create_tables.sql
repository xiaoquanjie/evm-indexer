-- ─────────────────────────────────────────────────
-- Sync state: 状态同步表
-- 状态数据，非增量表，数据量大小可预测
-- ─────────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS sync_state (
    id          INTEGER PRIMARY KEY DEFAULT 1,
    last_block  BIGINT  NOT NULL DEFAULT 0,
    last_block_hash CHAR(66) NOT NULL,
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT single_row CHECK (id = 1)
);

-- 不插入，因为不知道hash值
-- INSERT INTO sync_state (id, last_block) VALUES (1, 0)
--    ON CONFLICT (id) DO NOTHING;

-- ─────────────────────────────────────────────────
-- Blocks
-- 流水数据，增量表，数据量大小不可预测
-- ─────────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS blocks (
    number          BIGINT      PRIMARY KEY,
    hash            CHAR(66)    NOT NULL UNIQUE,
    parent_hash     CHAR(66)    NOT NULL,
    nonce           VARCHAR(20),
    miner           CHAR(42)    NOT NULL,
    state_root      CHAR(66),
    transactions_root CHAR(66),
    receipts_root   CHAR(66),
    logs_bloom      TEXT,
    difficulty      NUMERIC(80), -- 精度为80位整数
    gas_limit       BIGINT NOT NULL,
    gas_used        BIGINT NOT NULL,
    timestamp       BIGINT NOT NULL,
    extra_data      TEXT, --  由矿工填写的任意数据，例子矿工身份
    mix_hash        CHAR(66),
    base_fee_per_gas BIGINT,
    withdrawals_root CHAR(66), -- 质押 ETH 提现数据的默克尔根，体现数据在区块中 EIP-4895，Shanghai硬分叉
    blob_gas_used   BIGINT,
    excess_blob_gas BIGINT,
    parent_beacon_block_root CHAR(66),
    requests_hash   CHAR(66),
    sha3_uncles     CHAR(66),
    transaction_count INTEGER NOT NULL DEFAULT 0,
    withdrawal_count  INTEGER NOT NULL DEFAULT 0,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- 创建blocks表索引
CREATE INDEX IF NOT EXISTS idx_blocks_hash      ON blocks (hash);
CREATE INDEX IF NOT EXISTS idx_blocks_timestamp ON blocks (timestamp);
CREATE INDEX IF NOT EXISTS idx_blocks_miner     ON blocks (miner);

-- ─────────────────────────────────────────────────
-- Transactions
-- 流水数据，增量表，数据量大小不可预测
-- ─────────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS transactions (
    hash                CHAR(66)    PRIMARY KEY,
    block_number        BIGINT      NOT NULL REFERENCES blocks(number) ON DELETE CASCADE,
    block_hash          CHAR(66)    NOT NULL,
    transaction_index   INTEGER     NOT NULL,
    from_address        CHAR(42)    NOT NULL,
    to_address          CHAR(42),               -- NULL for contract creation
    value               NUMERIC(80) NOT NULL,
    chain_id            BIGINT      NOT NULL,
    nonce               BIGINT      NOT NULL,
    gas_limit           BIGINT,
    gas_used            BIGINT,
    effective_gas_price NUMERIC(80),
    blob_gas_used       BIGINT,
    blob_gas_price      NUMERIC(80),
    contract_address    CHAR(42),               -- set on contract creation
    status              SMALLINT,               -- 1 = success, 0 = fail, NULL = pre-Byzantium
    cumulative_gas_used BIGINT,
    tx_type             SMALLINT    NOT NULL DEFAULT 0, -- 交易类型
    input               TEXT,

    -- 类型特有字段（可为NULL）
    -- Legacy/EIP-2930
    gas_price           NUMERIC(80),

    -- EIP-1559
    max_fee_per_gas     NUMERIC(80),
    max_priority_fee_per_gas NUMERIC(80),

    -- EIP-2930/EIP-1559/EIP-4844/EIP-7702
    access_list         TEXT,               -- 访问列表
    -- EIP-7702特有
    authorization_list  TEXT,               -- 授权列表

    created_at          TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- 交易索引
CREATE INDEX IF NOT EXISTS idx_tx_block_number   ON transactions (block_number);
CREATE INDEX IF NOT EXISTS idx_tx_from_address   ON transactions (from_address);
CREATE INDEX IF NOT EXISTS idx_tx_to_address     ON transactions (to_address);
CREATE INDEX IF NOT EXISTS idx_tx_contract_addr  ON transactions (contract_address) WHERE contract_address IS NOT NULL;

-- ─────────────────────────────────────────────────
-- Transaction logs
-- 流水数据，增量表，数据量大小不可预测
-- ─────────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS transaction_logs (
    id              BIGSERIAL   PRIMARY KEY,
    transaction_hash CHAR(66)   NOT NULL REFERENCES transactions(hash) ON DELETE CASCADE,
    block_number    BIGINT      NOT NULL,
    log_index       INTEGER     NOT NULL,
    address         CHAR(42)    NOT NULL,
    topic0          CHAR(66),
    topic1          CHAR(66),
    topic2          CHAR(66),
    topic3          CHAR(66),
    data            TEXT,
    removed         BOOLEAN     NOT NULL DEFAULT FALSE,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (transaction_hash, log_index)
);

-- 交易日志索引
CREATE INDEX IF NOT EXISTS idx_logs_tx_hash     ON transaction_logs (transaction_hash);
-- CREATE INDEX IF NOT EXISTS idx_logs_address     ON transaction_logs (address);
CREATE INDEX IF NOT EXISTS idx_logs_topic0      ON transaction_logs (topic0);
-- CREATE INDEX IF NOT EXISTS idx_logs_block       ON transaction_logs (block_number);

-- ─────────────────────────────────────────────────
-- ETH native value transfers
-- ─────────────────────────────────────────────────
-- CREATE TABLE IF NOT EXISTS eth_transfers (
--     id              BIGSERIAL   PRIMARY KEY,
--     transaction_hash CHAR(66)   NOT NULL REFERENCES transactions(hash) ON DELETE CASCADE,
--     block_number    BIGINT      NOT NULL,
--     from_address    CHAR(42)    NOT NULL,
--     to_address      CHAR(42)    NOT NULL,
--     value           NUMERIC(80) NOT NULL,   -- in Wei
--     created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
-- );

-- CREATE INDEX IF NOT EXISTS idx_eth_from         ON eth_transfers (from_address);
-- CREATE INDEX IF NOT EXISTS idx_eth_to           ON eth_transfers (to_address);
-- CREATE INDEX IF NOT EXISTS idx_eth_block        ON eth_transfers (block_number);
-- CREATE INDEX IF NOT EXISTS idx_eth_addr_union   ON eth_transfers (from_address, block_number);

-- ─────────────────────────────────────────────────
-- ERC-20 TOKEN meta
-- 状态数据，增量表，数据量大小可预测
-- ─────────────────────────────────────────────────

CREATE TABLE IF NOT EXISTS erc20_meta (
    contract_address CHAR(42)   NOT NULL PRIMARY KEY,
    name            TEXT        NOT NULL,
    symbol          TEXT        NOT NULL,
    creator         CHAR(42)    NOT NULL,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- ─────────────────────────────────────────────────
-- ERC-20 token transfers  (Transfer event)
-- topic0 = keccak256("Transfer(address,address,uint256)")
-- 流水数据，增量表，数据量大小不可预测
-- ─────────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS erc20_transfers (
    id              BIGSERIAL   PRIMARY KEY,
    transaction_hash CHAR(66)   NOT NULL REFERENCES transactions(hash) ON DELETE CASCADE,
    block_number    BIGINT      NOT NULL,
    log_index       INTEGER     NOT NULL,
    contract_address CHAR(42)   NOT NULL,
    from_address    CHAR(42)    NOT NULL,
    to_address      CHAR(42)    NOT NULL,
    amount          NUMERIC(80) NOT NULL,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (transaction_hash, log_index)
);

CREATE INDEX IF NOT EXISTS idx_erc20_contract   ON erc20_transfers (contract_address);
CREATE INDEX IF NOT EXISTS idx_erc20_from       ON erc20_transfers (contract_address, from_address);
CREATE INDEX IF NOT EXISTS idx_erc20_to         ON erc20_transfers (contract_address, to_address);

-- ─────────────────────────────────────────────────
-- ERC-20 balances
-- 状态数据，增量表，数据量大小不可预测
-- ─────────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS erc20_balances (
    contract_address CHAR(42)   NOT NULL,
    holder_address   CHAR(42)   NOT NULL,
    balance          NUMERIC(80) NOT NULL DEFAULT 0,
    updated_at       TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (contract_address, holder_address)
);

CREATE INDEX IF NOT EXISTS idx_erc20_bal_contract ON erc20_balances (contract_address);
CREATE INDEX IF NOT EXISTS idx_erc20_bal_holder   ON erc20_balances (holder_address);

-- ─────────────────────────────────────────────────
-- ERC-721 NFT meta
-- 状态数据，增量表，数据量大小可预测
-- ─────────────────────────────────────────────────

CREATE TABLE IF NOT EXISTS erc721_meta (
    contract_address CHAR(42)   NOT NULL PRIMARY KEY,
    name            TEXT        NOT NULL,
    symbol          TEXT       NOT NULL,
    creator         CHAR(42)    NOT NULL,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- ─────────────────────────────────────────────────
-- ERC-721 NFT transfers
-- topic0 = keccak256("Transfer(address,address,uint256)")
-- 流水数据，增量表，数据量大小不可预测
-- ─────────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS erc721_transfers (
    id              BIGSERIAL   PRIMARY KEY,
    transaction_hash CHAR(66)   NOT NULL REFERENCES transactions(hash) ON DELETE CASCADE,
    block_number    BIGINT      NOT NULL,
    log_index       INTEGER     NOT NULL,
    contract_address CHAR(42)   NOT NULL,
    from_address    CHAR(42)    NOT NULL,
    to_address      CHAR(42)    NOT NULL,
    token_id        NUMERIC(80) NOT NULL,
    url             TEXT,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (transaction_hash, log_index)
);

CREATE INDEX IF NOT EXISTS idx_erc721_contract   ON erc721_transfers (contract_address);
CREATE INDEX IF NOT EXISTS idx_erc721_from       ON erc721_transfers (contract_address, from_address);
CREATE INDEX IF NOT EXISTS idx_erc721_to         ON erc721_transfers (contract_address, to_address);
CREATE INDEX IF NOT EXISTS idx_erc721_token_id   ON erc721_transfers (contract_address, token_id);

-- ─────────────────────────────────────────────────
-- ERC-721 balances
-- 状态数据，增量表，数据量大小不可预测
-- ─────────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS erc721_balances (
    contract_address CHAR(42)   NOT NULL,
    holder_address   CHAR(42)   NOT NULL,
    token_count      BIGINT     NOT NULL DEFAULT 0,
    updated_at       TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (contract_address, holder_address)
);

CREATE INDEX IF NOT EXISTS idx_erc721_bal_contract ON erc721_balances (contract_address);
CREATE INDEX IF NOT EXISTS idx_erc721_bal_holder   ON erc721_balances (holder_address);

-- ─────────────────────────────────────────────────
-- ERC-721 holders
-- 状态数据，增量表，数据量大小不可预测
-- ─────────────────────────────────────────────────

CREATE TABLE IF NOT EXISTS erc721_holders (
    token_id      NUMERIC(80) NOT NULL,
    contract_address CHAR(42)   NOT NULL,
    holder_address   CHAR(42)   NOT NULL,
    url              TEXT,
    updated_at       TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (contract_address, token_id)
);

CREATE INDEX IF NOT EXISTS idx_erc721_hld_holder ON erc721_holders (holder_address);
CREATE INDEX IF NOT EXISTS idx_erc721_hld_token  ON erc721_holders (token_id);
CREATE INDEX IF NOT EXISTS idx_erc721_hld_contract  ON erc721_holders (contract_address);