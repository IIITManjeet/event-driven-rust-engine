use std::collections::HashMap;
use chrono::{DateTime, Utc};
use super::position::{Position, PositionSide};

#[derive(Debug, Default)]
pub struct Portfolio {
    positions: HashMap<String, Position>,
    realized_pnl: f64,
}

impl Portfolio {
    pub fn new() -> Self {
        Self {
            positions: HashMap::new(),
            realized_pnl: 0.0,
        }
    }

    pub fn open_position(
        &mut self,
        symbol: String,
        side: PositionSide,
        entry_price: f64,
        size: f64,
        stop_loss: f64,
    ) -> Result<(), String> {
        if self.positions.contains_key(&symbol) {
            return Err(format!("Position already open for {}", symbol));
        }

        let position = Position::new(symbol.clone(), side, entry_price, size, stop_loss)?;
        self.positions.insert(symbol, position);
        Ok(())
    }

    pub fn close_position(&mut self, symbol: &str, exit_price: f64) -> Result<f64, String> {
        let mut position = self.positions.remove(symbol).ok_or_else(|| {
            format!("No open position for {}", symbol)
        })?;

        position.update_price(exit_price)?;
        let pnl = position.unrealized_pnl();
        self.realized_pnl += pnl;
        Ok(pnl)
    }

    pub fn update_price(&mut self, symbol: &str, price: f64) -> Result<(), String> {
        if let Some(position) = self.positions.get_mut(symbol) {
            position.update_price(price)?;
        }
        Ok(())
    }

    pub fn get_position(&self, symbol: &str) -> Option<&Position> {
        self.positions.get(symbol)
    }

    pub fn position_symbols(&self) -> Vec<String> {
        self.positions.keys().cloned().collect()
    }

    pub fn open_positions(&self) -> usize {
        self.positions.len()
    }

    pub fn exposure(&self) -> f64 {
        self.positions.values().map(|p| p.notional_value()).sum()
    }

    pub fn unrealized_pnl(&self) -> f64 {
        self.positions.values().map(|p| p.unrealized_pnl()).sum()
    }

    pub fn realized_pnl(&self) -> f64 {
        self.realized_pnl
    }

    pub fn total_pnl(&self) -> f64 {
        self.realized_pnl + self.unrealized_pnl()
    }

    pub fn close_all_at_last(&mut self) -> Vec<(String, f64, f64)> {
        let mut results = Vec::new();
        let symbols: Vec<String> = self.positions.keys().cloned().collect();

        for symbol in symbols {
            if let Some(position) = self.positions.remove(&symbol) {
                let exit_price = position.last_price;
                let pnl = position.unrealized_pnl();
                self.realized_pnl += pnl;
                results.push((symbol, exit_price, pnl));
            }
        }

        results
    }

    pub fn check_stop_losses(&self) -> Vec<String> {
        self.positions
            .iter()
            .filter(|(_, p)| p.is_stop_loss_hit())
            .map(|(symbol, _)| symbol.clone())
            .collect()
    }
}