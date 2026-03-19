use crate::models;
use alloy::primitives::{Address, B256, U256};
use bigdecimal::BigDecimal;
use num_bigint::BigUint;
use num_traits::Num;

/// erc20和erc721通用
/// keccak256("Transfer(address,address,uint256)")
pub const TRANSFER_TOPIC: &str =
    "0xddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef";

/// A decoded Transfer event (works for both ERC-20 and ERC-721)
#[derive(Debug)]
pub struct TransferEvent {
    pub from: String,
    pub to: String,
    /// For ERC-20: token amount; for ERC-721: token ID
    pub value: BigDecimal,
    pub is_nft: bool,
}

pub fn decode_transfer(topics: &[B256], data: &[u8]) -> Option<TransferEvent> {
    // Must match Transfer(address,address,uint256) signature
    if topics.is_empty() {
        return None;
    }
    let sig = format!("0x{}", hex::encode(topics[0].as_slice()));
    if sig.to_lowercase() != TRANSFER_TOPIC.to_lowercase() {
        return None;
    }

    match topics.len() {
        // ERC-20: topics = [sig, from, to],  data = amount
        3 => {
            let from = address_from_topic(&topics[1])?;
            let to = address_from_topic(&topics[2])?;
            let amount = uint256_from_bytes(data)?;
            Some(TransferEvent {
                from,
                to,
                value: amount,
                is_nft: false,
            })
        }
        // ERC-721: topics = [sig, from, to, tokenId]
        4 => {
            let from = address_from_topic(&topics[1])?;
            let to = address_from_topic(&topics[2])?;
            let token_id = uint256_from_topic(&topics[3])?;
            Some(TransferEvent {
                from,
                to,
                value: token_id,
                is_nft: true,
            })
        }
        _ => None,
    }
}

pub fn decode_transaction_log(log: &models::TransactionLog) -> Option<TransferEvent> {
    if log.topic3.is_none() {
        Some(TransferEvent {
            from: topic_to_address(&log.topic1),
            to: topic_to_address(&log.topic2),
            value: hex_to_bigdecimal(log.data.as_ref().unwrap()),
            is_nft: false,
        })
    } else {
        Some(TransferEvent {
            from: topic_to_address(&log.topic1),
            to: topic_to_address(&log.topic2),
            value: hex_to_bigdecimal(log.topic3.as_ref().unwrap()),
            is_nft: true,
        })
    }
}

fn address_from_topic(topic: &B256) -> Option<String> {
    // The last 20 bytes of a 32-byte topic are the address
    let bytes = topic.as_slice();
    if bytes.len() < 32 {
        return None;
    }
    let addr = Address::from_slice(&bytes[12..32]);
    Some(format!("{:#x}", addr))
}

fn uint256_from_bytes(data: &[u8]) -> Option<BigDecimal> {
    if data.len() < 32 {
        // Pad with leading zeros if short
        let mut padded = vec![0u8; 32];
        if !data.is_empty() {
            padded[32 - data.len()..].copy_from_slice(data);
        }
        let n = BigUint::from_bytes_be(&padded);
        return Some(BigDecimal::from(bigdecimal::BigDecimal::from(
            num_bigint::BigInt::from(n),
        )));
    }
    let n = BigUint::from_bytes_be(&data[..32]);
    Some(bigdecimal_from_biguint(n))
}

fn uint256_from_topic(topic: &B256) -> Option<BigDecimal> {
    uint256_from_bytes(topic.as_slice())
}

fn bigdecimal_from_biguint(n: BigUint) -> BigDecimal {
    let hex_str = format!("{:x}", n);
    let bi = num_bigint::BigInt::from_str_radix(&hex_str, 16).unwrap_or_default();
    BigDecimal::from(bi)
}

/// Convert a U256 to BigDecimal
pub fn u256_to_bigdecimal(v: U256) -> BigDecimal {
    let hex = format!("{:x}", v);
    if hex.is_empty() || hex == "0" {
        return BigDecimal::from(0);
    }
    let bi = num_bigint::BigInt::from_str_radix(&hex, 16).unwrap_or_default();
    BigDecimal::from(bi)
}

pub fn hex_to_bigdecimal(h: &str) -> BigDecimal {
    let hex_without_prefix = h.trim_start_matches("0x");
    let bi = num_bigint::BigInt::from_str_radix(&hex_without_prefix, 16).unwrap_or_default();
    BigDecimal::from(bi)
}

fn topic_to_address(topic: &Option<String>) -> String {
    topic
        .as_ref()
        .map(|t| truncate_hex_to_address(t).unwrap_or_default())
        .unwrap_or_default()
}

fn truncate_hex_to_address(h: &str) -> Result<String, HexError> {
    // 1. 检查前缀
    if !h.starts_with("0x") {
        return Err(HexError::InvalidPrefix);
    }

    // 2. 检查长度 (0x + 64 个十六进制字符 = 66)
    if h.len() != 66 {
        return Err(HexError::InvalidLength);
    }

    // 3. 截断：保留后 40 个字符
    Ok(format!("0x{}", &h[26..])) // 跳过前 26 个字符，保留后 40 个
}

#[derive(Debug, PartialEq)]
enum HexError {
    InvalidPrefix,
    InvalidLength,
}
