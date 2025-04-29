//! Provides glue to support memory encryption features of some CPUs.
//!
//! Memory encryption typically relies on using a physical address bit in the page table entry to
//! mark a page as encrypted. This module provides a function that, given a c-bit, updates the
//! dynamic page-table flags structure to ensure that the flag is treated properly and not
//! returned as part of the physical address.

use core::sync::atomic::{AtomicU64, Ordering};

use crate::structures::paging::page_table::PHYSICAL_ADDRESS_MASK;
use crate::structures::paging::PageTableFlags;

static C_BIT_MASK: AtomicU64 = AtomicU64::new(0);

/// Enable memory encryption by defining the physical address bit that is used to mark a page
/// encrypted in a page table entry
pub fn define_encryption_bit(bit_position: u64) {
    let c_bit_mask = 1u64 << bit_position;

    PHYSICAL_ADDRESS_MASK.fetch_and(!c_bit_mask, Ordering::AcqRel);
    C_BIT_MASK.store(c_bit_mask, Ordering::Relaxed);
}

impl PageTableFlags {
    #[inline]
    fn c_bit_mask() -> PageTableFlags {
        let bit_mask = C_BIT_MASK.load(Ordering::Relaxed);
        assert_ne!(bit_mask, 0, "C-bit is not set");
        PageTableFlags::from_bits_retain(bit_mask)
    }

    /// Sets the encryption bit on the page table entry.
    ///
    /// Requires memory encryption to be enabled, or this will panic.
    pub fn set_encrypted(&mut self, encrypted: bool) {
        self.set(Self::c_bit_mask(), encrypted);
    }

    /// Checks if the encryption bit is set on the page table entry.
    ///
    /// Requires memory encryption to be enabled, or this will panic.
    pub fn is_encrypted(&self) -> bool {
        self.contains(Self::c_bit_mask())
    }
}
