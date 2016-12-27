//! Functions to flush the translation lookaside buffer (TLB).

use VirtualAddress;

/// Invalidate the given address in the TLB using the `invlpg` instruction.
pub fn flush(addr: VirtualAddress) {
    unsafe { asm!("invlpg ($0)" :: "r" (addr) : "memory") };
}

/// Invalidate the TLB completely by reloading the CR3 register.
pub fn flush_all() {
    use control_regs::{cr3, cr3_write};
    unsafe { cr3_write(cr3()) }
}
