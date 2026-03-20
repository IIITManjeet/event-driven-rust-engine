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
        symbol: symbol.to_string(),
        exchange: "coingecko".to_string(),
        instrument_type: InstrumentType::Spot,
        base_asset: Some(symbol.to_string()),
        quote_asset: Some("USD".to_string()),
        expiry_date: None,
        strike_price: None,
        option_type: None,
    };

    MarketEvent::Price(PriceTick {
        instrument,
        price,
        timestamp,
        bid: None,
        ask: None,
        volume: None,
    })
}