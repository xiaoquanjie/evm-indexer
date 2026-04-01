use crate::scanner::subscription::{SubOption, WithSubOption};
use alloy::primitives::{keccak256, Address};
use alloy::providers::Provider;
use alloy::rpc::types::{Filter, Log, Topic};
use std::sync::Arc;

pub struct EventScanner {
    provider: Arc<dyn Provider>,
}

impl EventScanner {
    pub fn new(provider: Arc<dyn Provider>) -> Self {
        Self { provider }
    }

    pub fn subscribe(&self) -> EventOption {
        let mut o = EventOption {
            inner: SubOption::new(self.provider.clone()),
            address: None,
            topics: Default::default(),
        };
        o.inner.block_range = 200;
        o
    }
}

pub struct EventOption {
    inner: SubOption,
    address: Option<Address>,
    topics: [Topic; 4],
}

impl WithSubOption for EventOption {
    fn sub_option_mut(&mut self) -> &mut SubOption {
        &mut self.inner
    }

    fn sub_option(&self) -> &SubOption {
        &self.inner
    }
}

impl EventOption {
    pub fn address<T: Into<Address>>(mut self, address: T) -> Self {
        self.address = Some(address.into());
        self
    }

    pub fn topics(mut self, topics: [Topic; 4]) -> Self {
        self.topics = topics;
        self
    }

    pub fn event(mut self, event_name: &str) -> Self {
        let hash = keccak256(event_name.as_bytes());
        self.topics[0] = hash.into();
        self
    }

    /// 多个event
    pub fn events(mut self, events: impl IntoIterator<Item = impl AsRef<[u8]>>) -> Self {
        let events = events.into_iter().map(|e| keccak256(e.as_ref())).collect::<Vec<_>>();
        self.topics[0] = events.into();
        self
    }

    pub fn event_signature<T: Into<Topic>>(mut self, topic: T) -> Self {
        self.topics[0] = topic.into();
        self
    }

    pub fn topic1<T: Into<Topic>>(mut self, topic: T) -> Self {
        self.topics[1] = topic.into();
        self
    }

    pub fn topic2<T: Into<Topic>>(mut self, topic: T) -> Self {
        self.topics[2] = topic.into();
        self
    }

    pub fn topic3<T: Into<Topic>>(mut self, topic: T) -> Self {
        self.topics[3] = topic.into();
        self
    }
    pub fn build(self) -> super::subscription::Subscription<Self, Log> {
        super::subscription::Subscription::new(self)
    }
}

impl super::subscription::Sub<Log> for super::subscription::Subscription<EventOption, Log> {
    async fn recv(&mut self) -> anyhow::Result<Option<Log>> {
        let provider = self.opt.inner.provider.clone();
        let address = self.opt.address.clone();
        let topics = self.opt.topics.clone();
        let sub = move |start, end| {
            let provider = provider.clone();
            let address = address.clone();
            let topics = topics.clone();
            async move {
                let mut filter = if end == u64::MAX {
                    Filter::new().select(start..)
                } else {
                    Filter::new().select(start..=end)
                };
                if let Some(a) = address {
                    filter = filter.address(a);
                }
                filter.topics = topics;
                let mut sub = provider.subscribe_logs(&filter).await?;
                // 获取订阅后的第一个
                let l = sub.recv().await?;
                Ok((sub, l))
            }
        };

        let provider = self.opt.inner.provider.clone();
        let address = self.opt.address.clone();
        let topics = self.opt.topics.clone();
        let buf = move |start, end| {
            let provider = provider.clone();
            let address = address.clone();
            let topics = topics.clone();
            async move {
                let mut filter = Filter::new().select(start..=end);
                if let Some(a) = address {
                    filter = filter.address(a);
                }

                filter.topics = topics;
                let logs = provider.get_logs(&filter).await?;
                let b: Box<dyn Iterator<Item = Log>> = Box::new(logs.into_iter());
                let o: anyhow::Result<Box<dyn Iterator<Item = Log>>> = Ok(b);
                o
            }
        };

        self.inner_recv(sub, buf).await
    }

    fn block_number_from_data(data: &Log) -> u64 {
        data.block_number.unwrap()
    }

    fn block(&self) -> Option<u64> {
        self.opt.inner.start_block
    }
}
