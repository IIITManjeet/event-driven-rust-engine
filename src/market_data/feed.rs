use async_trait::async_trait;
use super::event::MarketEvent;
#[async_trait]
pub trait MarketDataFeed {
    async fn next_event(&mut self) -> Option<MarketEvent>;
    fn source(&self) -> &str;
}

