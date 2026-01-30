//! State module containing all account structures for the Matchbook program.
//!
//! This module exports the core account types used by the on-chain program:
//!
//! - [`Market`] - Central market configuration and state
//! - [`MarketStatus`] - Market operational status enum

mod market;

pub use market::{Market, MarketStatus, MARKET_SEED};
