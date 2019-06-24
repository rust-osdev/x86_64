#![cfg(target_arch = "x86_64")]

//! Special x86_64 instructions.

pub mod interrupts;
pub mod port;
pub mod random;
pub mod segmentation;
pub mod tables;
pub mod tlb;

/// Halts the CPU until the next interrupt arrives.
#[inline]
pub fn hlt() {
    unsafe {
        asm!("hlt" :::: "volatile");
    }
}

/// Emits a 'magic breakpoint' instruction for the [Bochs](http://bochs.sourceforge.net/) CPU
/// emulator. Make sure to set `magic_break: enabled=1` in your `.bochsrc` file.
#[inline]
pub fn bochs_breakpoint() {
    unsafe {
        asm!("xchgw %bx, %bx" :::: "volatile");
    }
}

/// Gets the current instruction pointer. Note that this is only approximate as it requires a few
/// instructions to execute.
#[inline]
pub fn read_rip() -> u64 {
    let rip: u64;
    unsafe {
        asm!(
            "call .next
            .next:
            pop $0"
            : "=r"(rip) ::: "volatile"
        );
    }
    rip
}
