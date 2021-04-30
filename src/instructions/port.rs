//! Access to I/O ports

use core::fmt;
use core::marker::PhantomData;

pub use crate::structures::port::{PortRead, PortWrite};

impl PortRead for u8 {
    #[cfg(feature = "inline_asm")]
    #[inline]
    unsafe fn read_from_port(port: u16) -> u8 {
        let value: u8;
        asm!("in al, dx", out("al") value, in("dx") port, options(nomem, nostack));
        value
    }

    #[cfg(not(feature = "inline_asm"))]
    #[inline]
    unsafe fn read_from_port(port: u16) -> u8 {
        crate::asm::x86_64_asm_read_from_port_u8(port)
    }
}

impl PortRead for u16 {
    #[cfg(feature = "inline_asm")]
    #[inline]
    unsafe fn read_from_port(port: u16) -> u16 {
        let value: u16;
        asm!("in ax, dx", out("ax") value, in("dx") port, options(nomem, nostack));
        value
    }

    #[cfg(not(feature = "inline_asm"))]
    #[inline]
    unsafe fn read_from_port(port: u16) -> u16 {
        crate::asm::x86_64_asm_read_from_port_u16(port)
    }
}

impl PortRead for u32 {
    #[cfg(feature = "inline_asm")]
    #[inline]
    unsafe fn read_from_port(port: u16) -> u32 {
        let value: u32;
        asm!("in eax, dx", out("eax") value, in("dx") port, options(nomem, nostack));
        value
    }

    #[cfg(not(feature = "inline_asm"))]
    #[inline]
    unsafe fn read_from_port(port: u16) -> u32 {
        crate::asm::x86_64_asm_read_from_port_u32(port)
    }
}

impl PortWrite for u8 {
    #[cfg(feature = "inline_asm")]
    #[inline]
    unsafe fn write_to_port(port: u16, value: u8) {
        asm!("out dx, al", in("dx") port, in("al") value, options(nomem, nostack));
    }

    #[cfg(not(feature = "inline_asm"))]
    #[inline]
    unsafe fn write_to_port(port: u16, value: u8) {
        crate::asm::x86_64_asm_write_to_port_u8(port, value)
    }
}

impl PortWrite for u16 {
    #[cfg(feature = "inline_asm")]
    #[inline]
    unsafe fn write_to_port(port: u16, value: u16) {
        asm!("out dx, ax", in("dx") port, in("ax") value, options(nomem, nostack));
    }

    #[cfg(not(feature = "inline_asm"))]
    #[inline]
    unsafe fn write_to_port(port: u16, value: u16) {
        crate::asm::x86_64_asm_write_to_port_u16(port, value)
    }
}

impl PortWrite for u32 {
    #[cfg(feature = "inline_asm")]
    #[inline]
    unsafe fn write_to_port(port: u16, value: u32) {
        asm!("out dx, eax", in("dx") port, in("eax") value, options(nomem, nostack));
    }

    #[cfg(not(feature = "inline_asm"))]
    #[inline]
    unsafe fn write_to_port(port: u16, value: u32) {
        crate::asm::x86_64_asm_write_to_port_u32(port, value)
    }
}

/// A marker trait for access types which allow reading port values.
pub trait PortReadAccess {}

/// A marker trait for access types which allow writing port values.
pub trait PortWriteAccess {}

/// An access marker type indicating that a port is only allowed to read values.
#[derive(Debug)]
pub struct ReadOnlyAccess {
    _priv: (),
}

impl PortReadAccess for ReadOnlyAccess {}

/// An access marker type indicating that a port is only allowed to write values.
#[derive(Debug)]
pub struct WriteOnlyAccess {
    _priv: (),
}

impl PortWriteAccess for WriteOnlyAccess {}

/// An access marker type indicating that a port is allowed to read or write values.
#[derive(Debug)]
pub struct ReadWriteAccess {
    _priv: (),
}

impl PortReadAccess for ReadWriteAccess {}

impl PortWriteAccess for ReadWriteAccess {}

/// An I/O port.
///
/// The port reads or writes values of type `T` and has read/write access specified by `A`.
///
/// Use the provided marker types or aliases to get a port type with the access you need:
/// * `Port<T, ReadWriteAccess>` -> `PortReadWrite<T>`
/// * `Port<T, ReadOnlyAccess>` -> `PortReadOnly<T>`
/// * `Port<T, WriteOnlyAccess>` -> `PortWriteOnly<T>`
pub struct Port<T, A> {
    port: u16,
    phantom: PhantomData<(T, A)>,
}

/// A read-write I/O port.
pub type PortReadWrite<T> = Port<T, ReadWriteAccess>;

/// A read-only I/O port.
pub type PortReadOnly<T> = Port<T, ReadOnlyAccess>;

/// A write-only I/O port.
pub type PortWriteOnly<T> = Port<T, WriteOnlyAccess>;

impl<T, A> Port<T, A> {
    /// Creates an I/O port with the given port number.
    #[inline]
    pub const fn new(port: u16) -> Port<T, A> {
        Port {
            port,
            phantom: PhantomData,
        }
    }
}

impl<T: PortRead, A: PortReadAccess> Port<T, A> {
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

impl<T: PortWrite, A: PortWriteAccess> Port<T, A> {
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

impl<T, A> fmt::Debug for Port<T, A> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Port").field("port", &self.port).finish()
    }
}

impl<T, A> Clone for Port<T, A> {
    fn clone(&self) -> Self {
        Self {
            port: self.port,
            phantom: PhantomData,
        }
    }
}

impl<T, A> PartialEq for Port<T, A> {
    fn eq(&self, other: &Self) -> bool {
        self.port == other.port
    }
}

impl<T, A> Eq for Port<T, A> {}
