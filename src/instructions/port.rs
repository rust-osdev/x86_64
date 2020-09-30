//! Access to I/O ports

use core::marker::PhantomData;

pub use crate::structures::port::{PortRead, PortReadWrite, PortWrite};

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

impl PortReadWrite for u8 {}
impl PortReadWrite for u16 {}
impl PortReadWrite for u32 {}

/// A read only I/O port.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PortReadOnly<T> {
    port: u16,
    phantom: PhantomData<T>,
}

/// A write only I/O port.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PortWriteOnly<T> {
    port: u16,
    phantom: PhantomData<T>,
}

/// An I/O port.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Port<T> {
    port: u16,
    phantom: PhantomData<T>,
}

impl<T> PortReadOnly<T> {
    /// Creates a read only I/O port with the given port number.
    #[inline]
    pub const fn new(port: u16) -> PortReadOnly<T> {
        PortReadOnly {
            port,
            phantom: PhantomData,
        }
    }
}

impl<T: PortRead> PortReadOnly<T> {
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

impl<T> PortWriteOnly<T> {
    /// Creates a write only I/O port with the given port number.
    #[inline]
    pub const fn new(port: u16) -> PortWriteOnly<T> {
        PortWriteOnly {
            port,
            phantom: PhantomData,
        }
    }
}

impl<T: PortWrite> PortWriteOnly<T> {
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

impl<T> Port<T> {
    /// Creates an I/O port with the given port number.
    #[inline]
    pub const fn new(port: u16) -> Port<T> {
        Port {
            port,
            phantom: PhantomData,
        }
    }
}

impl<T: PortReadWrite> Port<T> {
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
