use async_trait::async_trait;
use crate::market_data::event::MarketEvent;

pub mod event;
pub mod simple;
pub mod cross_exchange;

pub use event::StrategyEvent;
pub use simple::SimpleStrategy;
pub use cross_exchange::arbitrage::ArbitrageStrategy;

#[async_trait]
pub trait Strategy: Send {
    async fn on_market_event(
        &mut self,
        event: MarketEvent,
    ) -> Option<StrategyEvent>;

    fn name(&self) -> &str;
}