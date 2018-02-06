//! Functions to flush the translation lookaside buffer (TLB).

use VirtAddr;

/// Invalidate the given address in the TLB using the `invlpg` instruction.
pub fn flush(addr: VirtAddr) {
    unsafe { asm!("invlpg ($0)" :: "r" (addr.as_u64()) : "memory") };
}

/// Invalidate the TLB completely by reloading the CR3 register.
pub fn flush_all() {
    use registers::control::Cr3;
    let (frame, flags) = Cr3::read();
    unsafe { Cr3::write(frame, flags) }
}
