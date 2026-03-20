use chrono::{DateTime, Utc};
use crate::market_data::types::Instrument;

#[derive(Debug, Clone)]
pub enum StrategyEvent {
    Buy {
        instrument: Instrument,
        price: f64,
        timestamp: DateTime<Utc>,
    },
    Sell {
        instrument: Instrument,
        price: f64,
        timestamp: DateTime<Utc>,
    },

    Arbitrage {
        buy_exchange: String,
        sell_exchange: String,
        instrument: Instrument,
        spread: f64,
        timestamp: DateTime<Utc>,
    },
}