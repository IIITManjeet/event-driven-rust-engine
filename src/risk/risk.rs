use crate::engine::Signal;

#[derive(Debug, Clone)]
pub struct RiskLimits {
    pub max_positions: usize,
    pub max_position_size: f64,
    pub max_daily_loss_pct: f64,
    pub max_exposure: f64,
    pub min_trade_size: f64,
}

impl Default for RiskLimits {
    fn default() -> Self {
        Self {
            max_positions: 5,
            max_position_size: 1.0,
            max_daily_loss_pct: 0.10, // 10%
            max_exposure: 0.8, // 80% of balance
            min_trade_size: 0.001,
        }
    }
}

impl RiskLimits {
    pub fn new(max_positions: usize, max_position_size: f64, max_daily_loss_pct: f64) -> Self {
        Self {
            max_positions,
            max_position_size,
            max_daily_loss_pct,
            max_exposure: 0.8,
            min_trade_size: 0.001,
        }
    }

    pub fn conservative() -> Self {
        Self {
            max_positions: 3,
            max_position_size: 0.5,
            max_daily_loss_pct: 0.05, // 5%
            max_exposure: 0.5,
            min_trade_size: 0.01,
        }
    }

    pub fn aggressive() -> Self {
        Self {
            max_positions: 10,
            max_position_size: 2.0,
            max_daily_loss_pct: 0.20, // 20%
            max_exposure: 0.95,
            min_trade_size: 0.001,
        }
    }
}

#[derive(Debug, Clone)]
pub struct RiskState {
    pub kill_switch_active: bool,
    pub kill_reason: Option<String>,
    pub daily_start_balance: f64,
    pub daily_pnl: f64,
    pub trades_today: usize,
}

impl Default for RiskState {
    fn default() -> Self {
        Self {
            kill_switch_active: false,
            kill_reason: None,
            daily_start_balance: 0.0,
            daily_pnl: 0.0,
            trades_today: 0,
        }
    }
}

impl RiskState {
    pub fn new(initial_balance: f64) -> Self {
        Self {
            kill_switch_active: false,
            kill_reason: None,
            daily_start_balance: initial_balance,
            daily_pnl: 0.0,
            trades_today: 0,
        }
    }

    pub fn check_daily_loss(&mut self, current_balance: f64, daily_loss_limit: f64) {
        self.daily_pnl = current_balance - self.daily_start_balance;
        
        if self.daily_pnl < -daily_loss_limit {
            self.kill_switch_active = true;
            self.kill_reason = Some(format!(
                "Daily loss limit hit: ${:.2} (limit: ${:.2})",
                self.daily_pnl, daily_loss_limit
            ));
        }
    }

    pub fn reset_daily(&mut self, current_balance: f64) {
        self.daily_start_balance = current_balance;
        self.daily_pnl = 0.0;
        self.trades_today = 0;
    }
}

pub struct RiskEngine {
    limits: RiskLimits,
    state: RiskState,
}

impl RiskEngine {
    pub fn new(initial_balance: f64, limits: RiskLimits) -> Self {
        Self {
            limits,
            state: RiskState::new(initial_balance),
        }
    }

    pub fn with_default_limits(initial_balance: f64) -> Self {
        Self::new(initial_balance, RiskLimits::default())
    }

    pub fn pre_trade_check(
        &self,
        current_positions: usize,
        balance: f64,
        trade_value: f64,
        symbol: &str,
    ) -> Result<(), String> {
        // Check kill switch
        if self.state.kill_switch_active {
            return Err(format!(
                "Kill switch active: {}",
                self.state.kill_reason.as_deref().unwrap_or("Unknown")
            ));
        }

        // Check max positions
        if current_positions >= self.limits.max_positions {
            return Err(format!(
                "Max positions reached: {}",
                self.limits.max_positions
            ));
        }

        // Check position size
        if trade_value / balance > self.limits.max_exposure {
            return Err(format!(
                "Trade would exceed max exposure: {:.1}%",
                self.limits.max_exposure * 100.0
            ));
        }

        // Check min trade size
        if trade_value < self.limits.min_trade_size * balance {
            return Err(format!(
                "Trade too small: min ${:.2}",
                self.limits.min_trade_size * balance
            ));
        }

        Ok(())
    }

    pub fn post_trade_check(&mut self, current_balance: f64) {
        let daily_loss_limit = self.state.daily_start_balance * self.limits.max_daily_loss_pct;
        self.state.check_daily_loss(current_balance, daily_loss_limit);
        self.state.trades_today += 1;
    }

    pub fn is_kill_switch_active(&self) -> bool {
        self.state.kill_switch_active
    }

    pub fn kill_reason(&self) -> Option<&str> {
        self.state.kill_reason.as_deref()
    }

    pub fn reset(&mut self, current_balance: f64) {
        self.state.reset_daily(current_balance);
    }

    pub fn get_limits(&self) -> &RiskLimits {
        &self.limits
    }

    pub fn get_state(&self) -> &RiskState {
        &self.state
    }
}