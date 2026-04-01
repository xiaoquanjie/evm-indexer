use alloy::providers::Provider;
use serde::de::DeserializeOwned;
use std::cmp::min;
use std::future::Future;
use std::sync::Arc;

pub trait Sub<T> {
    async fn recv(&mut self) -> anyhow::Result<Option<T>>;

    fn block_number_from_data(data: &T) -> u64;
}

pub struct Subscription<O, T> {
    pub(super) opt: O,
    pub(super) buf: Option<Box<dyn Iterator<Item = T>>>,
    pub(super) sub: Option<alloy::pubsub::Subscription<T>>,
}

impl<O, T> Subscription<O, T> {
    pub fn new(opt: O) -> Self {
        Subscription {
            opt,
            buf: None,
            sub: None,
        }
    }

    pub fn into_stream(self) -> impl futures::stream::Stream<Item = anyhow::Result<T>>
    where
        Self: Sub<T>,
    {
        futures::stream::unfold(self, |mut sub| async {
            match sub.recv().await {
                Ok(None) => None,
                Ok(Some(item)) => Some((Ok(item), sub)),
                Err(e) => Some((Err(anyhow::Error::from(e)), sub)),
            }
        })
    }

    pub(super) fn next_buf(&mut self) -> Option<T> {
        if let Some(mut iter) = self.buf.take() {
            let d = iter.next()?;
            self.buf = Some(iter);
            Some(d)
        } else {
            None
        }
    }

    pub(super) async fn next_sub(&mut self) -> anyhow::Result<bool>
    where
        T: DeserializeOwned + 'static,
    {
        if let Some(mut sub) = self.sub.take() {
            let d = sub.recv().await?;
            self.buf = Some(Box::new([d].into_iter()));
            self.sub = Some(sub);
            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub(super) async fn inner_recv<S, SF, B, BF>(
        &mut self,
        sub: S,
        buf: B,
    ) -> anyhow::Result<Option<T>>
    where
        T: DeserializeOwned + 'static,
        S: Fn(u64, u64) -> SF,
        SF: Future<Output = anyhow::Result<(alloy::pubsub::Subscription<T>, T)>>,
        B: Fn(u64, u64) -> BF,
        BF: Future<Output = anyhow::Result<Box<dyn Iterator<Item = T>>>>,
        Self: Sub<T>,
        O: WithSubOption,
    {
        // 区块区间
        let range = self.opt.sub_option().block_range;

        // 循环获取
        loop {
            let (start, end) = self.opt.init_block_range().await?;
            if start > end {
                return Ok(None);
            }

            // 读取缓存
            if let Some(d) = self.next_buf() {
                return Ok(Some(d));
            }

            // 有了订阅就继续读取订阅
            let has = self.next_sub().await?;
            if has {
                continue;
            }

            // 取当前得最高位置
            let height = self
                .opt
                .sub_option_mut()
                .provider
                .get_block_number()
                .await?;

            // 当前位置距离最高位进入可接受范围
            if start > height + 1 {
                panic!("something wrong")
            } else if start == height + 1 {
                // 开始创建订阅
                let (sub, d) = sub(start, end).await?;
                let num = Self::block_number_from_data(&d);

                if num <= start + 1 {
                    // 订阅后的数据追上了
                    self.buf = Some(Box::new([d].into_iter()));
                    self.sub = Some(sub);
                    self.update_start_block(num+1);
                } else {
                    let dist = num - start - 1;
                    let to = min(start + min(dist, range), end);
                    let mut buf = buf(start, to).await?;
                    if dist <= range {
                        // 将h插入buf后面
                        buf = Box::new(buf.chain(Some(d)));
                        // 距离不远可以保留订阅，否则就要丢弃掉
                        self.sub = Some(sub);
                    }
                    self.buf = Some(buf);
                    self.update_start_block(to+1);
                }
            } else {
                // 差值过大，继续读取历史数据
                let to = min(min(start + range, height), end);
                self.buf = Some(buf(start, to).await?);
                self.update_start_block(to+1);
            }
        }
    }

    /// 更新起始区块
    fn update_start_block(&mut self, block: u64)
    where
        O: WithSubOption,
    {
        self.opt.sub_option_mut().start_block.as_mut().map(|v| {
            if block > *v {
                *v = block;
            }
        });
    }
}

pub struct SubOption {
    pub(super) provider: Arc<dyn Provider>,
    pub(super) start_block: Option<u64>,
    pub(super) end_block: Option<u64>,
    /// 一次性获取区块的范围有多大，对于事件来讲这个范围数受rpc的影响
    pub(super) block_range: u64,
}

impl SubOption {
    pub fn new(provider: Arc<dyn Provider>) -> Self {
        Self {
            provider,
            start_block: None,
            end_block: None,
            block_range: 20,
        }
    }
}

pub trait WithSubOption: Sized {
    fn sub_option_mut(&mut self) -> &mut SubOption;

    fn sub_option(&self) -> &SubOption;

    fn provider(mut self, provider: Arc<dyn Provider>) -> Self {
        self.sub_option_mut().provider = provider;
        self
    }

    /// 设置为none表示从最新的开始
    fn start_block(mut self, start: Option<u64>) -> Self {
        self.sub_option_mut().start_block = start;
        self
    }

    /// 设置为none表示没有结束
    fn end_block(mut self, end: Option<u64>) -> Self {
        self.sub_option_mut().end_block = end;
        self
    }

    fn block_range(mut self, block_range: u64) -> Self {
        self.sub_option_mut().block_range = block_range;
        self
    }
}

trait WithSubOptionExt: WithSubOption {
    async fn init_block_range(&mut self) -> anyhow::Result<(u64, u64)> {
        let opt = self.sub_option_mut();
        let start = match opt.start_block {
            None => {
                let n = opt.provider.get_block_number().await?;
                opt.start_block = Some(n);
                n as u64
            }
            Some(s) => s,
        };
        let end = match opt.end_block {
            None => {
                opt.end_block = Some(u64::MAX);
                u64::MAX
            }
            Some(e) => e,
        };

        Ok((start, end))
    }
}

impl<T> WithSubOptionExt for T where T: WithSubOption {}
