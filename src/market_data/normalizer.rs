use chrono::{DateTime, Utc};

use super::{
    event::MarketEvent,
    types::{Instrument, InstrumentType, PriceTick},
};

pub fn normalize_coingecko_price(
    symbol: &str,
    price: f64,
    timestamp: DateTime<Utc>,
) -> MarketEvent {
    let instrument = Instrument {
        symbol: symbol.to_string(),          // BTC, ETH
        exchange: "coingecko".to_string(),
        instrument_type: InstrumentType::Spot,
    };

    MarketEvent::Price(PriceTick {
        instrument,
        price,
        timestamp,
    })
}
