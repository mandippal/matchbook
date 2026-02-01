//! Request models for the API.

use serde::{Deserialize, Serialize};

/// Query parameters for order book requests.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct OrderBookQuery {
    /// Maximum depth per side (default: 20).
    pub depth: Option<usize>,
}

/// Query parameters for trades requests.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct TradesQuery {
    /// Maximum number of trades to return (default: 100).
    pub limit: Option<usize>,
    /// Cursor for pagination.
    pub cursor: Option<String>,
}

/// Query parameters for candles requests.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct CandlesQuery {
    /// Candle interval (1m, 5m, 15m, 1h, 4h, 1d).
    pub interval: Option<String>,
    /// Start time (Unix timestamp).
    pub start: Option<i64>,
    /// End time (Unix timestamp).
    pub end: Option<i64>,
    /// Maximum number of candles (default: 100).
    pub limit: Option<usize>,
}

/// Query parameters for user orders.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct OrdersQuery {
    /// Filter by market.
    pub market: Option<String>,
    /// Maximum number of orders (default: 100).
    pub limit: Option<usize>,
}

/// Query parameters for user trades.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct UserTradesQuery {
    /// Filter by market.
    pub market: Option<String>,
    /// Maximum number of trades (default: 100).
    pub limit: Option<usize>,
    /// Cursor for pagination.
    pub cursor: Option<String>,
}

/// Request body for placing an order.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PlaceOrderRequest {
    /// Market address (base58).
    pub market: String,
    /// Owner address (base58).
    pub owner: String,
    /// Order side ("buy" or "sell").
    pub side: String,
    /// Price in quote lots.
    pub price: u64,
    /// Quantity in base lots.
    pub quantity: u64,
    /// Client order ID.
    pub client_order_id: Option<u64>,
}

/// Request body for cancelling an order.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CancelOrderRequest {
    /// Market address (base58).
    pub market: String,
    /// Owner address (base58).
    pub owner: String,
    /// Order ID to cancel.
    pub order_id: u128,
}

/// Request body for deposit.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DepositRequest {
    /// Market address (base58).
    pub market: String,
    /// Owner address (base58).
    pub owner: String,
    /// Base token amount.
    pub base_amount: Option<u64>,
    /// Quote token amount.
    pub quote_amount: Option<u64>,
}

/// Request body for withdrawal.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct WithdrawRequest {
    /// Market address (base58).
    pub market: String,
    /// Owner address (base58).
    pub owner: String,
    /// Base token amount.
    pub base_amount: Option<u64>,
    /// Quote token amount.
    pub quote_amount: Option<u64>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_orderbook_query_default() {
        let query = OrderBookQuery::default();
        assert!(query.depth.is_none());
    }

    #[test]
    fn test_trades_query_default() {
        let query = TradesQuery::default();
        assert!(query.limit.is_none());
        assert!(query.cursor.is_none());
    }

    #[test]
    fn test_place_order_request() {
        let req = PlaceOrderRequest {
            market: "market123".to_string(),
            owner: "owner456".to_string(),
            side: "buy".to_string(),
            price: 1000,
            quantity: 100,
            client_order_id: Some(1),
        };

        assert_eq!(req.market, "market123");
        assert_eq!(req.side, "buy");
    }
}
