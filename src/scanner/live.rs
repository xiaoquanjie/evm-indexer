use std::sync::Arc;
use alloy::primitives::Address;
use crate::config::AppConfig;
use crate::db::repo::Repository;
use crate::rpc::Rpc;

/// 数据订阅
/// 支持区块订阅
/// 支持账户地址订阅
/// 支持日志订阅
pub struct LiveScanner {
    config: AppConfig,
    repo: Arc<Repository>,
    rpc: Arc<Rpc>
}

impl LiveScanner {
    pub fn new(config: AppConfig, repo: Arc<Repository>, rpc: Arc<Rpc>) -> Self {
        Self { config, repo, rpc }
    }
}

pub enum SubOption {
    /// 从哪个区块开始订阅
    Block(u64),
    /// 从哪个区块开始订阅函数
    Call(u64, Address, String),
    /// 从哪个区块开始订阅哪个地址
    Address(u64, Address),
    /// 从哪个区块开始订阅哪个事件
    Event(u64, String),
}