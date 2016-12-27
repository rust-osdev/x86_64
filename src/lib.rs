#![cfg(any(target_arch="x86", target_arch="x86_64"))]

#![feature(const_fn)]
#![feature(asm)]
#![feature(associated_consts)]
#![no_std]
#![cfg_attr(test, allow(unused_features))]

#[macro_use]
extern crate bitflags;

#[macro_use]
extern crate raw_cpuid;

#[cfg(target_arch="x86_64")]
pub mod bits64;
pub mod shared;

pub mod current {
    #[cfg(target_arch="x86_64")]
    pub use bits64::*;
}

mod std {
    pub use core::fmt;
    pub use core::ops;
    pub use core::option;
}
