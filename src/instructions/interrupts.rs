//! Enable and disable hardware interrupts.

/// Enable hardware interrupts using the `sti` instruction.
pub unsafe fn enable() {
    asm!("sti");
}

/// Disable hardware interrupts using the `cli` instruction.
pub unsafe fn disable() {
    asm!("cli");
}
