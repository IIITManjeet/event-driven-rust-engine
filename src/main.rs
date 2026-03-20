use tokio::time::{sleep, Duration};

use rust_event_driven_trader::market_data::{
    adapters::binance::BinanceFetcher,
    adapters::bybit::BybitFetcher,
    adapters::coingecko::CoinGeckoFeed,
    feed::MarketDataFeed,
    fetcher::MarketDataFetcher,
};

#[tokio::main]
async fn main() {
    println!("Choose data source:");
    println!("1. CoinGecko");
    println!("2. Binance");
    println!("3. Bybit");
    print!("Enter choice (1-3): ");

    let mut choice = String::new();
    std::io::stdin().read_line(&mut choice).unwrap();
    let choice = choice.trim();

    match choice {
        "1" => run_coingecko().await,
        "2" => run_binance().await,
        "3" => run_bybit().await,
        _ => println!("Invalid choice"),
    }
}

async fn run_coingecko() {
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
                println!("CoinGecko: {:?}", event);
            }
        }
        sleep(Duration::from_secs(5)).await;
    }
}

async fn run_binance() {
    let fetcher = BinanceFetcher::new();
    let symbols = ["BTCUSDT", "ETHUSDT", "SOLUSDT"];

    loop {
        for symbol in &symbols {
            match fetcher.fetch_price(symbol).await {
                Ok(event) => println!("Binance: {:?}", event),
                Err(e) => println!("Binance error: {}", e),
            }
        }
        sleep(Duration::from_secs(5)).await;
    }
}

async fn run_bybit() {
    let fetcher = BybitFetcher::new();
    let symbols = ["BTC", "ETH", "SOL"];

    loop {
        for symbol in &symbols {
            match fetcher.fetch_price(symbol).await {
                Ok(event) => println!("Bybit: {:?}", event),
                Err(e) => println!("Bybit error: {}", e),
            }
        }
        sleep(Duration::from_secs(5)).await;
    }
}