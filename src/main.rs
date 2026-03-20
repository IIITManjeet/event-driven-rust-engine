use tokio::time::{sleep, Duration};
use std::collections::HashMap;

use rust_event_driven_trader::market_data::{
    adapters::binance::BinanceFetcher,
    adapters::binance_futures::BinanceFuturesFetcher,
    adapters::bybit::BybitFetcher,
    adapters::coingecko::CoinGeckoFeed,
    feed::MarketDataFeed,
    fetcher::MarketDataFetcher,
};
use rust_event_driven_trader::engine::{EventBus, EngineEvent, Signal};
use rust_event_driven_trader::portfolio::{Portfolio, PositionSide};
use rust_event_driven_trader::execution::ExecutionEngine;
use rust_event_driven_trader::strategy::{Strategy, SimpleStrategy};
use rust_event_driven_trader::risk::{RiskEngine, RiskLimits};

const RESET: &str = "\x1b[0m";
const BOLD: &str = "\x1b[1m";
const DIM: &str = "\x1b[2m";

const GREEN: &str = "\x1b[32m";
const RED: &str = "\x1b[31m";
const YELLOW: &str = "\x1b[33m";
const BLUE: &str = "\x1b[34m";
const CYAN: &str = "\x1b[36m";
const MAGENTA: &str = "\x1b[35m";

fn print_header(title: &str) {
    println!("\n{}{}{}", BOLD, CYAN, "═".repeat(50));
    println!("{}  {}  {}", CYAN, title, RESET);
    println!("{}{}{}", BOLD, CYAN, "═".repeat(50));
}

fn print_menu() {
    println!("\n{}{}╔══════════════════════════════════════╗{}", BOLD, BLUE, RESET);
    println!("{}{}║     TRADING ENGINE - DATA SOURCE    ║{}", BOLD, BLUE, RESET);
    println!("{}{}╚══════════════════════════════════════╝{}", BOLD, BLUE, RESET);
    println!("\n  {}1.{}{} CoinGecko        (Spot)", BOLD, GREEN, RESET);
    println!("  {}2.{}{} Binance          (Spot)", BOLD, GREEN, RESET);
    println!("  {}3.{}{} Bybit            (Spot)", BOLD, GREEN, RESET);
    println!("  {}4.{}{} Binance Futures  (Perpetual)", BOLD, GREEN, RESET);
    print!("\n  {}Select: {} ", BOLD, YELLOW);
}

fn print_price(instrument: &str, price: f64, change: f64, exchange: &str) {
    let color = if change >= 0.0 { GREEN } else { RED };
    let sign = if change >= 0.0 { "+" } else { "" };
    println!(
        "  {}│ {} {:12} │ ${:>10.2} │ {} {}{:>6.4}% │",
        DIM, exchange, instrument, price, color, sign, change
    );
}

fn print_signal(_signal_type: &str, symbol: &str, price: f64, color: &str) {
    println!(
        "\n  {}⚡ {} SIGNAL{} | {} @ ${:.2}",
        BOLD, color, RESET, symbol, price
    );
}

fn print_trade(action: &str, symbol: &str, price: f64, size: f64, sl: f64) {
    let line = format!("{}{}", BOLD, "─".repeat(40));
    println!("  {}│{}│{}", line, RESET, BOLD);
    println!("  {}│ {} Action: {} {}", BOLD, GREEN, action, RESET);
    println!("  {}│ {} Symbol:  {} {}", BOLD, GREEN, symbol, RESET);
    println!("  {}│ {} Price:   ${:.2} {}", BOLD, GREEN, price, RESET);
    println!("  {}│ {} Size:    {} {}", BOLD, GREEN, size, RESET);
    println!("  {}│ {} Stop:    ${:.2} {}", BOLD, GREEN, sl, RESET);
    println!("  {}│{}│{}", line, RESET, BOLD);
}

fn print_portfolio(balance: f64, positions: usize, unrealized: f64, realized: f64, risk_engine: &RiskEngine) {
    let limits = risk_engine.get_limits();
    let state = risk_engine.get_state();
    
    println!("\n  {}┌─────────────────────────────────────┐{}", BOLD, RESET);
    println!("  {}│ {} PORTFOLIO STATUS{}              │", BOLD, YELLOW, RESET);
    println!("  {}├─────────────────────────────────────┤{}", BOLD, RESET);
    println!("  {}│ {} Balance:      ${:>12.2}    │{}", BOLD, DIM, balance, RESET);
    println!("  {}│ {} Open Trades:  {:>12}/{}    │{}", BOLD, DIM, positions, limits.max_positions, RESET);
    
    let pnl_color = if unrealized >= 0.0 { GREEN } else { RED };
    println!("  {}│ {} Unrealized:   {}${:>11.2}    │{}", BOLD, DIM, pnl_color, unrealized, RESET);
    
    let realized_color = if realized >= 0.0 { GREEN } else { RED };
    println!("  {}│ {} Realized:     {}${:>11.2}    │{}", BOLD, DIM, realized_color, realized, RESET);
    
    let risk_color = if state.kill_switch_active { RED } else { GREEN };
    let risk_status = if state.kill_switch_active { 
        state.kill_reason.as_deref().unwrap_or("ACTIVE") 
    } else { 
        "OK" 
    };
    println!("  {}├─────────────────────────────────────┤{}", BOLD, RESET);
    println!("  {}│ {} Risk Status:  {}{}        │{}", BOLD, DIM, risk_color, risk_status, RESET);
    println!("  {}│ {} Daily P&L:    ${:>11.2}    │{}", BOLD, DIM, state.daily_pnl, RESET);
    println!("  {}│ {} Max Loss:     {:>11.1}%    │{}", BOLD, DIM, limits.max_daily_loss_pct * 100.0, RESET);
    println!("  {}└─────────────────────────────────────┘{}", BOLD, RESET);
}

fn print_skip(symbol: &str, reason: &str) {
    println!("  {}│ {}⏭️  {} - {}│", DIM, YELLOW, symbol, reason);
}

fn print_trade_closed(symbol: &str, exit_price: f64, pnl: f64, reason: &str) {
    let color = if pnl >= 0.0 { GREEN } else { RED };
    let sign = if pnl >= 0.0 { "+" } else { "" };
    println!(
        "  {}│ {}🔚 CLOSED{} | {} @ ${:.2} | PnL: {}{}${:.2} | {}│",
        BOLD, YELLOW, RESET, symbol, exit_price, color, sign, pnl, reason
    );
}

fn update_and_check_positions(
    portfolio: &mut Portfolio,
    execution: &mut ExecutionEngine,
    event_bus: &EventBus,
    prices: &HashMap<String, f64>,
) {
    let symbols = portfolio.position_symbols();
    
    for symbol in symbols {
        let current_price = match prices.get(&symbol) {
            Some(p) => *p,
            None => continue,
        };
        
        portfolio.update_price(&symbol, current_price).ok();
        
        let stop_hit = if let Some(position) = portfolio.get_position(&symbol) {
            position.is_stop_loss_hit()
        } else {
            false
        };
        
        if stop_hit {
            println!("\n  {}⚠️  STOP LOSS TRIGGERED{} │ {} @ ${:.2}", BOLD, RED, symbol, current_price);
            
            match execution.close_trade(&symbol, current_price) {
                Ok(pnl) => {
                    let _ = portfolio.close_position(&symbol, current_price);
                    print_trade_closed(&symbol, current_price, pnl, "Stop Loss");
                    
                    event_bus.publish(EngineEvent::TradeClosed {
                        symbol: symbol.clone(),
                        exit_price: current_price,
                        pnl,
                    }).ok();
                }
                Err(e) => println!("  {}│ Error closing trade: {}│", RED, e),
            }
        }
    }
}

#[tokio::main]
async fn main() {
    print_menu();
    
    let mut choice = String::new();
    std::io::stdin().read_line(&mut choice).unwrap();
    let choice = choice.trim();

    let event_bus = EventBus::new();
    let mut portfolio = Portfolio::new();
    let mut execution = ExecutionEngine::new(event_bus.clone(), 10000.0);
    let mut risk_engine = RiskEngine::with_default_limits(10000.0);
    let mut simple_strategy = SimpleStrategy::new(0.01);
    let mut latest_prices: HashMap<String, f64> = HashMap::new();

    event_bus.subscribe("TradeExecuted", |event: &EngineEvent| {
        if let EngineEvent::TradeExecuted { symbol, entry_price, position_size, stop_loss, .. } = event {
            print_trade("BUY", symbol, *entry_price, *position_size, *stop_loss);
        }
    }).ok();

    match choice {
        "1" => run_coingecko(event_bus, &mut portfolio, &mut execution, &mut simple_strategy, &mut latest_prices, &mut risk_engine).await,
        "2" => run_binance(event_bus, &mut portfolio, &mut execution, &mut simple_strategy, &mut latest_prices, &mut risk_engine).await,
        "3" => run_bybit(event_bus, &mut portfolio, &mut execution, &mut simple_strategy, &mut latest_prices, &mut risk_engine).await,
        "4" => run_binance_futures(event_bus, &mut portfolio, &mut execution, &mut simple_strategy, &mut latest_prices, &mut risk_engine).await,
        _ => println!("{}Invalid choice!{}", RED, RESET),
    }
}

async fn run_coingecko(
    event_bus: EventBus,
    portfolio: &mut Portfolio,
    execution: &mut ExecutionEngine,
    strategy: &mut SimpleStrategy,
    latest_prices: &mut HashMap<String, f64>,
    risk_engine: &mut RiskEngine,
) {
    dotenv::dotenv().ok();
    
    let api_key = std::env::var("COINGECKO_API_KEY")
        .expect("COINGECKO_API_KEY not set");
    
    let coin_ids = std::env::var("COIN_IDS")
        .unwrap_or_else(|_| "bitcoin,ethereum".to_string());
    
    let coins: Vec<&str> = coin_ids.split(',').map(|s| s.trim()).collect();
    
    let mut feeds: Vec<CoinGeckoFeed> = coins
        .iter()
        .map(|coin_id| CoinGeckoFeed::new(coin_id, "usd", &api_key))
        .collect();

    print_header("COINGECKO TRADING (Spot)");
    println!("  {}Coins: {:?}{}", DIM, coins, RESET);

    loop {
        for feed in &mut feeds {
            if let Some(event) = feed.next_event().await {
                if let rust_event_driven_trader::market_data::event::MarketEvent::Price(tick) = &event {
                    print_price(&tick.instrument.symbol, tick.price, 0.0, "CoinGecko");
                    latest_prices.insert(tick.instrument.symbol.clone(), tick.price);
                }
                
                if let Some(strategy_event) = strategy.on_market_event(event.clone()).await {
                    handle_strategy_event(strategy_event, portfolio, execution, risk_engine);
                }

                event_bus.publish(EngineEvent::PriceUpdated(event)).ok();
            }
        }
        
        update_and_check_positions(portfolio, execution, &event_bus, latest_prices);
        print_portfolio(execution.balance(), execution.open_positions(), portfolio.unrealized_pnl(), portfolio.realized_pnl(), risk_engine);
        sleep(Duration::from_secs(5)).await;
    }
}

async fn run_binance(
    event_bus: EventBus,
    portfolio: &mut Portfolio,
    execution: &mut ExecutionEngine,
    strategy: &mut SimpleStrategy,
    latest_prices: &mut HashMap<String, f64>,
    risk_engine: &mut RiskEngine,
) {
    let fetcher = BinanceFetcher::new();
    let symbols = ["BTCUSDT", "ETHUSDT"];

    print_header("BINANCE TRADING (Spot)");
    println!("  {}Symbols: {:?}{}", DIM, symbols, RESET);

    loop {
        for symbol in &symbols {
            match fetcher.fetch_price(symbol).await {
                Ok(event) => {
                    if let rust_event_driven_trader::market_data::event::MarketEvent::Price(tick) = &event {
                        print_price(&tick.instrument.symbol, tick.price, 0.0, "Binance");
                        latest_prices.insert(tick.instrument.symbol.clone(), tick.price);
                    }
                    
                    if let Some(strategy_event) = strategy.on_market_event(event.clone()).await {
                        handle_strategy_event(strategy_event, portfolio, execution, risk_engine);
                    }

                    event_bus.publish(EngineEvent::PriceUpdated(event)).ok();
                }
                Err(e) => println!("  {}│ Error: {}│", RED, e),
            }
        }
        
        update_and_check_positions(portfolio, execution, &event_bus, latest_prices);
        print_portfolio(execution.balance(), execution.open_positions(), portfolio.unrealized_pnl(), portfolio.realized_pnl(), risk_engine);
        sleep(Duration::from_secs(5)).await;
    }
}

async fn run_bybit(
    event_bus: EventBus,
    portfolio: &mut Portfolio,
    execution: &mut ExecutionEngine,
    strategy: &mut SimpleStrategy,
    latest_prices: &mut HashMap<String, f64>,
    risk_engine: &mut RiskEngine,
) {
    let fetcher = BybitFetcher::new();
    let symbols = ["BTC", "ETH"];

    print_header("BYBIT TRADING (Spot)");
    println!("  {}Symbols: {:?}{}", DIM, symbols, RESET);

    loop {
        for symbol in &symbols {
            match fetcher.fetch_price(symbol).await {
                Ok(event) => {
                    if let rust_event_driven_trader::market_data::event::MarketEvent::Price(tick) = &event {
                        print_price(&tick.instrument.symbol, tick.price, 0.0, "Bybit");
                        latest_prices.insert(tick.instrument.symbol.clone(), tick.price);
                    }
                    
                    if let Some(strategy_event) = strategy.on_market_event(event.clone()).await {
                        handle_strategy_event(strategy_event, portfolio, execution, risk_engine);
                    }

                    event_bus.publish(EngineEvent::PriceUpdated(event)).ok();
                }
                Err(e) => println!("  {}│ Error: {}│", RED, e),
            }
        }
        
        update_and_check_positions(portfolio, execution, &event_bus, latest_prices);
        print_portfolio(execution.balance(), execution.open_positions(), portfolio.unrealized_pnl(), portfolio.realized_pnl(), risk_engine);
        sleep(Duration::from_secs(5)).await;
    }
}

async fn run_binance_futures(
    event_bus: EventBus,
    portfolio: &mut Portfolio,
    execution: &mut ExecutionEngine,
    strategy: &mut SimpleStrategy,
    latest_prices: &mut HashMap<String, f64>,
    risk_engine: &mut RiskEngine,
) {
    let fetcher = BinanceFuturesFetcher::new();
    let symbols = ["BTCUSDT", "ETHUSDT"];

    print_header("BINANCE FUTURES (Perpetual)");
    println!("  {}Symbols: {:?}{}", DIM, symbols, RESET);

    loop {
        for symbol in &symbols {
            match fetcher.fetch_price(symbol).await {
                Ok(event) => {
                    if let rust_event_driven_trader::market_data::event::MarketEvent::Price(tick) = &event {
                        print_price(&tick.instrument.symbol, tick.price, 0.0, "Binance Futures");
                        latest_prices.insert(tick.instrument.symbol.clone(), tick.price);
                    }
                    
                    if let Some(strategy_event) = strategy.on_market_event(event.clone()).await {
                        handle_strategy_event(strategy_event, portfolio, execution, risk_engine);
                    }

                    event_bus.publish(EngineEvent::PriceUpdated(event)).ok();
                }
                Err(e) => println!("  {}│ Error: {}│", RED, e),
            }
        }
        
        update_and_check_positions(portfolio, execution, &event_bus, latest_prices);
        print_portfolio(execution.balance(), execution.open_positions(), portfolio.unrealized_pnl(), portfolio.realized_pnl(), risk_engine);
        sleep(Duration::from_secs(5)).await;
    }
}

fn handle_strategy_event(
    event: rust_event_driven_trader::strategy::StrategyEvent,
    portfolio: &mut Portfolio,
    execution: &mut ExecutionEngine,
    risk_engine: &mut RiskEngine,
) {
    match event {
        rust_event_driven_trader::strategy::StrategyEvent::Buy { instrument, price, timestamp: _ } => {
            if portfolio.get_position(&instrument.symbol).is_some() {
                print_skip(&instrument.symbol, "Position already open");
                return;
            }
            
            let trade_value = price * 0.01;
            if let Err(e) = risk_engine.pre_trade_check(
                portfolio.open_positions(),
                execution.balance(),
                trade_value,
                &instrument.symbol,
            ) {
                println!("  {}│ Risk rejected: {}│", RED, e);
                return;
            }
            
            print_signal("BUY", &instrument.symbol, price, GREEN);
            
            let position_size = 0.01;
            let stop_loss = price * 0.95;
            
            match execution.execute(
                instrument.symbol.clone(),
                Signal::Buy,
                price,
                position_size,
                stop_loss,
            ) {
                Ok(_) => {
                    portfolio.open_position(
                        instrument.symbol,
                        PositionSide::Long,
                        price,
                        position_size,
                        stop_loss,
                    ).ok();
                    risk_engine.post_trade_check(execution.balance());
                }
                Err(e) => println!("  {}│ Execution failed: {}│", RED, e),
            }
        }
        
        rust_event_driven_trader::strategy::StrategyEvent::Sell { instrument, price, timestamp: _ } => {
            if portfolio.get_position(&instrument.symbol).is_some() {
                print_skip(&instrument.symbol, "Position already open");
                return;
            }
            
            let trade_value = price * 0.01;
            if let Err(e) = risk_engine.pre_trade_check(
                portfolio.open_positions(),
                execution.balance(),
                trade_value,
                &instrument.symbol,
            ) {
                println!("  {}│ Risk rejected: {}│", RED, e);
                return;
            }
            
            print_signal("SELL", &instrument.symbol, price, RED);
            
            let position_size = 0.01;
            let stop_loss = price * 1.05;
            
            match execution.execute(
                instrument.symbol.clone(),
                Signal::Sell,
                price,
                position_size,
                stop_loss,
            ) {
                Ok(_) => {
                    portfolio.open_position(
                        instrument.symbol,
                        PositionSide::Short,
                        price,
                        position_size,
                        stop_loss,
                    ).ok();
                    risk_engine.post_trade_check(execution.balance());
                }
                Err(e) => println!("  {}│ Execution failed: {}│", RED, e),
            }
        }
        
        rust_event_driven_trader::strategy::StrategyEvent::Arbitrage { 
            buy_exchange, 
            sell_exchange, 
            instrument, 
            spread, 
            timestamp: _ 
        } => {
            print_signal("ARBITRAGE", &instrument.symbol, spread, MAGENTA);
            println!("  {}│ Buy:  {} │ Sell: {}│", DIM, buy_exchange, sell_exchange);
        }
    }
}