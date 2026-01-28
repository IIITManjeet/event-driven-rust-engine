use chrono::{DateTime, Utc};

#[derive(Debug, Clone)]
pub enum InstrumentType {
    Spot,
    Perpetual,
    Future,
    Option,
}

#[derive(Debug, Clone)]
pub struct Instrument {
    pub symbol: String,          // BTCUSDT, ETH-USD-PERP
    pub exchange: String,        // binance, bybit, coingecko
    pub instrument_type: InstrumentType,
}

#[derive(Debug, Clone)]
pub struct PriceTick {
    pub instrument: Instrument,
    pub price: f64,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct TradeTick {
    pub instrument: Instrument,
    pub price: f64,
    pub quantity: f64,
    pub side: TradeSide,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub enum TradeSide {
    Buy,
    Sell,
}