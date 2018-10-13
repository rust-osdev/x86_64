#![cfg(any(target_arch = "x86", target_arch = "x86_64"))]
#![allow(stable_features)]
#![feature(const_fn, asm, repr_transparent)]
#![no_std]
#![cfg_attr(test, allow(unused_features))]

#[macro_use]
extern crate bitflags;
extern crate raw_cpuid;
#[cfg(feature = "performance-counter")]
#[macro_use]
extern crate phf;

#[cfg(test)]
#[macro_use]
extern crate std;

#[cfg(target_arch = "x86")]
use core::arch::x86 as arch;
#[cfg(target_arch = "x86_64")]
use core::arch::x86_64 as arch;

macro_rules! bit {
    ($x:expr) => {
        1 << $x
    };
}

pub mod bits16;
pub mod bits32;
pub mod bits64;

pub mod controlregs;
pub mod dtables;
pub mod io;
pub mod irq;
pub mod msr;
pub mod segmentation;
pub mod task;
pub mod time;
pub mod tlb;

#[cfg(feature = "performance-counter")]
pub mod perfcnt;

pub mod current {
    #[cfg(target_arch = "x86")]
    pub use bits32::*;
    #[cfg(target_arch = "x86_64")]
    pub use bits64::*;
}

pub mod cpuid {
    pub use raw_cpuid::*;
}

#[cfg(not(test))]
mod std {
    pub use core::fmt;
    pub use core::ops;
    pub use core::option;
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u8)]
/// x86 Protection levels
/// Note: This should not contain values larger than 2 bits, otherwise
/// segment descriptor code needs to be adjusted accordingly.
pub enum Ring {
    Ring0 = 0b00,
    Ring1 = 0b01,
    Ring2 = 0b10,
    Ring3 = 0b11,
}

#[inline(always)]
pub unsafe fn halt() {
    asm!("hlt" :::: "volatile");
}
