# EVM Indexer

一个用 Rust 编写的以太坊（及 EVM 兼容链）区块链索引器，类似 Etherscan 的数据采集后端。

## 功能特性

- **历史扫描**：从指定区块号开始扫描，支持并发批量拉取
- **断点续传**：程序重启后自动从上次处理的区块继续
- **实时订阅**：历史扫描完成后通过 WebSocket 订阅新区块（未实现）
- **自动补块**：实时模式下断线重连后自动填补遗漏区块

## 数据库索引能力

| 查询需求                | 对应表 | 索引字段 |
|---------------------|--------|---------|
| 通过区块号查完整区块          | `blocks` | `number` (PK) |
| 通过区块 Hash 查区块       | `blocks` | `hash` (UNIQUE) |
| 通过交易 Hash 查交易       | `transactions` | `hash` (PK) |
| 通过交易 Hash 查日志       | `transaction_logs` | `transaction_hash` |
| 通过账户地址查所有交易         | `transactions` | `from_address`, `to_address` |
| 通过账户地址查 ETH 变化（未实现） | `eth_transfers` | `from_address`, `to_address` |
| 通过合约地址查 ERC-20 变化   | `erc20_transfers` | `contract_address` |
| 查询 ERC-20 账户余额      | `erc20_balances` | `(contract_address, holder_address)` |
| 通过合约地址查 ERC-721 变化  | `erc721_transfers` | `contract_address` |
| 查询 ERC-721 账户持有数量   | `erc721_balances` | `(contract_address, holder_address)` |

## 技术栈

- **Rust** — 系统语言
- **alloy 1.7.3** — 以太坊 RPC 客户端
- **tokio 1.50.0** — 异步运行时
- **sqlx 0.8** — 异步 PostgreSQL 驱动
- **PostgreSQL 16** — 数据存储

## 快速开始

### 1. 配置

复制配置模板并填写你的 RPC 节点地址：

`config.toml` 关键字段说明：

```toml
[rpc]
http_url = "https://mainnet.infura.io/v3/YOUR_KEY"   # 历史扫描用 HTTP
ws_url   = "wss://mainnet.infura.io/ws/v3/YOUR_KEY"  # 实时订阅用 WS

[database]
url = "postgresql://postgres:password@localhost:5432/evm_indexer"

[indexer]
start_block      = 0    # 从第几个区块开始扫
concurrent_blocks = 10  # 并发拉取区块数
confirmations    = 12   # 确认数（防重组）
```

也可以通过环境变量覆盖（优先级高于文件）：

```bash
export EVM_INDEXER__RPC__HTTP_URL=https://...
export EVM_INDEXER__INDEXER__START_BLOCK=19000000
```

### 3. 编译运行

```bash
# Debug 模式（开发调试）
cargo run

# Release 模式（生产部署）
cargo build --release
./target/release/evm-indexer
```

### 4. 日志级别

通过环境变量控制日志详细程度：

```bash
RUST_LOG=evm_indexer=debug cargo run   # 详细（含每笔 Transfer 解码）
RUST_LOG=evm_indexer=info  cargo run   # 标准（每个区块一行日志）
RUST_LOG=evm_indexer=warn  cargo run   # 仅警告和错误
```

## 常用查询示例

连接数据库后可直接运行：

```sql
-- 查询区块信息（按区块号）
SELECT * FROM blocks WHERE number = 19000000;

-- 查询区块信息（按 Hash）
SELECT * FROM blocks WHERE hash = '0xabc...';

-- 查询交易信息
SELECT * FROM transactions WHERE hash = '0xdef...';

-- 查询交易的所有日志
SELECT * FROM transaction_logs WHERE transaction_hash = '0xdef...' ORDER BY log_index;

-- 查询某地址所有交易
SELECT * FROM transactions
WHERE from_address = '0x...' OR to_address = '0x...'
ORDER BY block_number DESC;

-- 查询某地址所有 ETH 变化记录
SELECT * FROM eth_transfers
WHERE from_address = '0x...' OR to_address = '0x...'
ORDER BY block_number DESC;

-- 查询某 ERC-20 合约所有 Transfer 记录
SELECT * FROM erc20_transfers WHERE contract_address = '0x...' ORDER BY block_number DESC;

-- 查询某账户持有某 ERC-20 合约的余额
SELECT balance FROM erc20_balances
WHERE contract_address = '0x...' AND holder_address = '0x...';

-- 查询某 ERC-721 合约所有 NFT 转移记录
SELECT * FROM erc721_transfers WHERE contract_address = '0x...' ORDER BY block_number DESC;

-- 查询某账户持有某 ERC-721 合约 NFT 数量
SELECT token_count FROM erc721_balances
WHERE contract_address = '0x...' AND holder_address = '0x...';
```

## 断点续传机制

程序将当前已处理的最大区块号存储在 `sync_state` 表中：

```sql
SELECT * FROM sync_state;
--  id | last_block |         updated_at
-- ----+------------+----------------------------
--   1 |   19000123 | 2024-01-15 08:32:11+00
```

重启后程序自动读取此值，从 `last_block + 1` 继续处理，无需手动干预。

