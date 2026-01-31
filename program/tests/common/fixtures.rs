//! Test fixtures for Matchbook integration tests.
//!
//! This module provides pre-configured test scenarios and data
//! for consistent and repeatable integration testing.

use super::{
    DEFAULT_BASE_LOT_SIZE, DEFAULT_MAKER_FEE_BPS, DEFAULT_MIN_ORDER_SIZE, DEFAULT_QUOTE_LOT_SIZE,
    DEFAULT_TAKER_FEE_BPS, DEFAULT_TICK_SIZE,
};

/// Market configuration for testing.
#[derive(Debug, Clone)]
pub struct MarketConfig {
    /// Base token lot size in native units.
    pub base_lot_size: u64,
    /// Quote token lot size in native units.
    pub quote_lot_size: u64,
    /// Minimum price increment in quote lots.
    pub tick_size: u64,
    /// Minimum order size in base lots.
    pub min_order_size: u64,
    /// Taker fee in basis points.
    pub taker_fee_bps: u16,
    /// Maker fee in basis points (negative = rebate).
    pub maker_fee_bps: i16,
}

impl Default for MarketConfig {
    fn default() -> Self {
        Self {
            base_lot_size: DEFAULT_BASE_LOT_SIZE,
            quote_lot_size: DEFAULT_QUOTE_LOT_SIZE,
            tick_size: DEFAULT_TICK_SIZE,
            min_order_size: DEFAULT_MIN_ORDER_SIZE,
            taker_fee_bps: DEFAULT_TAKER_FEE_BPS,
            maker_fee_bps: DEFAULT_MAKER_FEE_BPS,
        }
    }
}

impl MarketConfig {
    /// Creates a market config with zero fees for simpler testing.
    #[must_use]
    pub fn zero_fees() -> Self {
        Self {
            taker_fee_bps: 0,
            maker_fee_bps: 0,
            ..Default::default()
        }
    }

    /// Creates a market config with high fees for fee testing.
    #[must_use]
    pub fn high_fees() -> Self {
        Self {
            taker_fee_bps: 100, // 1%
            maker_fee_bps: 50,  // 0.5%
            ..Default::default()
        }
    }
}

/// Order parameters for testing.
#[derive(Debug, Clone)]
pub struct OrderParams {
    /// Price in ticks.
    pub price: u64,
    /// Quantity in lots.
    pub quantity: u64,
    /// Order side (true = bid, false = ask).
    pub is_bid: bool,
}

impl OrderParams {
    /// Creates a new bid order.
    #[must_use]
    pub const fn bid(price: u64, quantity: u64) -> Self {
        Self {
            price,
            quantity,
            is_bid: true,
        }
    }

    /// Creates a new ask order.
    #[must_use]
    pub const fn ask(price: u64, quantity: u64) -> Self {
        Self {
            price,
            quantity,
            is_bid: false,
        }
    }
}

/// Standard test scenarios.
pub mod scenarios {
    use super::OrderParams;

    /// Simple matching scenario: one bid, one ask at same price.
    pub fn simple_match() -> (OrderParams, OrderParams) {
        (OrderParams::bid(1000, 10), OrderParams::ask(1000, 10))
    }

    /// Partial fill scenario: bid larger than ask.
    pub fn partial_fill_bid() -> (OrderParams, OrderParams) {
        (OrderParams::bid(1000, 20), OrderParams::ask(1000, 10))
    }

    /// Partial fill scenario: ask larger than bid.
    pub fn partial_fill_ask() -> (OrderParams, OrderParams) {
        (OrderParams::bid(1000, 10), OrderParams::ask(1000, 20))
    }

    /// Price improvement scenario: bid higher than ask.
    pub fn price_improvement() -> (OrderParams, OrderParams) {
        (OrderParams::bid(1100, 10), OrderParams::ask(1000, 10))
    }

    /// No match scenario: bid lower than ask.
    pub fn no_match() -> (OrderParams, OrderParams) {
        (OrderParams::bid(900, 10), OrderParams::ask(1000, 10))
    }

    /// Multiple orders on same side.
    pub fn order_book_depth() -> Vec<OrderParams> {
        vec![
            OrderParams::bid(1000, 10),
            OrderParams::bid(990, 20),
            OrderParams::bid(980, 30),
            OrderParams::ask(1010, 10),
            OrderParams::ask(1020, 20),
            OrderParams::ask(1030, 30),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_market_config_default() {
        let config = MarketConfig::default();
        assert_eq!(config.base_lot_size, DEFAULT_BASE_LOT_SIZE);
        assert_eq!(config.taker_fee_bps, DEFAULT_TAKER_FEE_BPS);
        assert_eq!(config.maker_fee_bps, DEFAULT_MAKER_FEE_BPS);
    }

    #[test]
    fn test_market_config_zero_fees() {
        let config = MarketConfig::zero_fees();
        assert_eq!(config.taker_fee_bps, 0);
        assert_eq!(config.maker_fee_bps, 0);
    }

    #[test]
    fn test_order_params() {
        let bid = OrderParams::bid(1000, 10);
        assert!(bid.is_bid);
        assert_eq!(bid.price, 1000);
        assert_eq!(bid.quantity, 10);

        let ask = OrderParams::ask(1000, 10);
        assert!(!ask.is_bid);
    }

    #[test]
    fn test_scenarios() {
        let (bid, ask) = scenarios::simple_match();
        assert_eq!(bid.price, ask.price);
        assert_eq!(bid.quantity, ask.quantity);

        let orders = scenarios::order_book_depth();
        assert_eq!(orders.len(), 6);
    }
}
