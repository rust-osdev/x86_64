//! Special x86_64 instructions.

pub mod interrupts;
pub mod port;
pub mod segmentation;
pub mod tables;
pub mod tlb;

/// Cause a breakpoint exception by invoking the `int3` instruction.
pub fn int3() {
    unsafe {
        asm!("int3");
    }
}

/// Halts the CPU until the next interrupt arrives.
#[inline(always)]
pub unsafe fn hlt() {
    asm!("hlt" :::: "volatile");
}
