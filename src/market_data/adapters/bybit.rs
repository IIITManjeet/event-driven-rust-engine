use reqwest::Client;
use serde::Deserialize;
use chrono::Utc;
use async_trait::async_trait;

use crate::market_data::{
    event::MarketEvent,
    fetcher::MarketDataFetcher,
    types::{Instrument, InstrumentType, PriceTick},
};

pub struct BybitFetcher {
    client: Client,
    base_url: String,
}

impl BybitFetcher {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
            base_url: "https://api.bybit.com/v5/market".to_string(),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct BybitResponse<T> {
    pub result: BybitResult<T>,
}

#[derive(Debug, Deserialize)]
pub struct BybitResult<T> {
    pub list: Vec<T>,
}

#[derive(Debug, Deserialize)]
pub struct BybitTickerData {
    pub symbol: String,
    #[serde(rename = "lastPrice")]
    pub last_price: String,
    pub volume24h: String,
}

#[async_trait]
impl MarketDataFetcher for BybitFetcher {
    async fn fetch_price(&self, symbol: &str) -> Result<MarketEvent, Box<dyn std::error::Error + Send + Sync>> {
        let bybit_symbol = if symbol.contains("USDT") {
            symbol.to_string()
        } else {
            format!("{}USDT", symbol)
        };

        let url = format!(
            "{}/tickers?category=spot&symbol={}",
            self.base_url, bybit_symbol
        );

        let response = self.client
            .get(&url)
            .send()
            .await?
            .json::<BybitResponse<BybitTickerData>>()
            .await?;

        let ticker = response
            .result
            .list
            .into_iter()
            .next()
            .ok_or_else(|| "No ticker data from Bybit".to_string())?;

        let price: f64 = ticker.last_price.parse()?;

        Ok(normalize_bybit_price(
            &ticker.symbol,
            price,
            Utc::now(),
        ))
    }

    fn exchange_name(&self) -> &str {
        "Bybit"
    }
}

pub fn normalize_bybit_price(
    symbol: &str,
    price: f64,
    timestamp: chrono::DateTime<Utc>,
) -> MarketEvent {
    let instrument = Instrument {
        symbol: symbol.to_string(),
        exchange: "bybit".to_string(),
        instrument_type: InstrumentType::Spot,
        base_asset: Some(symbol.replace("USDT", "")),
        quote_asset: Some("USDT".to_string()),
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