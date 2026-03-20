use std::collections::HashMap;
use chrono::Utc;
use crate::engine::{EventBus, EngineEvent, Signal};
use crate::portfolio::position::PositionSide;
use super::order::{Order, OrderSide, OrderType, OrderStatus, TimeInForce};
use super::fill::Fill;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TradeSide {
    Long,
    Short,
}

#[derive(Debug, Clone)]
pub struct Trade {
    pub symbol: String,
    pub signal: Signal,
    pub entry_price: f64,
    pub position_size: f64,
    pub stop_loss: f64,
    pub timestamp: chrono::DateTime<Utc>,
}

pub struct ExecutionEngine {
    event_bus: EventBus,
    trades: Vec<Trade>,
    orders: HashMap<u64, Order>,
    fills: Vec<Fill>,
    next_order_id: u64,
    account_balance: f64,
}

impl ExecutionEngine {
    pub fn new(event_bus: EventBus, initial_balance: f64) -> Self {
        Self {
            event_bus,
            trades: Vec::new(),
            orders: HashMap::new(),
            fills: Vec::new(),
            next_order_id: 1,
            account_balance: initial_balance,
        }
    }

    pub fn execute(
        &mut self,
        symbol: String,
        signal: Signal,
        entry_price: f64,
        position_size: f64,
        stop_loss: f64,
    ) -> Result<Trade, String> {
        let trade_cost = entry_price * position_size;
        
        if trade_cost > self.account_balance {
            return Err(format!("Insufficient balance: need ${:.2}, have ${:.2}", trade_cost, self.account_balance));
        }
        
        self.account_balance -= trade_cost;

        let order_side = match signal {
            Signal::Buy => OrderSide::Buy,
            Signal::Sell => OrderSide::Sell,
            Signal::Hold => return Err("Cannot execute HOLD signal".to_string()),
        };

        let order_id = self.submit_order(
            symbol.clone(),
            order_side,
            OrderType::Market,
            TimeInForce::IOC,
            position_size,
            Some(entry_price),
        )?;

        let fill = Fill::new(order_id, symbol.clone(), position_size, entry_price);
        self.fills.push(fill.clone());

        let order = self.orders.get_mut(&order_id).ok_or("Order not found")?;
        order.fill(position_size, entry_price);
        order.status = OrderStatus::Filled;

        self.event_bus.publish(EngineEvent::OrderFilled {
            order_id,
            symbol: symbol.clone(),
            filled_qty: position_size,
            price: entry_price,
        }).ok();

        let trade = Trade {
            symbol: symbol.clone(),
            signal,
            entry_price,
            position_size,
            stop_loss,
            timestamp: Utc::now(),
        };

        self.trades.push(trade.clone());

        self.event_bus.publish(EngineEvent::TradeExecuted {
            symbol,
            signal,
            entry_price,
            position_size,
            stop_loss,
        }).ok();

        Ok(trade)
    }

    pub fn submit_order(
        &mut self,
        symbol: String,
        side: OrderSide,
        order_type: OrderType,
        tif: TimeInForce,
        quantity: f64,
        price: Option<f64>,
    ) -> Result<u64, String> {
        if quantity <= 0.0 {
            return Err("Order quantity must be positive".to_string());
        }

        let order_id = self.next_order_id;
        self.next_order_id += 1;

        let order = Order::new(
            order_id,
            symbol.clone(),
            side,
            order_type,
            tif,
            quantity,
            price,
        );

        self.orders.insert(order_id, order.clone());

        let signal = match side {
            OrderSide::Buy => Signal::Buy,
            OrderSide::Sell => Signal::Sell,
        };

        self.event_bus.publish(EngineEvent::OrderSubmitted {
            order_id,
            symbol,
            side: signal,
            quantity,
            price,
        }).ok();

        Ok(order_id)
    }

    pub fn cancel_order(&mut self, order_id: u64) -> Result<(), String> {
        let order = self.orders.get_mut(&order_id).ok_or("Order not found")?;

        if order.is_closed() {
            return Err("Order already closed".to_string());
        }

        order.status = OrderStatus::Cancelled;

        self.event_bus.publish(EngineEvent::OrderCancelled {
            order_id,
            symbol: order.symbol.clone(),
        }).ok();

        Ok(())
    }

    pub fn close_trade(&mut self, symbol: &str, exit_price: f64) -> Result<f64, String> {
        let trade_idx = self.trades.iter().position(|t| t.symbol == symbol)
            .ok_or_else(|| format!("No open trade for {}", symbol))?;
        
        let trade = self.trades.remove(trade_idx);
        
        let pnl = match trade.signal {
            Signal::Buy => (exit_price - trade.entry_price) * trade.position_size,
            Signal::Sell => (trade.entry_price - exit_price) * trade.position_size,
            Signal::Hold => 0.0,
        };
        
        // Return the position value plus PnL to balance
        let position_value = exit_price * trade.position_size;
        self.account_balance += position_value + pnl;

        self.event_bus.publish(EngineEvent::TradeClosed {
            symbol: symbol.to_string(),
            exit_price,
            pnl,
        }).ok();

        Ok(pnl)
    }

    pub fn check_stop_loss(&self, symbol: &str, current_price: f64) -> Option<bool> {
        let trade = self.trades.iter().find(|t| t.symbol == symbol)?;
        
        let is_long = matches!(trade.signal, Signal::Buy);
        let stop_hit = if is_long {
            current_price <= trade.stop_loss
        } else {
            current_price >= trade.stop_loss
        };
        
        Some(stop_hit)
    }

    pub fn balance(&self) -> f64 {
        self.account_balance
    }

    pub fn open_positions(&self) -> usize {
        self.trades.len()
    }

    pub fn trades(&self) -> &[Trade] {
        &self.trades
    }

    pub fn orders(&self) -> &HashMap<u64, Order> {
        &self.orders
    }

    pub fn fills(&self) -> &[Fill] {
        &self.fills
    }
}