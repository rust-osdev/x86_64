//! Access to I/O ports

use core::marker::PhantomData;

pub use crate::structures::port::{PortRead, PortReadWrite, PortWrite};

impl PortRead for u8 {
    #[cfg(feature = "inline_asm")]
    #[inline]
    unsafe fn read_from_port(port: u16) -> u8 {
        let value: u8;
        llvm_asm!("inb $1, $0" : "={al}"(value) : "N{dx}"(port) :: "volatile");
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
        llvm_asm!("inw $1, $0" : "={ax}"(value) : "N{dx}"(port) :: "volatile");
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
        llvm_asm!("inl $1, $0" : "={eax}"(value) : "N{dx}"(port) :: "volatile");
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
        llvm_asm!("outb $1, $0" :: "N{dx}"(port), "{al}"(value) :: "volatile");
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
        llvm_asm!("outw $1, $0" :: "N{dx}"(port), "{ax}"(value) :: "volatile");
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
        llvm_asm!("outl $1, $0" :: "N{dx}"(port), "{eax}"(value) :: "volatile");
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
    const_fn! {
        /// Creates a read only I/O port with the given port number.
        #[inline]
        pub fn new(port: u16) -> PortReadOnly<T> {
            PortReadOnly {
                port,
                phantom: PhantomData,
            }
        }
    }

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

impl<T: PortWrite> PortWriteOnly<T> {
    const_fn! {
        /// Creates a write only I/O port with the given port number.
        #[inline]
        pub fn new(port: u16) -> PortWriteOnly<T> {
            PortWriteOnly {
                port,
                phantom: PhantomData,
            }
        }
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

impl<T: PortReadWrite> Port<T> {
    const_fn! {
        /// Creates an I/O port with the given port number.
        #[inline]
        pub fn new(port: u16) -> Port<T> {
            Port {
                port,
                phantom: PhantomData,
            }
        }
    }

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
