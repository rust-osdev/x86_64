//! Enabling and disabling interrupts

/// Returns whether interrupts are enabled.
pub fn are_enabled() -> bool {
    use crate::registers::rflags::{self, RFlags};

    rflags::read().contains(RFlags::INTERRUPT_FLAG)
}

/// Enable interrupts.
///
/// This is a wrapper around the `sti` instruction.
#[inline]
pub fn enable() {
    #[cfg(feature = "inline_asm")]
    unsafe {
        asm!("sti" :::: "volatile");
    }
    #[cfg(not(feature = "inline_asm"))]
    unsafe {
        crate::asm::x86_64_asm_interrupt_enable();
    }
}

/// Disable interrupts.
///
/// This is a wrapper around the `cli` instruction.
#[inline]
pub fn disable() {
    #[cfg(feature = "inline_asm")]
    unsafe {
        asm!("cli" :::: "volatile");
    }

    #[cfg(not(feature = "inline_asm"))]
    unsafe {
        crate::asm::x86_64_asm_interrupt_disable();
    }
}

/// Run a closure with disabled interrupts.
///
/// Run the given closure, disabling interrupts before running it (if they aren't already disabled).
/// Afterwards, interrupts are enabling again if they were enabled before.
///
/// If you have other `enable` and `disable` calls _within_ the closure, things may not work as expected.
///
/// # Examples
///
/// ```ignore
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

/// Cause a breakpoint exception by invoking the `int3` instruction.
#[inline]
pub fn int3() {
    #[cfg(feature = "inline_asm")]
    unsafe {
        asm!("int3" :::: "volatile");
    }

    #[cfg(not(feature = "inline_asm"))]
    unsafe {
        crate::asm::x86_64_asm_int3();
    }
}

/// Generate a software interrupt by invoking the `int` instruction.
///
/// This currently needs to be a macro because the `int` argument needs to be an
/// immediate. This macro will be replaced by a generic function when support for
/// const generics is implemented in Rust.
#[cfg(feature = "inline_asm")]
#[macro_export]
macro_rules! software_interrupt {
    ($x:expr) => {{
        asm!("int $0" :: "N" ($x) :: "volatile");
    }};
}

/// Not implemented
#[cfg(not(feature = "inline_asm"))]
#[macro_export]
macro_rules! software_interrupt {
    ($x:expr) => {{
        compile_error!("software_interrupt not implemented for non-nightly");
    }};
}
