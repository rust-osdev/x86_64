//! Access to I/O ports

use core::fmt;
use core::marker::PhantomData;

pub use crate::structures::port::{PortRead, PortWrite};

impl PortRead for u8 {
    #[inline]
    unsafe fn read_from_port(port: u16) -> u8 {
        #[cfg(feature = "inline_asm")]
        {
            let value: u8;
            asm!("in al, dx", out("al") value, in("dx") port, options(nomem, nostack, preserves_flags));
            value
        }
        #[cfg(not(feature = "inline_asm"))]
        crate::asm::x86_64_asm_read_from_port_u8(port)
    }
}

impl PortRead for u16 {
    #[inline]
    unsafe fn read_from_port(port: u16) -> u16 {
        #[cfg(feature = "inline_asm")]
        {
            let value: u16;
            asm!("in ax, dx", out("ax") value, in("dx") port, options(nomem, nostack, preserves_flags));
            value
        }
        #[cfg(not(feature = "inline_asm"))]
        crate::asm::x86_64_asm_read_from_port_u16(port)
    }
}

impl PortRead for u32 {
    #[inline]
    unsafe fn read_from_port(port: u16) -> u32 {
        #[cfg(feature = "inline_asm")]
        {
            let value: u32;
            asm!("in eax, dx", out("eax") value, in("dx") port, options(nomem, nostack, preserves_flags));
            value
        }
        #[cfg(not(feature = "inline_asm"))]
        crate::asm::x86_64_asm_read_from_port_u32(port)
    }
}

impl PortWrite for u8 {
    #[inline]
    unsafe fn write_to_port(port: u16, value: u8) {
        #[cfg(feature = "inline_asm")]
        asm!("out dx, al", in("dx") port, in("al") value, options(nomem, nostack, preserves_flags));

        #[cfg(not(feature = "inline_asm"))]
        crate::asm::x86_64_asm_write_to_port_u8(port, value);
    }
}

impl PortWrite for u16 {
    #[inline]
    unsafe fn write_to_port(port: u16, value: u16) {
        #[cfg(feature = "inline_asm")]
        asm!("out dx, ax", in("dx") port, in("ax") value, options(nomem, nostack, preserves_flags));

        #[cfg(not(feature = "inline_asm"))]
        crate::asm::x86_64_asm_write_to_port_u16(port, value);
    }
}

impl PortWrite for u32 {
    #[inline]
    unsafe fn write_to_port(port: u16, value: u32) {
        #[cfg(feature = "inline_asm")]
        asm!("out dx, eax", in("dx") port, in("eax") value, options(nomem, nostack, preserves_flags));

        #[cfg(not(feature = "inline_asm"))]
        crate::asm::x86_64_asm_write_to_port_u32(port, value);
    }
}

/// An I/O port.
///
/// The port reads or writes values of type `T` and has read/write access
/// specified by the `R` and `W` const generic parameters.
///
/// Use the provided marker types or aliases to get a port type with the access you need:
/// * `PortGeneric<T, true, true>` -> `Port<T>`
/// * `PortGeneric<T, true, false>` -> `PortReadOnly<T>`
/// * `PortGeneric<T, false, true>` -> `PortWriteOnly<T>`
pub struct PortGeneric<T, const R: bool, const W: bool> {
    port: u16,
    phantom: PhantomData<T>,
}

/// A read-write I/O port.
pub type Port<T> = PortGeneric<T, true, true>;

/// A read-only I/O port.
pub type PortReadOnly<T> = PortGeneric<T, true, false>;

/// A write-only I/O port.
pub type PortWriteOnly<T> = PortGeneric<T, false, true>;

impl<T, const R: bool, const W: bool> PortGeneric<T, R, W> {
    /// Creates an I/O port with the given port number.
    #[inline]
    pub const fn new(port: u16) -> Self {
        PortGeneric {
            port,
            phantom: PhantomData,
        }
    }
}

impl<T: PortRead, const W: bool> PortGeneric<T, true, W> {
    /// Reads from the port.
    ///
    /// ## Safety
    ///
    /// This function is unsafe because the I/O port could have side effects that violate memory
    /// safety.
    #[inline]
    pub unsafe fn read(&mut self) -> T {
        T::read_from_port(self.port)
    }
}

impl<T: PortWrite, const R: bool> PortGeneric<T, R, true> {
    /// Writes to the port.
    ///
    /// ## Safety
    ///
    /// This function is unsafe because the I/O port could have side effects that violate memory
    /// safety.
    #[inline]
    pub unsafe fn write(&mut self, value: T) {
        T::write_to_port(self.port, value)
    }
}

impl<T, const R: bool, const W: bool> fmt::Debug for PortGeneric<T, R, W> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PortGeneric")
            .field("port", &self.port)
            .field("size", &core::mem::size_of::<T>())
            .field("read", &R)
            .field("write", &W)
            .finish()
    }
}

impl<T, const R: bool, const W: bool> Clone for PortGeneric<T, R, W> {
    fn clone(&self) -> Self {
        Self {
            port: self.port,
            phantom: PhantomData,
        }
    }
}

impl<T, const R: bool, const W: bool> PartialEq for PortGeneric<T, R, W> {
    fn eq(&self, other: &Self) -> bool {
        self.port == other.port
    }
}

impl<T, const R: bool, const W: bool> Eq for PortGeneric<T, R, W> {}
