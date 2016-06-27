#![cfg(any(target_arch="x86", target_arch="x86_64"))]

#![no_std]
#![crate_name="cpu"]
#![crate_type="rlib"]
#![feature(asm)]
#![feature(associated_consts)]

#[macro_use]
extern crate bitflags;

pub use cpu::*;

#[cfg(target_arch="x86")]
#[path = "x86.rs"]
mod cpu;
#[cfg(target_arch="x86_64")]
#[path = "x86_64.rs"]
mod cpu;

pub mod std { pub use core::*; }
