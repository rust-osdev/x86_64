//! This crate provides x86_64 specific functions and data structures,
//! and access to various system registers.

#![cfg_attr(not(test), no_std)]
#![cfg_attr(feature = "const_fn", feature(const_mut_refs))] // GDT add_entry()
#![cfg_attr(feature = "const_fn", feature(const_fn_fn_ptr_basics))] // IDT new()
#![cfg_attr(feature = "const_fn", feature(const_fn_trait_bound))] // PageSize marker trait
#![cfg_attr(feature = "inline_asm", feature(asm))]
#![cfg_attr(feature = "abi_x86_interrupt", feature(abi_x86_interrupt))]
#![cfg_attr(feature = "doc_cfg", feature(doc_cfg))]
#![warn(missing_docs)]
#![deny(missing_debug_implementations)]

pub use crate::addr::{align_down, align_up, PhysAddr, VirtAddr};

/// Makes a function const only when `feature = "const_fn"` is enabled.
///
/// This is needed for const functions with bounds on their generic parameters,
/// such as those in `Page` and `PhysFrame` and many more.
macro_rules! const_fn {
    (
        $(#[$attr:meta])*
        $sv:vis fn $($fn:tt)*
    ) => {
        $(#[$attr])*
        #[cfg(feature = "const_fn")]
        $sv const fn $($fn)*

        $(#[$attr])*
        #[cfg(not(feature = "const_fn"))]
        $sv fn $($fn)*
    };
    (
        $(#[$attr:meta])*
        $sv:vis unsafe fn $($fn:tt)*
    ) => {
        $(#[$attr])*
        #[cfg(feature = "const_fn")]
        $sv const unsafe fn $($fn)*

        $(#[$attr])*
        #[cfg(not(feature = "const_fn"))]
        $sv unsafe fn $($fn)*
    };
}

// Helper method for assert! in const fn. Uses out of bounds indexing if an
// assertion fails and the "const_fn" feature is not enabled.
#[cfg(feature = "const_fn")]
macro_rules! const_assert {
    ($cond:expr, $($arg:tt)+) => { assert!($cond, $($arg)*) };
}
#[cfg(not(feature = "const_fn"))]
macro_rules! const_assert {
    ($cond:expr, $($arg:tt)+) => {
        [(); 1][!($cond as bool) as usize]
    };
}

#[cfg(all(feature = "instructions", feature = "external_asm"))]
pub(crate) mod asm;

pub mod addr;
pub mod instructions;
pub mod registers;
pub mod structures;

/// Represents a protection ring level.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(u8)]
pub enum PrivilegeLevel {
    /// Privilege-level 0 (most privilege): This level is used by critical system-software
    /// components that require direct access to, and control over, all processor and system
    /// resources. This can include BIOS, memory-management functions, and interrupt handlers.
    Ring0 = 0,

    /// Privilege-level 1 (moderate privilege): This level is used by less-critical system-
    /// software services that can access and control a limited scope of processor and system
    /// resources. Software running at these privilege levels might include some device drivers
    /// and library routines. The actual privileges of this level are defined by the
    /// operating system.
    Ring1 = 1,

    /// Privilege-level 2 (moderate privilege): Like level 1, this level is used by
    /// less-critical system-software services that can access and control a limited scope of
    /// processor and system resources. The actual privileges of this level are defined by the
    /// operating system.
    Ring2 = 2,

    /// Privilege-level 3 (least privilege): This level is used by application software.
    /// Software running at privilege-level 3 is normally prevented from directly accessing
    /// most processor and system resources. Instead, applications request access to the
    /// protected processor and system resources by calling more-privileged service routines
    /// to perform the accesses.
    Ring3 = 3,
}

impl PrivilegeLevel {
    /// Creates a `PrivilegeLevel` from a numeric value. The value must be in the range 0..4.
    ///
    /// This function panics if the passed value is >3.
    #[inline]
    pub fn from_u16(value: u16) -> PrivilegeLevel {
        match value {
            0 => PrivilegeLevel::Ring0,
            1 => PrivilegeLevel::Ring1,
            2 => PrivilegeLevel::Ring2,
            3 => PrivilegeLevel::Ring3,
            i => panic!("{} is not a valid privilege level", i),
        }
    }
}
