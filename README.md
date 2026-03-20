# Event-Driven Trading Engine

A high-performance, event-driven cryptocurrency trading system built in Rust with real-time market data ingestion, pluggable strategies, and paper trading capabilities.

## Features

### Multi-Exchange Support
- **CoinGecko** - Spot market data
- **Binance** - Spot trading
- **Bybit** - Spot trading  
- **Binance Futures** - Perpetual futures

### Instrument Types
- Spot trading
- Perpetual futures
- Future contracts (with expiry)
- Options (with strike prices)

### Trading System
- **Event Bus** - Pub/sub architecture for decoupled components
- **Portfolio Management** - Position tracking, PnL calculation (realized/unrealized)
- **Execution Engine** - Paper trading with order management
- **Risk Management** - Stop-loss, position limits

### Strategies
- **Simple Strategy** - Price change threshold-based trading
- **Arbitrage Strategy** - Cross-exchange price difference detection

## Project Structure

```
src/
├── main.rs                 # CLI entry point with interactive menu
├── lib.rs                  # Library exports
├── engine/                 # Event bus & event types
│   ├── bus.rs             # Pub/sub event system
│   └── event.rs           # Engine events (TradeExecuted, OrderFilled, etc.)
├── market_data/            # Multi-exchange data fetching
│   ├── adapters/          # Exchange implementations
│   │   ├── binance.rs     # Binance spot fetcher
│   │   ├── binance_futures.rs
│   │   ├── bybit.rs       # Bybit spot fetcher
│   │   └── coingecko.rs   # CoinGecko API
│   ├── fetcher.rs         # MarketDataFetcher trait
│   ├── feed.rs            # MarketDataFeed trait (streaming)
│   ├── types.rs           # Instrument, PriceTick, TradeTick
│   └── normalizer.rs      # Data normalization
├── strategy/              # Trading strategies
│   ├── mod.rs             # Strategy trait
│   ├── simple.rs          # Simple price threshold strategy
│   ├── event.rs           # Strategy events (Buy/Sell/Arbitrage)
│   └── cross_exchange/
│       └── arbitrage.rs   # Arbitrage strategy
├── execution/             # Order execution
│   ├── engine.rs          # ExecutionEngine (paper trading)
│   ├── order.rs           # Order types & states
│   └── fill.rs            # Fill simulation
├── portfolio/             # Position management
│   ├── portfolio.rs       # Portfolio with PnL
│   └── position.rs        # Position with stop-loss
└── utils/                 # Utilities
```

## Getting Started

### Prerequisites
- Rust 1.70+
- API key for CoinGecko (optional, for CoinGecko data source)

### Build & Run

```bash
# Build
cargo build

# Run
cargo run
```

### Usage

1. Select a data source from the menu:
   - CoinGecko (Spot)
   - Binance (Spot)
   - Bybit (Spot)
   - Binance Futures (Perpetual)

2. The system will:
   - Fetch real-time market data
   - Run the strategy to generate signals
   - Execute trades (paper trading)
   - Display portfolio status

### Environment Variables

For CoinGecko:
```bash
COINGECKO_API_KEY=your_api_key
COIN_IDS=bitcoin,ethereum,solana
```

## Architecture

### Event Flow
```
Exchange API → MarketDataFetcher → Strategy → ExecutionEngine → Portfolio
                    ↓
              EventBus (pub/sub)
                    ↓
              Subscribers (logging, alerts)
```

### Key Traits

```rust
// Pull-based data fetching
trait MarketDataFetcher {
    async fn fetch_price(&self, symbol: &str) -> Result<MarketEvent>;
    fn exchange_name(&self) -> &str;
}

// Stream-based data fetching
trait MarketDataFeed {
    async fn next_event(&mut self) -> Option<MarketEvent>;
    fn source(&self) -> &str;
}

// Trading strategy
trait Strategy: Send {
    async fn on_market_event(&mut self, event: MarketEvent) -> Option<StrategyEvent>;
    fn name(&self) -> &str;
}
```

## Dependencies

- `tokio` - Async runtime
- `reqwest` - HTTP client
- `serde` - JSON serialization
- `chrono` - Date/time handling
- `async_trait` - Async traits
- `coingecko` - CoinGecko API client
- `governor` - Rate limiting

## License

MIT