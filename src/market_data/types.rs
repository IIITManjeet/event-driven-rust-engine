use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum InstrumentType {
    Spot,
    Perpetual,
    Future,
    Option,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Instrument {
    pub symbol: String,
    pub exchange: String,
    pub instrument_type: InstrumentType,
    pub base_asset: Option<String>,
    pub quote_asset: Option<String>,
    pub expiry_date: Option<DateTime<Utc>>,
    pub strike_price: Option<f64>,
    pub option_type: Option<OptionType>,
}

impl Instrument {
    pub fn spot(symbol: String, exchange: String) -> Self {
        let parts: Vec<&str> = symbol.split('_').collect();
        let (base, quote) = if parts.len() >= 2 {
            (Some(parts[0].to_string()), Some(parts[1].to_string()))
        } else {
            (None, None)
        };
        
        Self {
            symbol,
            exchange,
            instrument_type: InstrumentType::Spot,
            base_asset: base,
            quote_asset: quote,
            expiry_date: None,
            strike_price: None,
            option_type: None,
        }
    }

    pub fn perpetual(symbol: String, exchange: String) -> Self {
        Self {
            symbol: symbol.clone(),
            exchange,
            instrument_type: InstrumentType::Perpetual,
            base_asset: Some(symbol.replace("_PERP", "")),
            quote_asset: Some("USDT".to_string()),
            expiry_date: None,
            strike_price: None,
            option_type: None,
        }
    }

    pub fn future(symbol: String, exchange: String, expiry: DateTime<Utc>) -> Self {
        Self {
            symbol: symbol.clone(),
            exchange,
            instrument_type: InstrumentType::Future,
            base_asset: Some(symbol.split('_').next().unwrap_or(&symbol).to_string()),
            quote_asset: Some("USDT".to_string()),
            expiry_date: Some(expiry),
            strike_price: None,
            option_type: None,
        }
    }

    pub fn option(symbol: String, exchange: String, strike: f64, opt_type: OptionType, expiry: DateTime<Utc>) -> Self {
        Self {
            symbol,
            exchange,
            instrument_type: InstrumentType::Option,
            base_asset: None,
            quote_asset: Some("USDT".to_string()),
            expiry_date: Some(expiry),
            strike_price: Some(strike),
            option_type: Some(opt_type),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OptionType {
    Call,
    Put,
}

#[derive(Debug, Clone)]
pub struct PriceTick {
    pub instrument: Instrument,
    pub price: f64,
    pub timestamp: DateTime<Utc>,
    pub bid: Option<f64>,
    pub ask: Option<f64>,
    pub volume: Option<f64>,
}

impl PriceTick {
    pub fn new(instrument: Instrument, price: f64, timestamp: DateTime<Utc>) -> Self {
        Self {
            instrument,
            price,
            timestamp,
            bid: None,
            ask: None,
            volume: None,
        }
    }

    pub fn with_spread(mut self, bid: f64, ask: f64) -> Self {
        self.bid = Some(bid);
        self.ask = Some(ask);
        self
    }

    pub fn with_volume(mut self, volume: f64) -> Self {
        self.volume = Some(volume);
        self
    }

    pub fn spread(&self) -> Option<f64> {
        match (self.bid, self.ask) {
            (Some(bid), Some(ask)) => Some(ask - bid),
            _ => None,
        }
    }

    pub fn mid_price(&self) -> f64 {
        match (self.bid, self.ask) {
            (Some(bid), Some(ask)) => (bid + ask) / 2.0,
            _ => self.price,
        }
    }
}

#[derive(Debug, Clone)]
pub struct TradeTick {
    pub instrument: Instrument,
    pub price: f64,
    pub quantity: f64,
    pub side: TradeSide,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TradeSide {
    Buy,
    Sell,
}