//! Enabling and disabling interrupts

/// Enable interrupts. This is a wrapper around `sti`.
pub fn enable() {
    unsafe {
        asm!("sti");
    }
}

/// Disable interrupts. This is a wrapper around `cli`.
pub fn disable() {
    unsafe {
        asm!("cli");
    }
}

/// Run the given closure, disabling interrupts before running it and enabling interrupts
/// afterwards.
///
/// # Note
///
/// This function basically just does `disable`, runs the closure, the does `enable`. If you have
/// other `enable` and `disable` calls _within_ the closure, things may not work as expected.
pub fn without_interrupts<F, R>(f: F) -> R
where
    F: FnOnce() -> R,
{
    disable();
    let ret = f();
    enable();
    ret
}
