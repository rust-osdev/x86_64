#![no_std]
#![crate_name="cpu"]
#![crate_type="rlib"]
#![feature(asm, globs, macro_rules, phase)]

#[phase(plugin, link)]
extern crate core;

#[cfg(target_arch="x86")]
pub use x86::*;
#[cfg(target_arch="x86_64")]
pub use x86_64::*;
#[cfg(any(target_arch="x86", target_arch="x86_64"))]
pub use x86_shared::*;

mod bitflags;
#[cfg(target_arch="x86")]
mod x86;
#[cfg(target_arch="x86_64")]
mod x86_64;
#[cfg(any(target_arch="x86", target_arch="x86_64"))]
mod x86_shared;

pub mod std {
	pub use core::fmt;
	pub use core::num;
	pub use core::option;
	pub use core::cmp;
	pub use core::clone;
}
