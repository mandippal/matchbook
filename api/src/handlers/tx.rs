//! Transaction building endpoint handlers.

use axum::extract::State;
use axum::Json;

use crate::error::ApiError;
use crate::models::{
    CancelOrderRequest, DepositRequest, PlaceOrderRequest, TransactionResponse, WithdrawRequest,
};
use crate::state::AppState;

/// Build a PlaceOrder transaction.
///
/// POST /v1/tx/place-order
pub async fn build_place_order(
    State(_state): State<AppState>,
    Json(request): Json<PlaceOrderRequest>,
) -> Result<Json<TransactionResponse>, ApiError> {
    // Validate market address
    if bs58::decode(&request.market).into_vec().is_err() {
        return Err(ApiError::bad_request("Invalid market address"));
    }

    // Validate owner address
    if bs58::decode(&request.owner).into_vec().is_err() {
        return Err(ApiError::bad_request("Invalid owner address"));
    }

    // Validate side
    let side = request.side.to_lowercase();
    if side != "buy" && side != "sell" {
        return Err(ApiError::validation("Side must be 'buy' or 'sell'"));
    }

    // Validate price and quantity
    if request.price == 0 {
        return Err(ApiError::validation("Price must be greater than 0"));
    }
    if request.quantity == 0 {
        return Err(ApiError::validation("Quantity must be greater than 0"));
    }

    // TODO: Build actual transaction
    Ok(Json(TransactionResponse {
        transaction: "base64_encoded_transaction".to_string(),
        message: "base64_encoded_message".to_string(),
    }))
}

/// Build a CancelOrder transaction.
///
/// POST /v1/tx/cancel-order
pub async fn build_cancel_order(
    State(_state): State<AppState>,
    Json(request): Json<CancelOrderRequest>,
) -> Result<Json<TransactionResponse>, ApiError> {
    // Validate market address
    if bs58::decode(&request.market).into_vec().is_err() {
        return Err(ApiError::bad_request("Invalid market address"));
    }

    // Validate owner address
    if bs58::decode(&request.owner).into_vec().is_err() {
        return Err(ApiError::bad_request("Invalid owner address"));
    }

    // TODO: Build actual transaction
    Ok(Json(TransactionResponse {
        transaction: "base64_encoded_transaction".to_string(),
        message: "base64_encoded_message".to_string(),
    }))
}

/// Build a Deposit transaction.
///
/// POST /v1/tx/deposit
pub async fn build_deposit(
    State(_state): State<AppState>,
    Json(request): Json<DepositRequest>,
) -> Result<Json<TransactionResponse>, ApiError> {
    // Validate market address
    if bs58::decode(&request.market).into_vec().is_err() {
        return Err(ApiError::bad_request("Invalid market address"));
    }

    // Validate owner address
    if bs58::decode(&request.owner).into_vec().is_err() {
        return Err(ApiError::bad_request("Invalid owner address"));
    }

    // Validate amounts
    let base = request.base_amount.unwrap_or(0);
    let quote = request.quote_amount.unwrap_or(0);
    if base == 0 && quote == 0 {
        return Err(ApiError::validation(
            "At least one of base_amount or quote_amount must be greater than 0",
        ));
    }

    // TODO: Build actual transaction
    Ok(Json(TransactionResponse {
        transaction: "base64_encoded_transaction".to_string(),
        message: "base64_encoded_message".to_string(),
    }))
}

/// Build a Withdraw transaction.
///
/// POST /v1/tx/withdraw
pub async fn build_withdraw(
    State(_state): State<AppState>,
    Json(request): Json<WithdrawRequest>,
) -> Result<Json<TransactionResponse>, ApiError> {
    // Validate market address
    if bs58::decode(&request.market).into_vec().is_err() {
        return Err(ApiError::bad_request("Invalid market address"));
    }

    // Validate owner address
    if bs58::decode(&request.owner).into_vec().is_err() {
        return Err(ApiError::bad_request("Invalid owner address"));
    }

    // Validate amounts
    let base = request.base_amount.unwrap_or(0);
    let quote = request.quote_amount.unwrap_or(0);
    if base == 0 && quote == 0 {
        return Err(ApiError::validation(
            "At least one of base_amount or quote_amount must be greater than 0",
        ));
    }

    // TODO: Build actual transaction
    Ok(Json(TransactionResponse {
        transaction: "base64_encoded_transaction".to_string(),
        message: "base64_encoded_message".to_string(),
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_build_place_order_valid() {
        let state = AppState::default();
        let request = PlaceOrderRequest {
            market: "11111111111111111111111111111111".to_string(),
            owner: "11111111111111111111111111111111".to_string(),
            side: "buy".to_string(),
            price: 1000,
            quantity: 100,
            client_order_id: None,
        };
        let result = build_place_order(State(state), Json(request)).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_build_place_order_invalid_market() {
        let state = AppState::default();
        let request = PlaceOrderRequest {
            market: "invalid!".to_string(),
            owner: "11111111111111111111111111111111".to_string(),
            side: "buy".to_string(),
            price: 1000,
            quantity: 100,
            client_order_id: None,
        };
        let result = build_place_order(State(state), Json(request)).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_build_place_order_invalid_side() {
        let state = AppState::default();
        let request = PlaceOrderRequest {
            market: "11111111111111111111111111111111".to_string(),
            owner: "11111111111111111111111111111111".to_string(),
            side: "invalid".to_string(),
            price: 1000,
            quantity: 100,
            client_order_id: None,
        };
        let result = build_place_order(State(state), Json(request)).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_build_place_order_zero_price() {
        let state = AppState::default();
        let request = PlaceOrderRequest {
            market: "11111111111111111111111111111111".to_string(),
            owner: "11111111111111111111111111111111".to_string(),
            side: "buy".to_string(),
            price: 0,
            quantity: 100,
            client_order_id: None,
        };
        let result = build_place_order(State(state), Json(request)).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_build_cancel_order_valid() {
        let state = AppState::default();
        let request = CancelOrderRequest {
            market: "11111111111111111111111111111111".to_string(),
            owner: "11111111111111111111111111111111".to_string(),
            order_id: 12345,
        };
        let result = build_cancel_order(State(state), Json(request)).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_build_deposit_valid() {
        let state = AppState::default();
        let request = DepositRequest {
            market: "11111111111111111111111111111111".to_string(),
            owner: "11111111111111111111111111111111".to_string(),
            base_amount: Some(1000),
            quote_amount: None,
        };
        let result = build_deposit(State(state), Json(request)).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_build_deposit_zero_amounts() {
        let state = AppState::default();
        let request = DepositRequest {
            market: "11111111111111111111111111111111".to_string(),
            owner: "11111111111111111111111111111111".to_string(),
            base_amount: None,
            quote_amount: None,
        };
        let result = build_deposit(State(state), Json(request)).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_build_withdraw_valid() {
        let state = AppState::default();
        let request = WithdrawRequest {
            market: "11111111111111111111111111111111".to_string(),
            owner: "11111111111111111111111111111111".to_string(),
            base_amount: None,
            quote_amount: Some(500),
        };
        let result = build_withdraw(State(state), Json(request)).await;
        assert!(result.is_ok());
    }
}
