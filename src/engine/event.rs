use crate::market_data::event::MarketEvent;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Signal {
    Buy,
    Sell,
    Hold,
}

#[derive(Debug, Clone)]
pub enum EngineEvent {
    PriceUpdated(MarketEvent),
    
    SignalGenerated {
        strategy_name: String,
        symbol: String,
        signal: Signal,
        price: f64,
    },
    
    TradeExecuted {
        symbol: String,
        signal: Signal,
        entry_price: f64,
        position_size: f64,
        stop_loss: f64,
    },
    
    TradeClosed {
        symbol: String,
        exit_price: f64,
        pnl: f64,
    },

    OrderSubmitted {
        order_id: u64,
        symbol: String,
        side: Signal,
        quantity: f64,
        price: Option<f64>,
    },

    OrderFilled {
        order_id: u64,
        symbol: String,
        filled_qty: f64,
        price: f64,
    },

    OrderCancelled {
        order_id: u64,
        symbol: String,
    },

    OrderRejected {
        order_id: u64,
        symbol: String,
        reason: String,
    },

    RiskHalt {
        reason: String,
    },
    
    Error(String),
}

impl EngineEvent {
    pub fn event_type(&self) -> &str {
        match self {
            EngineEvent::PriceUpdated(_) => "PriceUpdated",
            EngineEvent::SignalGenerated { .. } => "SignalGenerated",
            EngineEvent::TradeExecuted { .. } => "TradeExecuted",
            EngineEvent::TradeClosed { .. } => "TradeClosed",
            EngineEvent::OrderSubmitted { .. } => "OrderSubmitted",
            EngineEvent::OrderFilled { .. } => "OrderFilled",
            EngineEvent::OrderCancelled { .. } => "OrderCancelled",
            EngineEvent::OrderRejected { .. } => "OrderRejected",
            EngineEvent::RiskHalt { .. } => "RiskHalt",
            EngineEvent::Error(_) => "Error",
        }
    }
}