#![cfg(target_arch="x86_64")]

#![feature(const_fn)]
#![feature(asm)]
#![feature(associated_consts)]
#![no_std]
#![cfg_attr(test, allow(unused_features))]

#[macro_use]
extern crate bitflags;

#[macro_use]
extern crate raw_cpuid;

macro_rules! bit {
    ( $x:expr ) => {
        1 << $x
    };
}

macro_rules! check_flag {
    ($doc:meta, $fun:ident, $flag:ident) => (
        #[$doc]
        pub fn $fun(&self) -> bool {
            self.contains($flag)
        }
    )
}

macro_rules! is_bit_set {
    ($field:expr, $bit:expr) => (
        $field & (1 << $bit) > 0
    )
}

macro_rules! check_bit_fn {
    ($doc:meta, $fun:ident, $field:ident, $bit:expr) => (
        #[$doc]
        pub fn $fun(&self) -> bool {
            is_bit_set!(self.$field, $bit)
        }
    )
}

pub mod time;
pub mod irq;
pub mod paging;
pub mod task;
pub mod syscall;
pub mod sgx;
pub mod control_regs;
pub mod descriptor;
pub mod dtables;
pub mod io;
pub mod msr;
pub mod flags;
pub mod segmentation;
pub mod tlb;

pub mod cpuid {
    pub use raw_cpuid::*;
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


mod std {
    pub use core::fmt;
    pub use core::ops;
    pub use core::option;
}
