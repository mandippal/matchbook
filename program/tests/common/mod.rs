//! Common test utilities for Matchbook integration tests.
//!
//! This module provides shared utilities, fixtures, and helper functions
//! for integration testing the Matchbook program.

pub mod fixtures;

use anchor_lang::prelude::Pubkey;

/// Default test market parameters.
pub const DEFAULT_BASE_LOT_SIZE: u64 = 1_000_000;
pub const DEFAULT_QUOTE_LOT_SIZE: u64 = 1_000;
pub const DEFAULT_TICK_SIZE: u64 = 100;
pub const DEFAULT_MIN_ORDER_SIZE: u64 = 1;
pub const DEFAULT_TAKER_FEE_BPS: u16 = 30;
pub const DEFAULT_MAKER_FEE_BPS: i16 = -10;

/// Derives the market PDA for given mints.
#[must_use]
pub fn derive_market_pda(program_id: &Pubkey, base_mint: &Pubkey, quote_mint: &Pubkey) -> Pubkey {
    let (pda, _bump) = Pubkey::find_program_address(
        &[b"market", base_mint.as_ref(), quote_mint.as_ref()],
        program_id,
    );
    pda
}

/// Derives the open orders PDA for a user in a market.
#[must_use]
pub fn derive_open_orders_pda(program_id: &Pubkey, market: &Pubkey, owner: &Pubkey) -> Pubkey {
    let (pda, _bump) = Pubkey::find_program_address(
        &[b"open_orders", market.as_ref(), owner.as_ref()],
        program_id,
    );
    pda
}

/// Derives the bids PDA for a market.
#[must_use]
pub fn derive_bids_pda(program_id: &Pubkey, market: &Pubkey) -> Pubkey {
    let (pda, _bump) = Pubkey::find_program_address(&[b"bids", market.as_ref()], program_id);
    pda
}

/// Derives the asks PDA for a market.
#[must_use]
pub fn derive_asks_pda(program_id: &Pubkey, market: &Pubkey) -> Pubkey {
    let (pda, _bump) = Pubkey::find_program_address(&[b"asks", market.as_ref()], program_id);
    pda
}

/// Derives the event queue PDA for a market.
#[must_use]
pub fn derive_event_queue_pda(program_id: &Pubkey, market: &Pubkey) -> Pubkey {
    let (pda, _bump) = Pubkey::find_program_address(&[b"event_queue", market.as_ref()], program_id);
    pda
}

/// Derives the base vault PDA for a market.
#[must_use]
pub fn derive_base_vault_pda(program_id: &Pubkey, market: &Pubkey) -> Pubkey {
    let (pda, _bump) = Pubkey::find_program_address(&[b"base_vault", market.as_ref()], program_id);
    pda
}

/// Derives the quote vault PDA for a market.
#[must_use]
pub fn derive_quote_vault_pda(program_id: &Pubkey, market: &Pubkey) -> Pubkey {
    let (pda, _bump) = Pubkey::find_program_address(&[b"quote_vault", market.as_ref()], program_id);
    pda
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_derive_market_pda() {
        let program_id = Pubkey::new_unique();
        let base_mint = Pubkey::new_unique();
        let quote_mint = Pubkey::new_unique();

        let pda = derive_market_pda(&program_id, &base_mint, &quote_mint);
        assert_ne!(pda, Pubkey::default());
    }

    #[test]
    fn test_derive_open_orders_pda() {
        let program_id = Pubkey::new_unique();
        let market = Pubkey::new_unique();
        let owner = Pubkey::new_unique();

        let pda = derive_open_orders_pda(&program_id, &market, &owner);
        assert_ne!(pda, Pubkey::default());
    }

    #[test]
    fn test_derive_orderbook_pdas() {
        let program_id = Pubkey::new_unique();
        let market = Pubkey::new_unique();

        let bids = derive_bids_pda(&program_id, &market);
        let asks = derive_asks_pda(&program_id, &market);

        assert_ne!(bids, Pubkey::default());
        assert_ne!(asks, Pubkey::default());
        assert_ne!(bids, asks);
    }

    #[test]
    fn test_derive_vault_pdas() {
        let program_id = Pubkey::new_unique();
        let market = Pubkey::new_unique();

        let base_vault = derive_base_vault_pda(&program_id, &market);
        let quote_vault = derive_quote_vault_pda(&program_id, &market);

        assert_ne!(base_vault, Pubkey::default());
        assert_ne!(quote_vault, Pubkey::default());
        assert_ne!(base_vault, quote_vault);
    }
}
