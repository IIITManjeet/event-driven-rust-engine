use reqwest::Client;
use serde::Deserialize;
use chrono::Utc;
use async_trait::async_trait;

use crate::market_data::{
    event::MarketEvent,
    fetcher::MarketDataFetcher,
    types::{Instrument, InstrumentType, PriceTick},
};

pub struct BinanceFetcher {
    client: Client,
    base_url: String,
}

impl BinanceFetcher {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
            base_url: "https://api.binance.com/api/v3".to_string(),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct BinanceTickerResponse {
    pub symbol: String,
    #[serde(rename = "lastPrice")]
    pub last_price: String,
    pub volume: String,
}

#[async_trait]
impl MarketDataFetcher for BinanceFetcher {
    async fn fetch_price(&self, symbol: &str) -> Result<MarketEvent, Box<dyn std::error::Error + Send + Sync>> {
        let url = format!("{}/ticker/24hr?symbol={}", self.base_url, symbol);

        let response = self.client
            .get(&url)
            .send()
            .await?
            .json::<BinanceTickerResponse>()
            .await?;

        let price: f64 = response.last_price.parse()?;

        Ok(normalize_binance_price(
            &response.symbol,
            price,
            Utc::now(),
        ))
    }

    fn exchange_name(&self) -> &str {
        "Binance"
    }
}

pub fn normalize_binance_price(
    symbol: &str,
    price: f64,
    timestamp: chrono::DateTime<Utc>,
) -> MarketEvent {
    let instrument = Instrument {
        symbol: symbol.to_string(),
        exchange: "binance".to_string(),
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