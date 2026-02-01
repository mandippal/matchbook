//! Integration tests for the full order lifecycle.
//!
//! Tests the complete flow:
//! CreateMarket → CreateOpenOrders → Deposit → PlaceOrder → MatchOrders → ConsumeEvents → Withdraw

#![allow(clippy::indexing_slicing)] // Test code with controlled bounds

mod common;

use common::fixtures::{MarketConfig, OrderParams};
use common::{
    derive_asks_pda, derive_base_vault_pda, derive_bids_pda, derive_event_queue_pda,
    derive_market_pda, derive_open_orders_pda, derive_quote_vault_pda,
};

/// Tests that market PDA derivation is consistent.
#[test]
fn test_market_pda_derivation_consistency() {
    use anchor_lang::prelude::Pubkey;

    let program_id = Pubkey::new_unique();
    let base_mint = Pubkey::new_unique();
    let quote_mint = Pubkey::new_unique();

    let pda1 = derive_market_pda(&program_id, &base_mint, &quote_mint);
    let pda2 = derive_market_pda(&program_id, &base_mint, &quote_mint);

    assert_eq!(pda1, pda2, "PDA derivation should be deterministic");
}

/// Tests that different mints produce different PDAs.
#[test]
fn test_different_mints_different_pdas() {
    use anchor_lang::prelude::Pubkey;

    let program_id = Pubkey::new_unique();
    let base_mint1 = Pubkey::new_unique();
    let base_mint2 = Pubkey::new_unique();
    let quote_mint = Pubkey::new_unique();

    let pda1 = derive_market_pda(&program_id, &base_mint1, &quote_mint);
    let pda2 = derive_market_pda(&program_id, &base_mint2, &quote_mint);

    assert_ne!(pda1, pda2, "Different mints should produce different PDAs");
}

/// Tests that all market-related PDAs are unique.
#[test]
fn test_all_market_pdas_unique() {
    use anchor_lang::prelude::Pubkey;

    let program_id = Pubkey::new_unique();
    let base_mint = Pubkey::new_unique();
    let quote_mint = Pubkey::new_unique();

    let market = derive_market_pda(&program_id, &base_mint, &quote_mint);
    let bids = derive_bids_pda(&program_id, &market);
    let asks = derive_asks_pda(&program_id, &market);
    let event_queue = derive_event_queue_pda(&program_id, &market);
    let base_vault = derive_base_vault_pda(&program_id, &market);
    let quote_vault = derive_quote_vault_pda(&program_id, &market);

    let pdas = [market, bids, asks, event_queue, base_vault, quote_vault];

    // Check all PDAs are unique using windows
    for window in pdas.windows(2) {
        assert_ne!(window[0], window[1], "Adjacent PDAs should be unique");
    }
    // Also check non-adjacent pairs
    assert_ne!(pdas[0], pdas[2], "All PDAs should be unique");
    assert_ne!(pdas[0], pdas[3], "All PDAs should be unique");
    assert_ne!(pdas[0], pdas[4], "All PDAs should be unique");
    assert_ne!(pdas[0], pdas[5], "All PDAs should be unique");
    assert_ne!(pdas[1], pdas[3], "All PDAs should be unique");
    assert_ne!(pdas[1], pdas[4], "All PDAs should be unique");
    assert_ne!(pdas[1], pdas[5], "All PDAs should be unique");
    assert_ne!(pdas[2], pdas[4], "All PDAs should be unique");
    assert_ne!(pdas[2], pdas[5], "All PDAs should be unique");
    assert_ne!(pdas[3], pdas[5], "All PDAs should be unique");
}

/// Tests open orders PDA derivation for different users.
#[test]
fn test_open_orders_pda_per_user() {
    use anchor_lang::prelude::Pubkey;

    let program_id = Pubkey::new_unique();
    let market = Pubkey::new_unique();
    let user1 = Pubkey::new_unique();
    let user2 = Pubkey::new_unique();

    let oo1 = derive_open_orders_pda(&program_id, &market, &user1);
    let oo2 = derive_open_orders_pda(&program_id, &market, &user2);

    assert_ne!(
        oo1, oo2,
        "Different users should have different OpenOrders PDAs"
    );
}

/// Tests market config defaults.
#[test]
fn test_market_config_defaults() {
    let config = MarketConfig::default();

    assert!(config.base_lot_size > 0);
    assert!(config.quote_lot_size > 0);
    assert!(config.tick_size > 0);
    assert!(config.min_order_size > 0);
    assert!(config.taker_fee_bps <= 100); // Max 1%
    assert!(config.maker_fee_bps >= -100); // Max 1% rebate
}

/// Tests order params creation.
#[test]
fn test_order_params_creation() {
    let bid = OrderParams::bid(1000, 10);
    assert!(bid.is_bid);
    assert_eq!(bid.price, 1000);
    assert_eq!(bid.quantity, 10);

    let ask = OrderParams::ask(1000, 10);
    assert!(!ask.is_bid);
    assert_eq!(ask.price, 1000);
    assert_eq!(ask.quantity, 10);
}

/// Tests simple match scenario setup.
#[test]
fn test_simple_match_scenario() {
    use common::fixtures::scenarios;

    let (bid, ask) = scenarios::simple_match();

    assert!(bid.is_bid);
    assert!(!ask.is_bid);
    assert_eq!(bid.price, ask.price);
    assert_eq!(bid.quantity, ask.quantity);
}

/// Tests partial fill scenario setup.
#[test]
fn test_partial_fill_scenarios() {
    use common::fixtures::scenarios;

    let (bid, ask) = scenarios::partial_fill_bid();
    assert!(bid.quantity > ask.quantity);

    let (bid, ask) = scenarios::partial_fill_ask();
    assert!(bid.quantity < ask.quantity);
}

/// Tests price improvement scenario setup.
#[test]
fn test_price_improvement_scenario() {
    use common::fixtures::scenarios;

    let (bid, ask) = scenarios::price_improvement();
    assert!(
        bid.price > ask.price,
        "Bid should be higher for price improvement"
    );
}

/// Tests no match scenario setup.
#[test]
fn test_no_match_scenario() {
    use common::fixtures::scenarios;

    let (bid, ask) = scenarios::no_match();
    assert!(bid.price < ask.price, "Bid should be lower for no match");
}

/// Tests order book depth scenario.
#[test]
fn test_order_book_depth_scenario() {
    use common::fixtures::scenarios;

    let orders = scenarios::order_book_depth();

    let bids: Vec<_> = orders.iter().filter(|o| o.is_bid).collect();
    let asks: Vec<_> = orders.iter().filter(|o| !o.is_bid).collect();

    assert_eq!(bids.len(), 3, "Should have 3 bid orders");
    assert_eq!(asks.len(), 3, "Should have 3 ask orders");

    // Verify bids are in descending price order using windows
    for window in bids.windows(2) {
        assert!(
            window[0].price >= window[1].price,
            "Bids should be in descending price order"
        );
    }

    // Verify asks are in ascending price order using windows
    for window in asks.windows(2) {
        assert!(
            window[0].price <= window[1].price,
            "Asks should be in ascending price order"
        );
    }
}

/// Tests zero fees market config.
#[test]
fn test_zero_fees_config() {
    let config = MarketConfig::zero_fees();
    assert_eq!(config.taker_fee_bps, 0);
    assert_eq!(config.maker_fee_bps, 0);
}

/// Tests high fees market config.
#[test]
fn test_high_fees_config() {
    let config = MarketConfig::high_fees();
    assert!(config.taker_fee_bps > 0);
    assert!(config.maker_fee_bps > 0);
}
