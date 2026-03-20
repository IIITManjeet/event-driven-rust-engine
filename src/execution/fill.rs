use chrono::{DateTime, Utc};

#[derive(Debug, Clone)]
pub struct Fill {
    pub order_id: u64,
    pub symbol: String,
    pub quantity: f64,
    pub price: f64,
    pub timestamp: DateTime<Utc>,
}

impl Fill {
    pub fn new(order_id: u64, symbol: String, quantity: f64, price: f64) -> Self {
        Self {
            order_id,
            symbol,
            quantity,
            price,
            timestamp: Utc::now(),
        }
    }
}

pub struct FillSimulator;

impl FillSimulator {
    pub fn simulate(
        order_id: u64,
        symbol: &str,
        price: f64,
        quantity: f64,
    ) -> Vec<Fill> {
        vec![Fill::new(order_id, symbol.to_string(), quantity, price)]
    }
}