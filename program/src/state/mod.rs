//! State module containing all account structures for the Matchbook program.
//!
//! This module exports the core account types used by the on-chain program:
//!
//! - [`Market`] - Central market configuration and state
//! - [`MarketStatus`] - Market operational status enum
//! - [`OrderBookSideHeader`] - Order book header (bids/asks)
//! - [`LeafNode`], [`InnerNode`], [`FreeNode`] - Tree node types
//! - [`OrderId`] - Order identifier with price-time encoding
//! - [`TimeInForce`] - Order time-in-force options

mod market;
mod orderbook;

pub use market::{Market, MarketStatus, MARKET_SEED};
pub use orderbook::{
    critbit, get_bit, AnyNode, FreeNode, InnerNode, LeafNode, NodeTag, OrderBookSideHeader,
    OrderId, TimeInForce, ASKS_SEED, BIDS_SEED, NODE_SIZE, ORDERBOOK_HEADER_SIZE, SENTINEL,
};
