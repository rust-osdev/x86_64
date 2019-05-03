//! Access to various system and model specific registers.

#![cfg(target_arch = "x86_64")]

pub mod control;
pub mod rflags;
pub mod msr;

/// A special register that can be read.
pub trait RegReader {
    /// Reads a special register.
    fn read() -> Self;
}

/// A special register that can be read.
pub trait RegWriter {
    /// Writes a special register.
    ///
    /// This function is unsafe because it can change the fundamental execution
    /// environment (such as the stack or paging).
    unsafe fn write(self);
}
