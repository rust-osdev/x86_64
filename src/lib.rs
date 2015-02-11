#![cfg(any(target_arch="x86", target_arch="x86_64"))]

#![no_std]
#![crate_name="cpu"]
#![crate_type="rlib"]
#![feature(no_std)]
#![feature(asm)]
#![feature(core)]
#![feature(hash)]

#[macro_use]
extern crate core;
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
