//! Enabling and disabling interrupts

use registers::flags::{flags, Flags};

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

/// Run the given closure, disabling interrupts before running it (if they aren't already disabled)
/// and enabling interrupts afterwards if they were enabled before.
///
/// # Note
///
/// This function basically just does `disable`, runs the closure, the does `enable`. If you have
/// other `enable` and `disable` calls _within_ the closure, things may not work as expected.
pub fn without_interrupts<F, R>(f: F) -> R
where
    F: FnOnce() -> R,
{
    let already_enabled = flags().contains(Flags::IF);

    if already_enabled {
        disable();
    }

    let ret = f();

    if already_enabled {
        enable();
    }

    ret
}
