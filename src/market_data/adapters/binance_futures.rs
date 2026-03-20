use reqwest::Client;
use serde::Deserialize;
use chrono::Utc;
use async_trait::async_trait;

use crate::market_data::{
    event::MarketEvent,
    fetcher::MarketDataFetcher,
    types::{Instrument, InstrumentType, PriceTick},
};

pub struct BinanceFuturesFetcher {
    client: Client,
    base_url: String,
}

impl BinanceFuturesFetcher {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
            base_url: "https://fapi.binance.com/fapi/v1".to_string(),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct BinanceFuturesTickerResponse {
    pub symbol: String,
    #[serde(rename = "lastPrice")]
    pub last_price: String,
    pub volume: String,
    #[serde(rename = "bidPrice")]
    pub bid_price: Option<String>,
    #[serde(rename = "askPrice")]
    pub ask_price: Option<String>,
}

#[async_trait]
impl MarketDataFetcher for BinanceFuturesFetcher {
    async fn fetch_price(&self, symbol: &str) -> Result<MarketEvent, Box<dyn std::error::Error + Send + Sync>> {
        let url = format!("{}/ticker/24hr?symbol={}", self.base_url, symbol);

        let response = self.client
            .get(&url)
            .send()
            .await?
            .json::<BinanceFuturesTickerResponse>()
            .await?;

        let price: f64 = response.last_price.parse()?;
        let bid: Option<f64> = response.bid_price.and_then(|b| b.parse().ok());
        let ask: Option<f64> = response.ask_price.and_then(|a| a.parse().ok());
        let volume: Option<f64> = response.volume.parse().ok();

        let instrument = Instrument {
            symbol: response.symbol.clone(),
            exchange: "binance_futures".to_string(),
            instrument_type: InstrumentType::Perpetual,
            base_asset: Some(response.symbol.replace("USDT", "")),
            quote_asset: Some("USDT".to_string()),
            expiry_date: None,
            strike_price: None,
            option_type: None,
        };

        let mut tick = PriceTick::new(instrument, price, Utc::now());
        if let (Some(b), Some(a)) = (bid, ask) {
            tick = tick.with_spread(b, a);
        }
        if let Some(v) = volume {
            tick = tick.with_volume(v);
        }

        Ok(MarketEvent::Price(tick))
    }

    fn exchange_name(&self) -> &str {
        "Binance Futures"
    }
}