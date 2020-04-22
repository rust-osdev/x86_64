//! Functions to flush the translation lookaside buffer (TLB).

use crate::VirtAddr;

/// Invalidate the given address in the TLB using the `invlpg` instruction.
#[inline]
pub fn flush(addr: VirtAddr) {
    #[cfg(feature = "inline_asm")]
    unsafe {
        llvm_asm!("invlpg ($0)" :: "r" (addr.as_u64()) : "memory")
    };

    #[cfg(not(feature = "inline_asm"))]
    unsafe {
        crate::asm::x86_64_asm_invlpg(addr.as_u64())
    };
}

/// Invalidate the TLB completely by reloading the CR3 register.
#[inline]
pub fn flush_all() {
    use crate::registers::control::Cr3;
    let (frame, flags) = Cr3::read();
    unsafe { Cr3::write(frame, flags) }
}
