use async_trait::async_trait;
use crate::market_data::event::MarketEvent;
use crate::strategy::event::StrategyEvent;

#[async_trait]
pub trait Strategy: Send {
    async fn on_market_event(
        &mut self,
        event: MarketEvent,
    ) -> Option<StrategyEvent>;

    fn name(&self) -> &str;
}
