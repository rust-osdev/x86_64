//! Access to I/O ports

use core::marker::PhantomData;

pub use crate::structures::port::{PortRead, PortReadWrite, PortWrite};

impl PortRead for u8 {
    #[inline]
    unsafe fn read_from_port(port: u16) -> u8 {
        let value: u8;
        asm!("inb %dx, %al" : "={al}"(value) : "{dx}"(port) :: "volatile");
        value
    }
}

impl PortRead for u16 {
    #[inline]
    unsafe fn read_from_port(port: u16) -> u16 {
        let value: u16;
        asm!("inw %dx, %ax" : "={ax}"(value) : "{dx}"(port) :: "volatile");
        value
    }
}

impl PortRead for u32 {
    #[inline]
    unsafe fn read_from_port(port: u16) -> u32 {
        let value: u32;
        asm!("inl %dx, %eax" : "={eax}"(value) : "{dx}"(port) :: "volatile");
        value
    }
}

impl PortWrite for u8 {
    #[inline]
    unsafe fn write_to_port(port: u16, value: u8) {
        asm!("outb %al, %dx" :: "{dx}"(port), "{al}"(value) :: "volatile");
    }
}

impl PortWrite for u16 {
    #[inline]
    unsafe fn write_to_port(port: u16, value: u16) {
        asm!("outw %ax, %dx" :: "{dx}"(port), "{ax}"(value) :: "volatile");
    }
}

impl PortWrite for u32 {
    #[inline]
    unsafe fn write_to_port(port: u16, value: u32) {
        asm!("outl %eax, %dx" :: "{dx}"(port), "{eax}"(value) :: "volatile");
    }
}

impl PortReadWrite for u8 {}
impl PortReadWrite for u16 {}
impl PortReadWrite for u32 {}

/// A read only I/O port.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PortReadOnly<T: PortRead> {
    port: u16,
    phantom: PhantomData<T>,
}

/// A write only I/O port.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PortWriteOnly<T: PortWrite> {
    port: u16,
    phantom: PhantomData<T>,
}

/// An I/O port.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Port<T: PortReadWrite> {
    port: u16,
    phantom: PhantomData<T>,
}

impl<T: PortRead> PortReadOnly<T> {
    /// Creates a read only I/O port with the given port number.
    pub const fn new(port: u16) -> PortReadOnly<T> {
        PortReadOnly {
            port: port,
            phantom: PhantomData,
        }
    }

    /// Reads from the port.
    ///
    /// This function is unsafe because the I/O port could have side effects that violate memory
    /// safety.
    #[inline]
    pub unsafe fn read(&mut self) -> T {
        T::read_from_port(self.port)
    }
}

impl<T: PortWrite> PortWriteOnly<T> {
    /// Creates a write only I/O port with the given port number.
    pub const fn new(port: u16) -> PortWriteOnly<T> {
        PortWriteOnly {
            port: port,
            phantom: PhantomData,
        }
    }

    /// Writes to the port.
    ///
    /// This function is unsafe because the I/O port could have side effects that violate memory
    /// safety.
    #[inline]
    pub unsafe fn write(&mut self, value: T) {
        T::write_to_port(self.port, value)
    }
}

impl<T: PortReadWrite> Port<T> {
    /// Creates an I/O port with the given port number.
    pub const fn new(port: u16) -> Port<T> {
        Port {
            port: port,
            phantom: PhantomData,
        }
    }

    /// Reads from the port.
    ///
    /// This function is unsafe because the I/O port could have side effects that violate memory
    /// safety.
    #[inline]
    pub unsafe fn read(&mut self) -> T {
        T::read_from_port(self.port)
    }

    /// Writes to the port.
    ///
    /// This function is unsafe because the I/O port could have side effects that violate memory
    /// safety.
    #[inline]
    pub unsafe fn write(&mut self, value: T) {
        T::write_to_port(self.port, value)
    }
}
