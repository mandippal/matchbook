//! Application state for the API server.
//!
//! Provides shared state across all request handlers.

use std::sync::Arc;

use matchbook_indexer::{BookBuilder, EventProcessor};
use tokio::sync::RwLock;

/// Application state shared across handlers.
#[derive(Clone)]
pub struct AppState {
    /// Book builder for order book data.
    pub book_builder: Arc<RwLock<BookBuilder>>,

    /// Event processor for trade data.
    pub event_processor: Arc<RwLock<EventProcessor>>,
}

impl AppState {
    /// Creates a new application state.
    #[must_use]
    pub fn new(book_builder: BookBuilder, event_processor: EventProcessor) -> Self {
        Self {
            book_builder: Arc::new(RwLock::new(book_builder)),
            event_processor: Arc::new(RwLock::new(event_processor)),
        }
    }

    /// Creates a default application state for testing.
    #[must_use]
    pub fn default_for_testing() -> Self {
        Self::new(BookBuilder::new(), EventProcessor::new())
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new(BookBuilder::new(), EventProcessor::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_state_new() {
        let state = AppState::new(BookBuilder::new(), EventProcessor::new());
        assert!(Arc::strong_count(&state.book_builder) == 1);
    }

    #[test]
    fn test_app_state_default() {
        let state = AppState::default();
        assert!(Arc::strong_count(&state.book_builder) == 1);
    }

    #[test]
    fn test_app_state_clone() {
        let state = AppState::default();
        let cloned = state.clone();
        assert!(Arc::strong_count(&state.book_builder) == 2);
        assert!(Arc::strong_count(&cloned.book_builder) == 2);
    }
}
