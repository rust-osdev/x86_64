//! This crate provides x86_64 specific functions and data structures,
//! and access to various system registers.

#![cfg_attr(not(test), no_std)]
#![cfg_attr(feature = "const_fn", feature(const_mut_refs))] // GDT add_entry()
#![cfg_attr(feature = "asm_const", feature(asm_const))]
#![cfg_attr(feature = "abi_x86_interrupt", feature(abi_x86_interrupt))]
#![cfg_attr(feature = "step_trait", feature(step_trait))]
#![cfg_attr(feature = "doc_cfg", feature(doc_cfg))]
#![warn(missing_docs)]
#![deny(missing_debug_implementations)]
#![deny(unsafe_op_in_unsafe_fn)]

use core::cell::UnsafeCell;
use core::sync::atomic::{AtomicBool, Ordering};

pub use crate::addr::{align_down, align_up, PhysAddr, VirtAddr};

pub mod addr;
pub mod instructions;
pub mod registers;
pub mod structures;

/// Represents a protection ring level.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
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

/// A wrapper that can be used to safely create one mutable reference `&'static mut T` from a static variable.
///
/// `SingleUseCell` is safe because it ensures that it only ever gives out one reference.
///
/// ``SingleUseCell<T>` is a safe alternative to `static mut` or a static `UnsafeCell<T>`.
#[derive(Debug)]
pub struct SingleUseCell<T> {
    used: AtomicBool,
    value: UnsafeCell<T>,
}

impl<T> SingleUseCell<T> {
    /// Construct a new SingleUseCell.
    pub const fn new(value: T) -> Self {
        Self {
            used: AtomicBool::new(false),
            value: UnsafeCell::new(value),
        }
    }

    /// Try to acquire a mutable reference to the wrapped value.
    /// This will only succeed the first time the function is
    /// called and fail on all following calls.
    ///
    /// ```
    /// use x86_64::SingleUseCell;
    ///
    /// static FOO: SingleUseCell<i32> = SingleUseCell::new(0);
    ///
    /// // Call `try_get_mut` for the first time and get a reference.
    /// let first: &'static mut i32 = FOO.try_get_mut().unwrap();
    /// assert_eq!(first, &0);
    ///
    /// // Calling `try_get_mut` again will return `None`.
    /// assert_eq!(FOO.try_get_mut(), None);
    /// ```
    pub fn try_get_mut(&self) -> Option<&mut T> {
        let already_used = self.used.swap(true, Ordering::AcqRel);
        if already_used {
            None
        } else {
            Some(unsafe {
                // SAFETY: no reference has been given out yet and we won't give out another.
                &mut *self.value.get()
            })
        }
    }
}

// SAFETY: Sharing a `SingleUseCell<T>` between threads is safe regardless of whether `T` is `Sync`
// because we only expose the inner value once to one thread. The `T: Send` bound makes sure that
// sending a unique reference to another thread is safe.
unsafe impl<T: Send> Sync for SingleUseCell<T> {}

// SAFETY: It's safe to send a `SingleUseCell<T>` to another thread if it's safe to send `T`.
unsafe impl<T: Send> Send for SingleUseCell<T> {}
