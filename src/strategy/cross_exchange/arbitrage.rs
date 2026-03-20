use async_trait::async_trait;
use chrono::Utc;
use std::collections::HashMap;

use crate::{
    market_data::event::MarketEvent,
    strategy::{Strategy, event::StrategyEvent},
};

pub struct ArbitrageStrategy {
    threshold: f64,
    prices: HashMap<String, f64>, // exchange -> price
}

impl ArbitrageStrategy {
    pub fn new(threshold: f64) -> Self {
        Self {
            threshold,
            prices: HashMap::new(),
        }
    }
}
#[async_trait]
impl Strategy for ArbitrageStrategy {
    async fn on_market_event(
        &mut self,
        event: MarketEvent,
    ) -> Option<StrategyEvent> {
        let price_tick = match event {
            MarketEvent::Price(p) => p,
            _ => return None,
        };

        self.prices.insert(
            price_tick.instrument.exchange.clone(),
            price_tick.price,
        );

        if self.prices.len() < 2 {
            return None;
        }

        let mut iter = self.prices.iter();
        let (ex1, p1) = iter.next()?;
        let (ex2, p2) = iter.next()?;

        let spread = (p1 - p2).abs();

        if spread >= self.threshold {
            let (buy_ex, sell_ex) = if p1 < p2 {
                (ex1, ex2)
            } else {
                (ex2, ex1)
            };

            return Some(StrategyEvent::Arbitrage {
                buy_exchange: buy_ex.clone(),
                sell_exchange: sell_ex.clone(),
                instrument: price_tick.instrument.clone(),
                spread,
                timestamp: Utc::now(),
            });
        }

        None
    }

    fn name(&self) -> &str {
        "Cross-Exchange Arbitrage"
    }
}
