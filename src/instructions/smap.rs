//! This module provides helpers for Supervisor Mode Access Prevention (SMAP).
//!
//! SMAP is a security feature that helps prevent accidental accesses to user
//! memory by a kernel. This feature can be enabled by setting
//! [`Cr4Flags::SUPERVISOR_MODE_ACCESS_PREVENTION`]. Once enabled, accesses to
//! user memory by the kernel will generate a page fault (#PF) unless the
//! [`RFlags::ALIGNMENT_CHECK`] bit is set.
//!
//! The `stac` and `clac` instructions can be used to efficiently update the
//! `ALIGNMENT_CHECK` flag.
//!
//! Not all processors support SMAP.

use core::arch::asm;

use bit_field::BitField;

#[cfg(doc)]
use crate::registers::control::Cr4Flags;
use crate::registers::rflags::{self, RFlags};

/// A helper type that provides SMAP related methods.
///
/// This type can only be instatiated if SMAP is supported by the CPU.
#[derive(Debug, Clone, Copy)]
pub struct Smap(());

impl Smap {
    /// Checks if the CPU supports SMAP and returns a [`Smap`] instance if
    /// supported or `None` if not.
    ///
    /// This function uses CPUID to determine if SMAP is supported by the CPU.
    ///
    /// Note that this function does not check whether SMAP has be enabled in
    /// CR4.
    pub fn new() -> Option<Self> {
        // Check if the CPU supports `stac` and `clac`.
        let cpuid = unsafe { core::arch::x86_64::__cpuid(7) };
        if cpuid.ebx.get_bit(20) {
            Some(Self(()))
        } else {
            None
        }
    }

    /// Returns a [`Smap`] instance.
    ///
    /// # Safety
    ///
    /// The caller must ensure that the CPU supports SMAP.
    #[inline]
    pub const unsafe fn new_unchecked() -> Self {
        Self(())
    }

    /// Returns whether the [`RFlags::ALIGNMENT_CHECK`] flag is unset.
    ///
    /// Note that SMAP also requires
    /// [`Cr4Flags::SUPERVISOR_MODE_ACCESS_PREVENTION`] to be set. This
    /// function does not check CR4 because doing so is much slower than just
    /// checking the AC flag.
    #[inline]
    pub fn is_enabled(self) -> bool {
        !rflags::read().contains(RFlags::ALIGNMENT_CHECK)
    }

    /// Disable SMAP access checks by setting [`RFlags::ALIGNMENT_CHECK`] using
    /// the `stac` instruction.
    ///
    /// This will do nothing if `SMAP` access checks are already disabled.
    #[doc(alias = "stac")]
    #[inline]
    pub fn disable(self) {
        // Technically this modifies the AC flag, but the Rust compiler doesn't
        // care about that, so it's fine to use preserves_flags.
        unsafe {
            asm!("stac", options(nomem, nostack, preserves_flags));
        }
    }

    /// Enable SMAP access checks by clearing [`RFlags::ALIGNMENT_CHECK`] using
    /// the `clac` instruction.
    ///
    /// This will do nothing if `SMAP` access checks are already enabled.
    #[doc(alias = "clac")]
    #[inline]
    pub fn enable(self) {
        // Technically this modifies the AC flag, but the Rust compiler doesn't
        // care about that, so it's fine to use preserves_flags.
        unsafe {
            asm!("clac", options(nomem, nostack, preserves_flags));
        }
    }

    /// Call a closure with SMAP disabled.
    ///
    /// This function disables SMAP before calling the closure and restores the
    /// SMAP state afterwards.
    pub fn without_smap<F, R>(self, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        let was_enabled = self.is_enabled();

        self.disable();

        let result = f();

        if was_enabled {
            self.enable();
        }

        result
    }
}
