//! Functions to flush the translation lookaside buffer (TLB).

/// Invalidate the given address in the TLB using the `invlpg` instruction.
///
/// # Safety
/// This function is unsafe as it causes a general protection fault (GP) if the current privilege
/// level is not 0.
pub unsafe fn flush(addr: usize) {
    asm!("invlpg ($0)" :: "r" (addr) : "memory");
}

/// Invalidate the TLB completely by reloading the CR3 register.
///
/// # Safety
/// This function is unsafe as it causes a general protection fault (GP) if the current privilege
/// level is not 0.
pub unsafe fn flush_all() {
    use super::controlregs::{cr3, cr3_write};
    cr3_write(cr3())
}
