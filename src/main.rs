use tokio::time::{sleep, Duration};

use rust_event_driven_trader::market_data::{
    adapters::coingecko::CoinGeckoFeed,
    feed::MarketDataFeed,
};

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    
    let api_key = std::env::var("COINGECKO_API_KEY")
        .expect("COINGECKO_API_KEY environment variable not set");
    
    let coin_ids = std::env::var("COIN_IDS")
        .unwrap_or_else(|_| "bitcoin".to_string());
    
    let coins: Vec<&str> = coin_ids.split(',').map(|s| s.trim()).collect();
    
    let mut feeds: Vec<CoinGeckoFeed> = coins
        .iter()
        .map(|coin_id| CoinGeckoFeed::new(coin_id, "usd", &api_key))
        .collect();

    loop {
        for feed in &mut feeds {
            if let Some(event) = feed.next_event().await {
                println!("Market Event: {:?}", event);
            }
        }

        sleep(Duration::from_secs(5)).await;
    }
}