use crate::scanner::subscription::{SubOption, WithSubOption};
use alloy::eips::BlockNumberOrTag;
use alloy::providers::Provider;
use alloy::rpc::types::Header;
use anyhow::anyhow;
use futures::StreamExt;
use std::sync::Arc;
use alloy::network::primitives::BlockTransactionsKind;

/// 区块订阅
pub struct LiveScanner {
    provider: Arc<dyn Provider>,
}

/// 优化：
/// 内部各种参数可以做调整以求最佳实践
/// Provider默认不会自动断线重连
/// 也增加一个http provider
impl LiveScanner {
    pub fn new(provider: Arc<dyn Provider>) -> Self {
        Self { provider }
    }

    pub fn subscribe(&self) -> BlockOption {
        BlockOption {
            inner: SubOption::new(self.provider.clone()),
            concurrent: 10,
        }
    }
}

pub struct BlockOption {
    inner: SubOption,
    concurrent: usize,
}

impl WithSubOption for BlockOption {
    fn sub_option_mut(&mut self) -> &mut SubOption {
        &mut self.inner
    }

    fn sub_option(&self) -> &SubOption {
        &self.inner
    }
}

impl BlockOption {
    /// 订阅历史区块时允许的获取并发数，默认值是10
    #[allow(unused)]
    pub fn concurrent(self, n: usize) -> Self {
        Self {
            concurrent: n,
            ..self
        }
    }

    pub fn build(self) -> super::subscription::Subscription<Self, Header> {
        super::subscription::Subscription::new(self)
    }
}

impl super::subscription::Sub<Header> for super::subscription::Subscription<BlockOption, Header> {
    async fn recv(&mut self) -> anyhow::Result<Option<Header>> {
        let provider = self.opt.inner.provider.clone();
        let sub = move |_start, _to| {
            let provider = provider.clone();
            async move {
                let mut sub = provider.subscribe_blocks().await?;
                // 获取订阅后的第一个
                let h = sub.recv().await?;
                Ok((sub, h))
            }
        };

        let provider = self.opt.inner.provider.clone();
        let concurrent = self.opt.concurrent;
        let buf = move |start, to| {
            let provider = provider.clone();
            async move {
                let o = move |id: u64| {
                    let provider = provider.clone();
                    async move {
                        let r = provider
                            .get_block_by_number(BlockNumberOrTag::Number(id))
                            .kind(BlockTransactionsKind::Hashes)
                            .await;
                        let r = r?;
                        r.map(|b| b.header).ok_or(anyhow!("block {id} not found"))
                    }
                };

                let v = futures::stream::iter(start..=to)
                    .map(o)
                    .buffered(concurrent)
                    .collect::<Vec<_>>()
                    .await;

                // 只要有一个是错误的就认为出错了
                let v = v.into_iter().collect::<Result<Vec<_>, _>>()?;
                let b: Box<dyn Iterator<Item = Header>> = Box::new(v.into_iter());
                let o: anyhow::Result<Box<dyn Iterator<Item = Header>>> = Ok(b);
                o
            }
        };

        self.inner_recv(sub, buf).await
    }

    fn block_number_from_data(data: &Header) -> u64 {
        data.number
    }
}
