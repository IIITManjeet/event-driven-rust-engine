use async_trait::async_trait;
use crate::market_data::event::MarketEvent;
use crate::strategy::event::StrategyEvent;
use crate::strategy::Strategy;

pub struct SimpleStrategy {
    base_prices: std::collections::HashMap<String, f64>,
    threshold: f64,
}

impl SimpleStrategy {
    pub fn new(threshold: f64) -> Self {
        Self {
            base_prices: std::collections::HashMap::new(),
            threshold,
        }
    }
}

#[async_trait]
impl Strategy for SimpleStrategy {
    async fn on_market_event(
        &mut self,
        event: MarketEvent,
    ) -> Option<StrategyEvent> {
        if let MarketEvent::Price(price_tick) = event {
            let symbol = price_tick.instrument.symbol.clone();
            let current_price = price_tick.price;
            let timestamp = price_tick.timestamp;

            if let Some(&base_price) = self.base_prices.get(&symbol) {
                let change_pct = (current_price - base_price) / base_price * 100.0;
                println!("   📈 {} price change: {:.4}% (threshold: {}%)", symbol, change_pct, self.threshold);

                if change_pct > self.threshold {
                    return Some(StrategyEvent::Buy {
                        instrument: price_tick.instrument,
                        price: current_price,
                        timestamp,
                    });
                } else if change_pct < -self.threshold {
                    return Some(StrategyEvent::Sell {
                        instrument: price_tick.instrument,
                        price: current_price,
                        timestamp,
                    });
                }
            }

            self.base_prices.insert(symbol, current_price);
        }
        None
    }

    fn name(&self) -> &str {
        "SimpleStrategy"
    }
}