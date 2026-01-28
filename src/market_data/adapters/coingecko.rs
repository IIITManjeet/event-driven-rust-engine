use async_trait::async_trait;
use chrono::Utc;
use governor::{
    RateLimiter,
    clock::DefaultClock,
    state::InMemoryState,
};
use governor::Quota;
use nonzero_ext::nonzero;
use std::sync::Arc;

use coingecko::CoinGeckoClient;

use crate::market_data::{
    event::MarketEvent,
    feed::MarketDataFeed,
    normalizer::normalize_coingecko_price,
};

pub struct CoinGeckoFeed {
    client: CoinGeckoClient,
    coin_id: String,
    vs_currency: String,
    rate_limiter: Arc<RateLimiter<
    governor::state::NotKeyed,
    InMemoryState,
    DefaultClock,
>>,
}

impl CoinGeckoFeed {
    pub fn new(coin_id: &str, vs_currency: &str, api_key: &str) -> Self {
        let quota = Quota::per_second(nonzero!(10u32));
        let rate_limiter = Arc::new(RateLimiter::direct(
        governor::Quota::per_second(nonzero_ext::nonzero!(5u32))
    ));

        Self {
            client: CoinGeckoClient::new_with_demo_api_key(api_key),
            coin_id: coin_id.to_string(),
            vs_currency: vs_currency.to_string(),
            rate_limiter,
        }
    }
}

#[async_trait]
impl MarketDataFeed for CoinGeckoFeed {
    async fn next_event(&mut self) -> Option<MarketEvent> {
        self.rate_limiter.until_ready().await;

        let data = self
            .client
            .price(
                &[self.coin_id.as_str()],
                &[self.vs_currency.as_str()],
                false,
                false,
                false,
                false,
            )
            .await
            .ok()?;

        let price_obj = data.get(&self.coin_id)?;

        let price = match self.vs_currency.as_str() {
            "usd" => price_obj.usd?,
            "eur" => price_obj.eur?,
            "inr" => price_obj.inr?,
            _ => return None,
        };

        Some(normalize_coingecko_price(
            &self.coin_id,
            price,
            Utc::now(),
        ))
    }

    fn source(&self) -> &str {
        "coingecko"
    }
}
