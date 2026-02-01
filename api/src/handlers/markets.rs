//! Market endpoint handlers.

use axum::extract::{Path, Query, State};
use axum::Json;

use crate::error::ApiError;
use crate::models::{
    CandlesQuery, CandlesResponse, MarketResponse, MarketsResponse, OrderBookQuery,
    OrderBookResponse, PriceLevelResponse, TradesQuery, TradesResponse,
};
use crate::state::AppState;

/// List all markets.
///
/// GET /v1/markets
pub async fn list_markets(
    State(_state): State<AppState>,
) -> Result<Json<MarketsResponse>, ApiError> {
    // TODO: Fetch from database
    Ok(Json(MarketsResponse { markets: vec![] }))
}

/// Get market details.
///
/// GET /v1/markets/{market}
pub async fn get_market(
    State(_state): State<AppState>,
    Path(market): Path<String>,
) -> Result<Json<MarketResponse>, ApiError> {
    // Validate market address
    if bs58::decode(&market).into_vec().is_err() {
        return Err(ApiError::bad_request("Invalid market address"));
    }

    // TODO: Fetch from database
    Err(ApiError::not_found(format!("Market {} not found", market)))
}

/// Get order book snapshot.
///
/// GET /v1/markets/{market}/orderbook
pub async fn get_orderbook(
    State(state): State<AppState>,
    Path(market): Path<String>,
    Query(query): Query<OrderBookQuery>,
) -> Result<Json<OrderBookResponse>, ApiError> {
    // Validate market address
    let market_bytes: [u8; 32] = bs58::decode(&market)
        .into_vec()
        .map_err(|_| ApiError::bad_request("Invalid market address"))?
        .try_into()
        .map_err(|_| ApiError::bad_request("Invalid market address length"))?;

    let depth = query.depth.unwrap_or(20);

    // Get order book from builder
    let book_builder = state.book_builder.read().await;

    if let Some(snapshot) = book_builder.get_snapshot(&market_bytes, depth) {
        match snapshot {
            matchbook_indexer::book::BookUpdate::Snapshot {
                bids,
                asks,
                slot,
                seq,
                ..
            } => {
                let bids: Vec<PriceLevelResponse> = bids
                    .into_iter()
                    .map(|l| PriceLevelResponse {
                        price: l.price,
                        quantity: l.quantity,
                        order_count: l.order_count,
                    })
                    .collect();

                let asks: Vec<PriceLevelResponse> = asks
                    .into_iter()
                    .map(|l| PriceLevelResponse {
                        price: l.price,
                        quantity: l.quantity,
                        order_count: l.order_count,
                    })
                    .collect();

                Ok(Json(OrderBookResponse {
                    market,
                    bids,
                    asks,
                    slot,
                    seq,
                }))
            }
            _ => Err(ApiError::internal("Unexpected snapshot type")),
        }
    } else {
        // Return empty order book if market not found
        Ok(Json(OrderBookResponse {
            market,
            bids: vec![],
            asks: vec![],
            slot: 0,
            seq: 0,
        }))
    }
}

/// Get recent trades.
///
/// GET /v1/markets/{market}/trades
pub async fn get_trades(
    State(_state): State<AppState>,
    Path(market): Path<String>,
    Query(query): Query<TradesQuery>,
) -> Result<Json<TradesResponse>, ApiError> {
    // Validate market address
    if bs58::decode(&market).into_vec().is_err() {
        return Err(ApiError::bad_request("Invalid market address"));
    }

    let _limit = query.limit.unwrap_or(100);

    // TODO: Fetch from database
    Ok(Json(TradesResponse {
        trades: vec![],
        next_cursor: None,
    }))
}

/// Get OHLCV candles.
///
/// GET /v1/markets/{market}/candles
pub async fn get_candles(
    State(_state): State<AppState>,
    Path(market): Path<String>,
    Query(query): Query<CandlesQuery>,
) -> Result<Json<CandlesResponse>, ApiError> {
    // Validate market address
    if bs58::decode(&market).into_vec().is_err() {
        return Err(ApiError::bad_request("Invalid market address"));
    }

    let interval = query.interval.unwrap_or_else(|| "1h".to_string());

    // Validate interval
    let valid_intervals = ["1m", "5m", "15m", "1h", "4h", "1d"];
    if !valid_intervals.contains(&interval.as_str()) {
        return Err(ApiError::validation(format!(
            "Invalid interval: {}. Valid intervals: {:?}",
            interval, valid_intervals
        )));
    }

    // TODO: Fetch from database
    Ok(Json(CandlesResponse {
        market,
        interval,
        candles: vec![],
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_list_markets() {
        let state = AppState::default();
        let result = list_markets(State(state)).await;
        assert!(result.is_ok());
        assert!(result.expect("result").markets.is_empty());
    }

    #[tokio::test]
    async fn test_get_market_invalid_address() {
        let state = AppState::default();
        let result = get_market(State(state), Path("invalid!".to_string())).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_get_orderbook_empty() {
        let state = AppState::default();
        let market = "11111111111111111111111111111111".to_string();
        let query = OrderBookQuery::default();
        let result = get_orderbook(State(state), Path(market), Query(query)).await;
        assert!(result.is_ok());
        let book = result.expect("result");
        assert!(book.bids.is_empty());
        assert!(book.asks.is_empty());
    }

    #[tokio::test]
    async fn test_get_trades_empty() {
        let state = AppState::default();
        let market = "11111111111111111111111111111111".to_string();
        let query = TradesQuery::default();
        let result = get_trades(State(state), Path(market), Query(query)).await;
        assert!(result.is_ok());
        assert!(result.expect("result").trades.is_empty());
    }

    #[tokio::test]
    async fn test_get_candles_invalid_interval() {
        let state = AppState::default();
        let market = "11111111111111111111111111111111".to_string();
        let query = CandlesQuery {
            interval: Some("invalid".to_string()),
            ..Default::default()
        };
        let result = get_candles(State(state), Path(market), Query(query)).await;
        assert!(result.is_err());
    }
}
