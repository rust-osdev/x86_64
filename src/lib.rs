#![cfg(any(target_arch="x86", target_arch="x86_64"))]

#![feature(const_fn)]
#![feature(asm)]
#![no_std]
#![cfg_attr(test, allow(unused_features))]

#[macro_use]
extern crate bitflags;
extern crate raw_cpuid;
#[cfg(feature = "performance-counter")]
#[macro_use]
extern crate phf;

macro_rules! bit {
    ( $x:expr ) => {
        1 << $x
    };
}

pub mod bits32;
pub mod bits64;

pub mod controlregs;
pub mod descriptor;
pub mod dtables;
pub mod io;
pub mod irq;
pub mod msr;
pub mod paging;
pub mod segmentation;
pub mod task;
pub mod tlb;
pub mod time;

#[cfg(feature = "performance-counter")]
pub mod perfcnt;

pub mod current {
  #[cfg(target_arch="x86")]
  pub use bits32::*;
  #[cfg(target_arch="x86_64")]
  pub use bits64::*;
}

pub mod cpuid {
    pub use raw_cpuid::*;
}

mod std {
    pub use core::fmt;
    pub use core::ops;
    pub use core::option;
}

#[derive(Copy, Clone, PartialEq, Eq)]
#[repr(u8)]
pub enum PrivilegeLevel {
    Ring0 = 0,
    Ring1 = 1,
    Ring2 = 2,
    Ring3 = 3,
}

#[inline(always)]
pub unsafe fn halt() {
    asm!("hlt" :::: "volatile");
}