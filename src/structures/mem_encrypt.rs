//! Provides glue to support memory encryption features of some CPUs.
//!
//! Memory encryption typically relies on using a physical address bit in the page table entry to
//! mark a page as encrypted/decrypted. This module provides a function that, given the position and
//! type of that physical address bit, updates the dynamic page-table flags structure to ensure that
//! the flag is treated properly and not returned as part of the physical address.

use core::sync::atomic::{AtomicBool, AtomicU64, Ordering};

use crate::structures::paging::page_table::PHYSICAL_ADDRESS_MASK;
use crate::structures::paging::PageTableFlags;

/// Position of the encryption (C/S) bit in the physical address
pub(crate) static ENC_BIT_MASK: AtomicU64 = AtomicU64::new(0);

/// Is the encryption bit reversed (i.e. its presence denote that the page is _decrypted_ rather
/// than encrypted)
static ENC_BIT_REVERSED: AtomicBool = AtomicBool::new(false);

/// Defines the configuration for memory encryption
#[derive(Debug)]
pub enum MemoryEncryptionConfiguration {
    /// Defines that a memory page should be accessed encrypted if this bit of its physical address
    /// is set in the page table entry.
    ///
    /// Use this for AMD Secure Memory Encryption (AMD-SME) and Secure Encrypted Virtualization (SEV)
    EncryptedBit(u8),

    /// Defines that a memory page should be accessed decrypted if this bit of its physical address
    /// is set in the page table entry.
    ///
    /// Use this for Intel Trust Domain Extension (Intel TDX)
    SharedBit(u8),
}

/// Enable memory encryption by defining the physical address bit that is used to mark a page
/// encrypted (or shared) in a page table entry
///
/// # Safety
/// Caller must make sure that any existing page table entry is discarded or adapted to take this
/// bit into consideration.
/// The configuration provided by caller must be correct, otherwise physical address bits will
/// incorrectly be considered as page table flags.
pub unsafe fn enable_memory_encryption(configuration: MemoryEncryptionConfiguration) {
    let (bit_position, reversed) = match configuration {
        MemoryEncryptionConfiguration::EncryptedBit(pos) => (pos, false),
        MemoryEncryptionConfiguration::SharedBit(pos) => (pos, true),
    };

    let c_bit_mask = 1u64 << bit_position;

    PHYSICAL_ADDRESS_MASK.fetch_and(!c_bit_mask, Ordering::Relaxed);
    ENC_BIT_MASK.store(c_bit_mask, Ordering::Relaxed);
    ENC_BIT_REVERSED.store(reversed, Ordering::Release);
}

impl PageTableFlags {
    #[inline]
    fn enc_bit_flag() -> Option<PageTableFlags> {
        let bit_mask = ENC_BIT_MASK.load(Ordering::Relaxed);

        if bit_mask > 0 {
            Some(PageTableFlags::from_bits_retain(bit_mask))
        } else {
            None
        }
    }

    /// Marks the page for encryption in the page table entry.
    ///
    /// # Panics
    ///
    /// Panics if memory encryption has not been previously configured by calling [`enable_memory_encryption`]
    pub fn set_encrypted(&mut self, encrypted: bool) {
        let flag = Self::enc_bit_flag().expect("memory encryption is not enabled");
        self.set(flag, encrypted ^ ENC_BIT_REVERSED.load(Ordering::Relaxed));
    }

    /// Checks if memory encryption is enabled on the page
    pub fn is_encrypted(&self) -> bool {
        if let Some(c_bit_flag) = Self::enc_bit_flag() {
            self.contains(c_bit_flag) ^ ENC_BIT_REVERSED.load(Ordering::Relaxed)
        } else {
            false
        }
    }
}
