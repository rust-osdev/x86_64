#![feature(no_std, core_str_ext, core_slice_ext)]
#![feature(asm)]
#![no_std]

#![crate_name = "x86"]
#![crate_type = "lib"]

#[macro_use]
mod bitflags;

#[cfg(test)]
#[macro_use]
extern crate std;

#[macro_use]
extern crate raw_cpuid;

#[macro_use]
extern crate phf;

#[cfg(not(test))]
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