//! OpenOrders account structure for tracking user orders and balances.
//!
//! Each user needs one OpenOrders account per market they trade on. This account
//! tracks their open orders and token balances (locked and free).
//!
//! # Structure
//!
//! - [`OpenOrders`] - Main account structure
//! - [`OrderSlot`] - Individual order slot within OpenOrders
//!
//! # PDA Derivation
//!
//! OpenOrders: `["open_orders", market, owner]`
//!
//! # Balance Invariants
//!
//! - `base_locked + base_free = total_base_deposited - total_base_withdrawn`
//! - `quote_locked + quote_free = total_quote_deposited - total_quote_withdrawn`

use anchor_lang::prelude::*;

use super::Side;

/// Seed prefix for OpenOrders PDA derivation.
pub const OPEN_ORDERS_SEED: &[u8] = b"open_orders";

/// Maximum number of orders per OpenOrders account.
pub const MAX_ORDERS: usize = 128;

/// Sentinel value for empty order slot.
pub const EMPTY_ORDER_ID: u128 = 0;

// ============================================================================
// Order Slot
// ============================================================================

/// Order slot in OpenOrders account.
///
/// Each slot stores minimal information about an active order for O(1) lookup.
/// The slot index is stored in the LeafNode for quick cancellation.
#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone, Copy, PartialEq, Eq, InitSpace)]
pub struct OrderSlot {
    /// Order ID (0 = empty slot).
    pub order_id: u128,
    /// Client-provided order ID.
    pub client_order_id: u64,
    /// Side of the order (bid/ask).
    pub side: Side,
}

impl Default for OrderSlot {
    fn default() -> Self {
        Self {
            order_id: EMPTY_ORDER_ID,
            client_order_id: 0,
            side: Side::Bid,
        }
    }
}

impl OrderSlot {
    /// Creates a new order slot.
    #[must_use]
    pub const fn new(order_id: u128, client_order_id: u64, side: Side) -> Self {
        Self {
            order_id,
            client_order_id,
            side,
        }
    }

    /// Returns true if this slot is empty.
    #[must_use]
    #[inline]
    pub const fn is_empty(&self) -> bool {
        self.order_id == EMPTY_ORDER_ID
    }

    /// Clears the slot.
    #[inline]
    pub fn clear(&mut self) {
        self.order_id = EMPTY_ORDER_ID;
        self.client_order_id = 0;
        self.side = Side::Bid;
    }
}

// ============================================================================
// OpenOrders Account
// ============================================================================

/// OpenOrders account tracking a user's orders and balances in a market.
///
/// Each user needs one OpenOrders account per market they trade on.
/// The account stores:
/// - Token balances (locked in orders vs free for withdrawal)
/// - Active order slots for O(1) lookup
/// - Optional delegate for third-party trading
///
/// # PDA Seeds
///
/// `["open_orders", market.key(), owner.key()]`
#[account]
#[derive(Debug, InitSpace)]
pub struct OpenOrders {
    /// Bump seed for PDA derivation.
    pub bump: u8,

    /// Market this account is for.
    pub market: Pubkey,

    /// Owner wallet (authority).
    pub owner: Pubkey,

    /// Delegate that can place/cancel orders (Pubkey::default() = no delegate).
    pub delegate: Pubkey,

    /// Base tokens locked in open orders.
    pub base_locked: u64,

    /// Quote tokens locked in open orders.
    pub quote_locked: u64,

    /// Base tokens available for withdrawal.
    pub base_free: u64,

    /// Quote tokens available for withdrawal.
    pub quote_free: u64,

    /// Accumulated referrer rebates.
    pub referrer_rebates: u64,

    /// Number of active orders.
    pub num_orders: u8,

    /// Reserved for future use.
    #[max_len(64)]
    pub reserved: [u8; 64],

    /// Order slots (fixed size array).
    #[max_len(MAX_ORDERS)]
    pub orders: [OrderSlot; MAX_ORDERS],
}

impl OpenOrders {
    /// Seed prefix for PDA derivation.
    pub const SEED_PREFIX: &'static [u8] = OPEN_ORDERS_SEED;

    /// Derives the PDA for an OpenOrders account.
    ///
    /// # Arguments
    ///
    /// * `market` - The market pubkey
    /// * `owner` - The owner's wallet pubkey
    /// * `program_id` - The program ID
    #[must_use]
    pub fn derive_pda(market: &Pubkey, owner: &Pubkey, program_id: &Pubkey) -> (Pubkey, u8) {
        Pubkey::find_program_address(
            &[Self::SEED_PREFIX, market.as_ref(), owner.as_ref()],
            program_id,
        )
    }

    /// Returns true if the signer is authorized to act on this account.
    ///
    /// Authorization is granted to the owner or the delegate (if set).
    ///
    /// # Arguments
    ///
    /// * `signer` - The signer's pubkey
    #[must_use]
    #[inline]
    pub fn is_authorized(&self, signer: &Pubkey) -> bool {
        *signer == self.owner || (self.delegate != Pubkey::default() && *signer == self.delegate)
    }

    /// Returns true if a delegate is set.
    #[must_use]
    #[inline]
    pub fn has_delegate(&self) -> bool {
        self.delegate != Pubkey::default()
    }

    /// Sets the delegate.
    ///
    /// # Arguments
    ///
    /// * `delegate` - The delegate's pubkey (or Pubkey::default() to clear)
    #[inline]
    pub fn set_delegate(&mut self, delegate: Pubkey) {
        self.delegate = delegate;
    }

    /// Clears the delegate.
    #[inline]
    pub fn clear_delegate(&mut self) {
        self.delegate = Pubkey::default();
    }

    // ========================================================================
    // Order Slot Management
    // ========================================================================

    /// Finds a free order slot.
    ///
    /// # Returns
    ///
    /// The index of a free slot, or None if all slots are full.
    #[must_use]
    pub fn find_free_slot(&self) -> Option<u8> {
        for (i, slot) in self.orders.iter().enumerate() {
            if slot.is_empty() {
                return Some(i as u8);
            }
        }
        None
    }

    /// Adds an order to a specific slot.
    ///
    /// # Arguments
    ///
    /// * `slot_index` - The slot index to use
    /// * `order_id` - The order ID
    /// * `client_order_id` - The client order ID
    /// * `side` - The order side
    ///
    /// # Returns
    ///
    /// `true` if successful, `false` if slot is out of bounds or already occupied.
    pub fn add_order(
        &mut self,
        slot_index: u8,
        order_id: u128,
        client_order_id: u64,
        side: Side,
    ) -> bool {
        let idx = slot_index as usize;
        if idx >= MAX_ORDERS {
            return false;
        }

        // Check if slot is already occupied
        if let Some(slot) = self.orders.get(idx) {
            if !slot.is_empty() {
                return false;
            }
        }

        if let Some(slot) = self.orders.get_mut(idx) {
            *slot = OrderSlot::new(order_id, client_order_id, side);
            self.num_orders = self.num_orders.saturating_add(1);
            true
        } else {
            false
        }
    }

    /// Removes an order from a specific slot.
    ///
    /// # Arguments
    ///
    /// * `slot_index` - The slot index to clear
    ///
    /// # Returns
    ///
    /// The removed order slot, or None if slot is out of bounds or empty.
    pub fn remove_order(&mut self, slot_index: u8) -> Option<OrderSlot> {
        let idx = slot_index as usize;
        if idx >= MAX_ORDERS {
            return None;
        }

        let slot = self.orders.get_mut(idx)?;
        if slot.is_empty() {
            return None;
        }

        let removed = *slot;
        slot.clear();
        self.num_orders = self.num_orders.saturating_sub(1);
        Some(removed)
    }

    /// Gets an order slot by index.
    ///
    /// # Arguments
    ///
    /// * `slot_index` - The slot index
    ///
    /// # Returns
    ///
    /// A reference to the order slot, or None if out of bounds.
    #[must_use]
    pub fn get_order(&self, slot_index: u8) -> Option<&OrderSlot> {
        self.orders.get(slot_index as usize)
    }

    /// Finds an order by order ID.
    ///
    /// # Arguments
    ///
    /// * `order_id` - The order ID to find
    ///
    /// # Returns
    ///
    /// The slot index if found, or None.
    #[must_use]
    pub fn find_order(&self, order_id: u128) -> Option<u8> {
        for (i, slot) in self.orders.iter().enumerate() {
            if slot.order_id == order_id {
                return Some(i as u8);
            }
        }
        None
    }

    /// Returns the number of active orders.
    #[must_use]
    #[inline]
    pub const fn order_count(&self) -> u8 {
        self.num_orders
    }

    /// Returns true if there are no active orders.
    #[must_use]
    #[inline]
    pub const fn has_no_orders(&self) -> bool {
        self.num_orders == 0
    }

    /// Returns true if all order slots are full.
    #[must_use]
    #[inline]
    pub const fn is_full(&self) -> bool {
        self.num_orders as usize >= MAX_ORDERS
    }

    // ========================================================================
    // Balance Management
    // ========================================================================

    /// Returns the total base balance (locked + free).
    #[must_use]
    #[inline]
    pub const fn total_base(&self) -> u64 {
        self.base_locked.saturating_add(self.base_free)
    }

    /// Returns the total quote balance (locked + free).
    #[must_use]
    #[inline]
    pub const fn total_quote(&self) -> u64 {
        self.quote_locked.saturating_add(self.quote_free)
    }

    /// Locks base tokens for an ask order.
    ///
    /// # Arguments
    ///
    /// * `amount` - Amount to lock
    ///
    /// # Returns
    ///
    /// `true` if successful, `false` if insufficient free balance.
    pub fn lock_base(&mut self, amount: u64) -> bool {
        if self.base_free < amount {
            return false;
        }
        self.base_free = self.base_free.saturating_sub(amount);
        self.base_locked = self.base_locked.saturating_add(amount);
        true
    }

    /// Locks quote tokens for a bid order.
    ///
    /// # Arguments
    ///
    /// * `amount` - Amount to lock
    ///
    /// # Returns
    ///
    /// `true` if successful, `false` if insufficient free balance.
    pub fn lock_quote(&mut self, amount: u64) -> bool {
        if self.quote_free < amount {
            return false;
        }
        self.quote_free = self.quote_free.saturating_sub(amount);
        self.quote_locked = self.quote_locked.saturating_add(amount);
        true
    }

    /// Releases locked base tokens back to free.
    ///
    /// # Arguments
    ///
    /// * `amount` - Amount to release
    ///
    /// # Returns
    ///
    /// `true` if successful, `false` if insufficient locked balance.
    pub fn release_base(&mut self, amount: u64) -> bool {
        if self.base_locked < amount {
            return false;
        }
        self.base_locked = self.base_locked.saturating_sub(amount);
        self.base_free = self.base_free.saturating_add(amount);
        true
    }

    /// Releases locked quote tokens back to free.
    ///
    /// # Arguments
    ///
    /// * `amount` - Amount to release
    ///
    /// # Returns
    ///
    /// `true` if successful, `false` if insufficient locked balance.
    pub fn release_quote(&mut self, amount: u64) -> bool {
        if self.quote_locked < amount {
            return false;
        }
        self.quote_locked = self.quote_locked.saturating_sub(amount);
        self.quote_free = self.quote_free.saturating_add(amount);
        true
    }

    /// Credits base tokens to free balance (from deposit or fill).
    ///
    /// # Arguments
    ///
    /// * `amount` - Amount to credit
    #[inline]
    pub fn credit_base(&mut self, amount: u64) {
        self.base_free = self.base_free.saturating_add(amount);
    }

    /// Credits quote tokens to free balance (from deposit or fill).
    ///
    /// # Arguments
    ///
    /// * `amount` - Amount to credit
    #[inline]
    pub fn credit_quote(&mut self, amount: u64) {
        self.quote_free = self.quote_free.saturating_add(amount);
    }

    /// Debits base tokens from free balance (for withdrawal).
    ///
    /// # Arguments
    ///
    /// * `amount` - Amount to debit
    ///
    /// # Returns
    ///
    /// `true` if successful, `false` if insufficient free balance.
    pub fn debit_base(&mut self, amount: u64) -> bool {
        if self.base_free < amount {
            return false;
        }
        self.base_free = self.base_free.saturating_sub(amount);
        true
    }

    /// Debits quote tokens from free balance (for withdrawal).
    ///
    /// # Arguments
    ///
    /// * `amount` - Amount to debit
    ///
    /// # Returns
    ///
    /// `true` if successful, `false` if insufficient free balance.
    pub fn debit_quote(&mut self, amount: u64) -> bool {
        if self.quote_free < amount {
            return false;
        }
        self.quote_free = self.quote_free.saturating_sub(amount);
        true
    }

    /// Settles a fill for the maker (ask side - sold base, received quote).
    ///
    /// # Arguments
    ///
    /// * `base_amount` - Base tokens sold
    /// * `quote_amount` - Quote tokens received (after fees)
    ///
    /// # Returns
    ///
    /// `true` if successful.
    pub fn settle_maker_ask(&mut self, base_amount: u64, quote_amount: u64) -> bool {
        if self.base_locked < base_amount {
            return false;
        }
        self.base_locked = self.base_locked.saturating_sub(base_amount);
        self.quote_free = self.quote_free.saturating_add(quote_amount);
        true
    }

    /// Settles a fill for the maker (bid side - received base, sold quote).
    ///
    /// # Arguments
    ///
    /// * `base_amount` - Base tokens received
    /// * `quote_amount` - Quote tokens sold
    ///
    /// # Returns
    ///
    /// `true` if successful.
    pub fn settle_maker_bid(&mut self, base_amount: u64, quote_amount: u64) -> bool {
        if self.quote_locked < quote_amount {
            return false;
        }
        self.quote_locked = self.quote_locked.saturating_sub(quote_amount);
        self.base_free = self.base_free.saturating_add(base_amount);
        true
    }

    /// Adds referrer rebates.
    ///
    /// # Arguments
    ///
    /// * `amount` - Rebate amount
    #[inline]
    pub fn add_referrer_rebates(&mut self, amount: u64) {
        self.referrer_rebates = self.referrer_rebates.saturating_add(amount);
    }

    /// Claims referrer rebates.
    ///
    /// # Returns
    ///
    /// The amount of rebates claimed.
    pub fn claim_referrer_rebates(&mut self) -> u64 {
        let amount = self.referrer_rebates;
        self.referrer_rebates = 0;
        amount
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_open_orders() -> OpenOrders {
        OpenOrders {
            bump: 255,
            market: Pubkey::new_unique(),
            owner: Pubkey::new_unique(),
            delegate: Pubkey::default(),
            base_locked: 0,
            quote_locked: 0,
            base_free: 1000,
            quote_free: 5000,
            referrer_rebates: 0,
            num_orders: 0,
            reserved: [0; 64],
            orders: [OrderSlot::default(); MAX_ORDERS],
        }
    }

    #[test]
    fn test_open_orders_pda_derivation() {
        let market = Pubkey::new_unique();
        let owner = Pubkey::new_unique();
        let program_id = Pubkey::new_unique();

        let (pda1, bump1) = OpenOrders::derive_pda(&market, &owner, &program_id);
        let (pda2, bump2) = OpenOrders::derive_pda(&market, &owner, &program_id);

        // Same inputs produce same outputs
        assert_eq!(pda1, pda2);
        assert_eq!(bump1, bump2);

        // Different owner produces different PDA
        let other_owner = Pubkey::new_unique();
        let (pda3, _) = OpenOrders::derive_pda(&market, &other_owner, &program_id);
        assert_ne!(pda1, pda3);

        // Different market produces different PDA
        let other_market = Pubkey::new_unique();
        let (pda4, _) = OpenOrders::derive_pda(&other_market, &owner, &program_id);
        assert_ne!(pda1, pda4);
    }

    #[test]
    fn test_find_free_slot_empty() {
        let oo = create_test_open_orders();
        let slot = oo.find_free_slot();
        assert_eq!(slot, Some(0));
    }

    #[test]
    fn test_find_free_slot_partial() {
        let mut oo = create_test_open_orders();

        // Fill first 3 slots
        oo.add_order(0, 100, 1, Side::Bid);
        oo.add_order(1, 200, 2, Side::Ask);
        oo.add_order(2, 300, 3, Side::Bid);

        let slot = oo.find_free_slot();
        assert_eq!(slot, Some(3));
    }

    #[test]
    fn test_find_free_slot_full() {
        let mut oo = create_test_open_orders();

        // Fill all slots
        for i in 0..MAX_ORDERS {
            if let Some(slot) = oo.orders.get_mut(i) {
                *slot = OrderSlot::new(i as u128 + 1, i as u64, Side::Bid);
            }
        }
        oo.num_orders = MAX_ORDERS as u8;

        let slot = oo.find_free_slot();
        assert!(slot.is_none());
    }

    #[test]
    fn test_add_remove_order() {
        let mut oo = create_test_open_orders();

        // Add order
        let success = oo.add_order(5, 12345, 100, Side::Ask);
        assert!(success);
        assert_eq!(oo.num_orders, 1);

        // Verify slot contents
        let slot = oo.get_order(5);
        assert!(slot.is_some());
        assert!(slot.is_some_and(|s| s.order_id == 12345));
        assert!(slot.is_some_and(|s| s.client_order_id == 100));
        assert!(slot.is_some_and(|s| s.side == Side::Ask));

        // Remove order
        let removed = oo.remove_order(5);
        assert!(removed.is_some());
        assert_eq!(oo.num_orders, 0);

        // Verify slot is empty
        let slot = oo.get_order(5);
        assert!(slot.is_some_and(|s| s.is_empty()));
    }

    #[test]
    fn test_add_order_slot_occupied() {
        let mut oo = create_test_open_orders();

        // Add first order
        let success1 = oo.add_order(0, 100, 1, Side::Bid);
        assert!(success1);

        // Try to add to same slot
        let success2 = oo.add_order(0, 200, 2, Side::Ask);
        assert!(!success2);
        assert_eq!(oo.num_orders, 1);
    }

    #[test]
    fn test_add_order_out_of_bounds() {
        let mut oo = create_test_open_orders();

        let success = oo.add_order(255, 100, 1, Side::Bid);
        assert!(!success);
    }

    #[test]
    fn test_remove_order_empty_slot() {
        let mut oo = create_test_open_orders();

        let removed = oo.remove_order(0);
        assert!(removed.is_none());
    }

    #[test]
    fn test_find_order() {
        let mut oo = create_test_open_orders();

        oo.add_order(3, 12345, 100, Side::Bid);
        oo.add_order(7, 67890, 200, Side::Ask);

        assert_eq!(oo.find_order(12345), Some(3));
        assert_eq!(oo.find_order(67890), Some(7));
        assert_eq!(oo.find_order(99999), None);
    }

    #[test]
    fn test_is_authorized_owner() {
        let oo = create_test_open_orders();
        assert!(oo.is_authorized(&oo.owner));
    }

    #[test]
    fn test_is_authorized_delegate() {
        let mut oo = create_test_open_orders();
        let delegate = Pubkey::new_unique();

        // No delegate set
        assert!(!oo.is_authorized(&delegate));

        // Set delegate
        oo.set_delegate(delegate);
        assert!(oo.is_authorized(&delegate));
        assert!(oo.has_delegate());

        // Clear delegate
        oo.clear_delegate();
        assert!(!oo.is_authorized(&delegate));
        assert!(!oo.has_delegate());
    }

    #[test]
    fn test_is_authorized_other() {
        let oo = create_test_open_orders();
        let random = Pubkey::new_unique();
        assert!(!oo.is_authorized(&random));
    }

    #[test]
    fn test_lock_release_base() {
        let mut oo = create_test_open_orders();
        assert_eq!(oo.base_free, 1000);
        assert_eq!(oo.base_locked, 0);

        // Lock some base
        let success = oo.lock_base(300);
        assert!(success);
        assert_eq!(oo.base_free, 700);
        assert_eq!(oo.base_locked, 300);
        assert_eq!(oo.total_base(), 1000);

        // Release some back
        let success = oo.release_base(100);
        assert!(success);
        assert_eq!(oo.base_free, 800);
        assert_eq!(oo.base_locked, 200);
    }

    #[test]
    fn test_lock_release_quote() {
        let mut oo = create_test_open_orders();
        assert_eq!(oo.quote_free, 5000);
        assert_eq!(oo.quote_locked, 0);

        // Lock some quote
        let success = oo.lock_quote(2000);
        assert!(success);
        assert_eq!(oo.quote_free, 3000);
        assert_eq!(oo.quote_locked, 2000);
        assert_eq!(oo.total_quote(), 5000);

        // Release some back
        let success = oo.release_quote(500);
        assert!(success);
        assert_eq!(oo.quote_free, 3500);
        assert_eq!(oo.quote_locked, 1500);
    }

    #[test]
    fn test_lock_insufficient() {
        let mut oo = create_test_open_orders();

        // Try to lock more than available
        let success = oo.lock_base(2000);
        assert!(!success);
        assert_eq!(oo.base_free, 1000);
        assert_eq!(oo.base_locked, 0);

        let success = oo.lock_quote(10000);
        assert!(!success);
        assert_eq!(oo.quote_free, 5000);
        assert_eq!(oo.quote_locked, 0);
    }

    #[test]
    fn test_release_insufficient() {
        let mut oo = create_test_open_orders();
        oo.lock_base(500);

        // Try to release more than locked
        let success = oo.release_base(600);
        assert!(!success);
        assert_eq!(oo.base_locked, 500);
    }

    #[test]
    fn test_credit_debit() {
        let mut oo = create_test_open_orders();

        // Credit
        oo.credit_base(500);
        assert_eq!(oo.base_free, 1500);

        oo.credit_quote(1000);
        assert_eq!(oo.quote_free, 6000);

        // Debit
        let success = oo.debit_base(200);
        assert!(success);
        assert_eq!(oo.base_free, 1300);

        let success = oo.debit_quote(500);
        assert!(success);
        assert_eq!(oo.quote_free, 5500);

        // Debit insufficient
        let success = oo.debit_base(2000);
        assert!(!success);
    }

    #[test]
    fn test_settle_maker_ask() {
        let mut oo = create_test_open_orders();
        oo.lock_base(500); // Lock base for ask order

        // Settle: sold 300 base, received 600 quote
        let success = oo.settle_maker_ask(300, 600);
        assert!(success);
        assert_eq!(oo.base_locked, 200);
        assert_eq!(oo.quote_free, 5600);
    }

    #[test]
    fn test_settle_maker_bid() {
        let mut oo = create_test_open_orders();
        oo.lock_quote(2000); // Lock quote for bid order

        // Settle: received 100 base, sold 1000 quote
        let success = oo.settle_maker_bid(100, 1000);
        assert!(success);
        assert_eq!(oo.quote_locked, 1000);
        assert_eq!(oo.base_free, 1100);
    }

    #[test]
    fn test_referrer_rebates() {
        let mut oo = create_test_open_orders();

        oo.add_referrer_rebates(100);
        oo.add_referrer_rebates(50);
        assert_eq!(oo.referrer_rebates, 150);

        let claimed = oo.claim_referrer_rebates();
        assert_eq!(claimed, 150);
        assert_eq!(oo.referrer_rebates, 0);
    }

    #[test]
    fn test_order_slot_default() {
        let slot = OrderSlot::default();
        assert!(slot.is_empty());
        assert_eq!(slot.order_id, EMPTY_ORDER_ID);
    }

    #[test]
    fn test_order_count_helpers() {
        let mut oo = create_test_open_orders();

        assert!(oo.has_no_orders());
        assert!(!oo.is_full());

        oo.add_order(0, 100, 1, Side::Bid);
        assert!(!oo.has_no_orders());
        assert_eq!(oo.order_count(), 1);
    }
}
