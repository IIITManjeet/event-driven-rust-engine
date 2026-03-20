use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OrderSide {
    Buy,
    Sell,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OrderType {
    Market,
    Limit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OrderStatus {
    New,
    Submitted,
    PartiallyFilled,
    Filled,
    Cancelled,
    Rejected,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimeInForce {
    GTC,
    IOC,
    FOK,
}

#[derive(Debug, Clone)]
pub struct Order {
    pub id: u64,
    pub symbol: String,
    pub side: OrderSide,
    pub order_type: OrderType,
    pub tif: TimeInForce,
    pub quantity: f64,
    pub price: Option<f64>,
    pub filled_quantity: f64,
    pub status: OrderStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Order {
    pub fn new(
        id: u64,
        symbol: String,
        side: OrderSide,
        order_type: OrderType,
        tif: TimeInForce,
        quantity: f64,
        price: Option<f64>,
    ) -> Self {
        let now = Utc::now();
        Self {
            id,
            symbol,
            side,
            order_type,
            tif,
            quantity,
            price,
            filled_quantity: 0.0,
            status: OrderStatus::New,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn is_closed(&self) -> bool {
        matches!(self.status, OrderStatus::Filled | OrderStatus::Cancelled | OrderStatus::Rejected)
    }

    pub fn fill(&mut self, filled_qty: f64, price: f64) {
        self.filled_quantity += filled_qty;
        self.updated_at = Utc::now();
        
        if self.filled_quantity >= self.quantity {
            self.status = OrderStatus::Filled;
        } else {
            self.status = OrderStatus::PartiallyFilled;
        }
    }
}