//! Account endpoint handlers.

use axum::extract::{Path, Query, State};
use axum::Json;

use crate::error::ApiError;
use crate::models::{
    BalancesResponse, OrdersQuery, OrdersResponse, TradesResponse, UserTradesQuery,
};
use crate::state::AppState;

/// Get user's open orders.
///
/// GET /v1/accounts/{owner}/orders
pub async fn get_user_orders(
    State(_state): State<AppState>,
    Path(owner): Path<String>,
    Query(query): Query<OrdersQuery>,
) -> Result<Json<OrdersResponse>, ApiError> {
    // Validate owner address
    if bs58::decode(&owner).into_vec().is_err() {
        return Err(ApiError::bad_request("Invalid owner address"));
    }

    let _limit = query.limit.unwrap_or(100);

    // TODO: Fetch from database
    Ok(Json(OrdersResponse { orders: vec![] }))
}

/// Get user's trade history.
///
/// GET /v1/accounts/{owner}/trades
pub async fn get_user_trades(
    State(_state): State<AppState>,
    Path(owner): Path<String>,
    Query(query): Query<UserTradesQuery>,
) -> Result<Json<TradesResponse>, ApiError> {
    // Validate owner address
    if bs58::decode(&owner).into_vec().is_err() {
        return Err(ApiError::bad_request("Invalid owner address"));
    }

    let _limit = query.limit.unwrap_or(100);

    // TODO: Fetch from database
    Ok(Json(TradesResponse {
        trades: vec![],
        next_cursor: None,
    }))
}

/// Get user's balances.
///
/// GET /v1/accounts/{owner}/balances
pub async fn get_user_balances(
    State(_state): State<AppState>,
    Path(owner): Path<String>,
) -> Result<Json<BalancesResponse>, ApiError> {
    // Validate owner address
    if bs58::decode(&owner).into_vec().is_err() {
        return Err(ApiError::bad_request("Invalid owner address"));
    }

    // TODO: Fetch from database
    Ok(Json(BalancesResponse {
        owner,
        balances: vec![],
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_get_user_orders_invalid_address() {
        let state = AppState::default();
        let query = OrdersQuery::default();
        let result =
            get_user_orders(State(state), Path("invalid!".to_string()), Query(query)).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_get_user_orders_empty() {
        let state = AppState::default();
        let owner = "11111111111111111111111111111111".to_string();
        let query = OrdersQuery::default();
        let result = get_user_orders(State(state), Path(owner), Query(query)).await;
        assert!(result.is_ok());
        assert!(result.expect("result").orders.is_empty());
    }

    #[tokio::test]
    async fn test_get_user_trades_empty() {
        let state = AppState::default();
        let owner = "11111111111111111111111111111111".to_string();
        let query = UserTradesQuery::default();
        let result = get_user_trades(State(state), Path(owner), Query(query)).await;
        assert!(result.is_ok());
        assert!(result.expect("result").trades.is_empty());
    }

    #[tokio::test]
    async fn test_get_user_balances_empty() {
        let state = AppState::default();
        let owner = "11111111111111111111111111111111".to_string();
        let result = get_user_balances(State(state), Path(owner.clone())).await;
        assert!(result.is_ok());
        let balances = result.expect("result");
        assert_eq!(balances.owner, owner);
        assert!(balances.balances.is_empty());
    }
}
