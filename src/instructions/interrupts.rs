//! Enabling and disabling interrupts

/// Returns whether interrupts are enabled.
pub fn are_enabled() -> bool {
    use registers::flags::{flags, Flags};

    flags().contains(Flags::IF)
}

/// Enable interrupts.
///
/// This is a wrapper around the `sti` instruction.
pub fn enable() {
    unsafe {
        asm!("sti");
    }
}

/// Disable interrupts.
///
/// This is a wrapper around the `cli` instruction.
pub fn disable() {
    unsafe {
        asm!("cli");
    }
}

/// Run a closue with disabled interrupts.
///
/// Run the given closure, disabling interrupts before running it (if they aren't already disabled).
/// Afterwards, interrupts are enabling again if they were enabled before.
///
/// If you have other `enable` and `disable` calls _within_ the closure, things may not work as expected.
///
/// # Examples
///
/// ```rust
/// // interrupts are enabled
/// without_interrupts(|| {
///     // interrupts are disabled
///     without_interrupts(|| {
///         // interrupts are disabled
///     });
///     // interrupts are still disabled
/// });
/// // interrupts are enabled again
/// ```
pub fn without_interrupts<F, R>(f: F) -> R
where
    F: FnOnce() -> R,
{
    // true if the interrupt flag is set (i.e. interrupts are enabled)
    let saved_intpt_flag = are_enabled();

    // if interrupts are enabled, disable them for now
    if saved_intpt_flag {
        disable();
    }

    // do `f` while interrupts are disabled
    let ret = f();

    // re-enable interrupts if they were previously enabled
    if saved_intpt_flag {
        enable();
    }

    // return the result of `f` to the caller
    ret
}
