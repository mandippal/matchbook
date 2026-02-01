//! API error types and responses.
//!
//! Provides consistent error handling across all endpoints.

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::{Deserialize, Serialize};

/// API error response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiError {
    /// Error code for programmatic handling.
    pub code: String,
    /// Human-readable error message.
    pub message: String,
    /// HTTP status code (not serialized).
    #[serde(skip)]
    pub status: StatusCode,
}

impl ApiError {
    /// Creates a new API error.
    #[must_use]
    pub fn new(code: impl Into<String>, message: impl Into<String>, status: StatusCode) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
            status,
        }
    }

    /// Creates a bad request error.
    #[must_use]
    pub fn bad_request(message: impl Into<String>) -> Self {
        Self::new("BAD_REQUEST", message, StatusCode::BAD_REQUEST)
    }

    /// Creates a not found error.
    #[must_use]
    pub fn not_found(message: impl Into<String>) -> Self {
        Self::new("NOT_FOUND", message, StatusCode::NOT_FOUND)
    }

    /// Creates an internal server error.
    #[must_use]
    pub fn internal(message: impl Into<String>) -> Self {
        Self::new("INTERNAL_ERROR", message, StatusCode::INTERNAL_SERVER_ERROR)
    }

    /// Creates a validation error.
    #[must_use]
    pub fn validation(message: impl Into<String>) -> Self {
        Self::new(
            "VALIDATION_ERROR",
            message,
            StatusCode::UNPROCESSABLE_ENTITY,
        )
    }

    /// Creates a rate limit error.
    #[must_use]
    pub fn rate_limited() -> Self {
        Self::new(
            "RATE_LIMITED",
            "Too many requests",
            StatusCode::TOO_MANY_REQUESTS,
        )
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let status = self.status;
        let body = Json(ErrorResponse {
            error: ErrorBody {
                code: self.code,
                message: self.message,
            },
        });
        (status, body).into_response()
    }
}

impl std::fmt::Display for ApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.code, self.message)
    }
}

impl std::error::Error for ApiError {}

/// Error response wrapper.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorResponse {
    /// Error details.
    pub error: ErrorBody,
}

/// Error body.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorBody {
    /// Error code.
    pub code: String,
    /// Error message.
    pub message: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_error_new() {
        let err = ApiError::new("TEST", "Test message", StatusCode::BAD_REQUEST);
        assert_eq!(err.code, "TEST");
        assert_eq!(err.message, "Test message");
        assert_eq!(err.status, StatusCode::BAD_REQUEST);
    }

    #[test]
    fn test_api_error_bad_request() {
        let err = ApiError::bad_request("Invalid input");
        assert_eq!(err.code, "BAD_REQUEST");
        assert_eq!(err.status, StatusCode::BAD_REQUEST);
    }

    #[test]
    fn test_api_error_not_found() {
        let err = ApiError::not_found("Market not found");
        assert_eq!(err.code, "NOT_FOUND");
        assert_eq!(err.status, StatusCode::NOT_FOUND);
    }

    #[test]
    fn test_api_error_internal() {
        let err = ApiError::internal("Something went wrong");
        assert_eq!(err.code, "INTERNAL_ERROR");
        assert_eq!(err.status, StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[test]
    fn test_api_error_validation() {
        let err = ApiError::validation("Invalid price");
        assert_eq!(err.code, "VALIDATION_ERROR");
        assert_eq!(err.status, StatusCode::UNPROCESSABLE_ENTITY);
    }

    #[test]
    fn test_api_error_rate_limited() {
        let err = ApiError::rate_limited();
        assert_eq!(err.code, "RATE_LIMITED");
        assert_eq!(err.status, StatusCode::TOO_MANY_REQUESTS);
    }

    #[test]
    fn test_api_error_display() {
        let err = ApiError::bad_request("Invalid input");
        assert_eq!(err.to_string(), "BAD_REQUEST: Invalid input");
    }
}
