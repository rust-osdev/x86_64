#![feature(no_std)]
#![feature(core)]
#![feature(asm)]
#![no_std]

#![crate_name = "x86"]
#![crate_type = "lib"]

#[macro_use]
extern crate core;

#[cfg(test)]
extern crate std;

pub mod io;
pub mod controlregs;
pub mod msr;
pub mod time;
pub mod irq;