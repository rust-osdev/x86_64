#![cfg(target_arch = "x86_64")]

//! Special x86_64 instructions.

pub mod interrupts;
pub mod port;
pub mod random;
pub mod segmentation;
pub mod tables;
pub mod tlb;

/// Halts the CPU until the next interrupt arrives.
#[inline(always)]
pub fn hlt() {
    unsafe {
        asm!("hlt" :::: "volatile");
    }
}
