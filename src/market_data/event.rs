use super::types::*;

#[derive(Debug, Clone)]
pub enum MarketEvent {
    Price(PriceTick),
    Trade(TradeTick),

    // later:
    // OrderBook(OrderBookTick),
    // FundingRate(FundingEvent),
    // Liquidation(LiquidationEvent),
}
