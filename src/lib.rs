#![feature(no_std, core_str_ext, core_slice_ext, const_fn)]
#![feature(asm)]
#![no_std]
#![cfg_attr(test, allow(unused_features))]

#![crate_name = "x86"]
#![crate_type = "lib"]

#[macro_use]
mod bitflags;

macro_rules! bit {
    ( $x:expr ) => {
        1 << $x
    };
}

#[macro_use]
extern crate raw_cpuid;

#[macro_use]
extern crate phf;

mod std {
    pub use core::fmt;
    pub use core::ops;
    pub use core::option;
}

pub mod io;
pub mod controlregs;
pub mod msr;
pub mod time;
pub mod irq;
pub mod rflags;
pub mod paging;
pub mod segmentation;
pub mod task;
pub mod dtables;
pub mod syscall;
pub mod perfcnt;
pub mod cpuid {
    pub use raw_cpuid::*;
}
pub mod tlb;
