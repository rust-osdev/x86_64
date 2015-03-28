/// Enable Interrupts.
pub unsafe fn enable() {
    asm!("sti");
}

/// Disable Interrupts.
pub unsafe fn disable() {
    asm!("cli");
}
