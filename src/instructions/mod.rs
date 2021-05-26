#![cfg(feature = "instructions")]

//! Special x86_64 instructions.

pub mod interrupts;
pub mod port;
pub mod random;
pub mod segmentation;
pub mod tables;
pub mod tlb;
#[cfg(feature = "inline_asm")]
pub mod tls;

/// Halts the CPU until the next interrupt arrives.
#[inline]
pub fn hlt() {
    unsafe {
        #[cfg(feature = "inline_asm")]
        asm!("hlt", options(nomem, nostack, preserves_flags));

        #[cfg(not(feature = "inline_asm"))]
        crate::asm::x86_64_asm_hlt();
    }
}

/// Executes the `nop` instructions, which performs no operation (i.e. does nothing).
///
/// This operation is useful to work around the LLVM bug that endless loops are illegally
/// optimized away (see [the issue](https://github.com/rust-lang/rust/issues/28728)). By invoking this
/// instruction (which is marked as volatile), the compiler should no longer optimize the
/// endless loop away.
#[inline]
pub fn nop() {
    unsafe {
        #[cfg(feature = "inline_asm")]
        asm!("nop", options(nomem, nostack, preserves_flags));

        #[cfg(not(feature = "inline_asm"))]
        crate::asm::x86_64_asm_nop();
    }
}

/// Emits a '[magic breakpoint](https://wiki.osdev.org/Bochs#Magic_Breakpoint)' instruction for the [Bochs](http://bochs.sourceforge.net/) CPU
/// emulator. Make sure to set `magic_break: enabled=1` in your `.bochsrc` file.
#[inline]
pub fn bochs_breakpoint() {
    unsafe {
        #[cfg(feature = "inline_asm")]
        asm!("xchg bx, bx", options(nomem, nostack, preserves_flags));

        #[cfg(not(feature = "inline_asm"))]
        crate::asm::x86_64_asm_bochs();
    }
}

/// Gets the current instruction pointer. Note that this is only approximate as it requires a few
/// instructions to execute.
#[cfg(feature = "inline_asm")]
#[cfg_attr(docsrs, doc(cfg(any(feature = "nightly", feature = "inline_asm"))))]
#[inline(always)]
pub fn read_rip() -> crate::VirtAddr {
    let rip: u64;
    unsafe {
        asm!("lea {}, [rip]", out(reg) rip, options(nostack, nomem, preserves_flags));
    }
    crate::VirtAddr::new(rip)
}
