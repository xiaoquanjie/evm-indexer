use alloy::eips::BlockNumberOrTag;
use alloy::network::primitives::BlockTransactionsKind;
use alloy::primitives::Address;
use alloy::providers::Provider;
use alloy::pubsub::Subscription;
use alloy::rpc::types::Header;
use anyhow::anyhow;
use futures::StreamExt;
use std::cmp::min;
use std::sync::Arc;

/// 数据订阅
/// 支持区块订阅
/// 支持账户地址订阅
/// 支持日志订阅
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

    pub fn subscribe_blocks(&self) -> BlockSubscribeOption {
        BlockSubscribeOption {
            provider: self.provider.clone(),
            start_block_id: None,
            end_block_id: None,
            concurrent: 10,
        }
    }
}

pub struct BlockSubscribeOption {
    provider: Arc<dyn Provider>,
    start_block_id: Option<u64>,
    end_block_id: Option<u64>,
    concurrent: usize,
}

impl BlockSubscribeOption {
    #[allow(unused)]
    pub fn provider(self, provider: Arc<dyn Provider>) -> Self {
        Self { provider, ..self }
    }

    /// 设置为none表示从最新的开始
    pub fn start_block_id(self, start: Option<u64>) -> Self {
        Self {
            start_block_id: start,
            ..self
        }
    }

    /// 设置为none表示没有结束
    pub fn end_block_id(self, end: Option<u64>) -> Self {
        Self {
            end_block_id: end,
            ..self
        }
    }

    /// 订阅历史区块时允许的获取并发数，默认值是10
    #[allow(unused)]
    pub fn concurrent(self, n: usize) -> Self {
        Self {
            concurrent: n,
            ..self
        }
    }

    pub fn build(self) -> BlockSubscribe {
        BlockSubscribe {
            opt: self,
            buf: None,
            sub: None,
        }
    }
}

pub struct BlockSubscribe {
    opt: BlockSubscribeOption,
    buf: Option<Box<dyn Iterator<Item = Header> + Unpin>>,
    sub: Option<Subscription<Header>>,
}

impl BlockSubscribe {
    /// 当返回错误后要弃用这个实例，重新创建一个新的BlockSubscribe
    /// 可能会收到重复的区块头，这可以用来判断是否发生了区块重组
    pub async fn recv(&mut self) -> anyhow::Result<Option<Header>> {
        let start = match self.opt.start_block_id {
            None => {
                let n = self.opt.provider.get_block_number().await?;
                self.opt.start_block_id = Some(n);
                n as u64
            }
            Some(s) => s,
        };
        let end = match self.opt.end_block_id {
            None => {
                self.opt.end_block_id = Some(u64::MAX);
                u64::MAX
            }
            Some(e) => e,
        };

        // 一次取得范围
        const RANGE: u64 = 20;

        loop {
            if start > end {
                return Ok(None);
            }

            if let Some(mut iter) = self.buf.take() {
                let d = iter.next();
                if let Some(d) = d {
                    self.buf = Some(iter);
                    self.opt.start_block_id.as_mut().map(|v| *v += 1);
                    return Ok(Some(d));
                }
            }

            // 有了订阅就继续读取订阅
            if let Some(mut sub) = self.sub.take() {
                let h = sub.recv().await?;
                self.buf = Some(Box::new([h].into_iter()));
                self.sub = Some(sub);
                continue;
            }

            // 还没开始订阅
            // 取当前得最高位置
            let height = self.opt.provider.get_block_number().await?;
            println!("height:{height}");
            // 当前位置距离最高位进入可接受范围
            if start + RANGE >= height {
                // 开始创建订阅
                let mut sub = self.start_subscribe().await?;

                // 获取订阅后的第一个
                let h = sub.recv().await?;

                if h.number <= start {
                    self.buf = Some(Box::new([h].into_iter()));
                    self.sub = Some(sub);
                } else {
                    let dist = h.number - start - 1;
                    let to = min(start + min(dist, RANGE), end);
                    let mut buf = self
                        .get_blocks_range(start, to, self.opt.concurrent)
                        .await?;
                    if dist <= 2 {
                        // 将h插入buf后面
                        buf = Box::new(buf.chain(Some(h)));
                        // 距离不远可以保留订阅，否则就要丢弃掉
                        self.sub = Some(sub);
                    }
                    self.buf = Some(buf);
                }
            } else {
                // 差值过大，继续读取历史数据
                let to = min(min(start + RANGE, height), end);
                let buf = self
                    .get_blocks_range(start, to, self.opt.concurrent)
                    .await?;
                self.buf = Some(buf);
            }
        }
    }

    async fn get_blocks_range(
        &self,
        start: u64,
        end: u64,
        concurrent: usize,
    ) -> anyhow::Result<Box<dyn Iterator<Item = Header> + Unpin>> {
        let provider = self.opt.provider.clone();
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

        let v = futures::stream::iter(start..=end)
            .map(o)
            .buffered(concurrent)
            .collect::<Vec<_>>()
            .await;

        // 只要有一个是错误的就认为出错了
        let v = v.into_iter().collect::<Result<Vec<_>, _>>()?;
        Ok(Box::new(v.into_iter()))
    }

    async fn start_subscribe(
        &self,
    ) -> anyhow::Result<Subscription<Header>> {
        Ok(self.opt.provider.subscribe_blocks().await?)
    }
}

#[allow(unused)]
pub enum SubOption {
    /// 从哪个区块开始订阅函数
    Call(u64, Address, String),
    /// 从哪个区块开始订阅哪个地址
    Address(u64, Address),
    /// 从哪个区块开始订阅哪个事件
    Event(u64, String),
}
