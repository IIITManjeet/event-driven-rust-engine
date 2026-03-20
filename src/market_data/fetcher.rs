use async_trait::async_trait;
use super::event::MarketEvent;

#[async_trait]
pub trait MarketDataFetcher: Send + Sync {
    async fn fetch_price(&self, symbol: &str) -> Result<MarketEvent, Box<dyn std::error::Error + Send + Sync>>;
    fn exchange_name(&self) -> &str;
}