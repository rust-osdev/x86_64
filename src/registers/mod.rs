//! Access to various system and model specific registers.

pub mod control;
pub mod debug;
pub mod model_specific;
pub mod mxcsr;
pub mod rflags;
pub mod segmentation;
pub mod xcontrol;

#[cfg(all(feature = "instructions", target_arch = "x86_64"))]
pub use crate::instructions::read_rip;
