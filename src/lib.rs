#![feature(no_std)]
#![feature(core)]
#![feature(asm)]
#![no_std]

#![crate_name = "x86"]
#![crate_type = "lib"]

#[macro_use]
extern crate core;

#[macro_use]
extern crate bitflags;

#[cfg(test)]
extern crate std;

#[cfg(not(test))]
mod std {
    pub use core::ops;
    pub use core::option;
}

pub mod io;
pub mod controlregs;
pub mod msr;
pub mod time;
pub mod irq;
pub mod mem;
pub mod rflags;