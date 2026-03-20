use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PositionSide {
    Long,
    Short,
}

#[derive(Debug, Clone)]
pub struct Position {
    pub symbol: String,
    pub side: PositionSide,
    pub entry_price: f64,
    pub size: f64,
    pub stop_loss: f64,
    pub opened_at: DateTime<Utc>,
    pub last_price: f64,
}

impl Position {
    pub fn new(
        symbol: String,
        side: PositionSide,
        entry_price: f64,
        size: f64,
        stop_loss: f64,
    ) -> Result<Self, String> {
        if entry_price <= 0.0 || size <= 0.0 || stop_loss <= 0.0 {
            return Err("Entry price, size, and stop loss must be positive".to_string());
        }

        Ok(Self {
            symbol,
            side,
            entry_price,
            size,
            stop_loss,
            opened_at: Utc::now(),
            last_price: entry_price,
        })
    }

    pub fn update_price(&mut self, price: f64) -> Result<(), String> {
        if price <= 0.0 {
            return Err("Price must be positive".to_string());
        }
        self.last_price = price;
        Ok(())
    }

    pub fn notional_value(&self) -> f64 {
        self.entry_price * self.size
    }

    pub fn unrealized_pnl(&self) -> f64 {
        let diff = match self.side {
            PositionSide::Long => self.last_price - self.entry_price,
            PositionSide::Short => self.entry_price - self.last_price,
        };
        diff * self.size
    }

    pub fn is_stop_loss_hit(&self) -> bool {
        match self.side {
            PositionSide::Long => self.last_price <= self.stop_loss,
            PositionSide::Short => self.last_price >= self.stop_loss,
        }
    }
}