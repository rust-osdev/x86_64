//! Access to I/O ports

use core::marker::PhantomData;

pub use structures::port::PortReadWrite;

impl PortReadWrite for u8 {
    #[inline]
    unsafe fn read_from_port(port: u16) -> u8 {
        let value: u8;
        asm!("inb %dx, %al" : "={al}"(value) : "{dx}"(port) :: "volatile");
        value
    }

    #[inline]
    unsafe fn write_to_port(port: u16, value: u8) {
        asm!("outb %al, %dx" :: "{dx}"(port), "{al}"(value) :: "volatile");
    }
}

impl PortReadWrite for u16 {
    #[inline]
    unsafe fn read_from_port(port: u16) -> u16 {
        let value: u16;
        asm!("inw %dx, %ax" : "={ax}"(value) : "{dx}"(port) :: "volatile");
        value
    }

    #[inline]
    unsafe fn write_to_port(port: u16, value: u16) {
        asm!("outw %ax, %dx" :: "{dx}"(port), "{ax}"(value) :: "volatile");
    }
}

impl PortReadWrite for u32 {
    #[inline]
    unsafe fn read_from_port(port: u16) -> u32 {
        let value: u32;
        asm!("inl %dx, %eax" : "={eax}"(value) : "{dx}"(port) :: "volatile");
        value
    }

    #[inline]
    unsafe fn write_to_port(port: u16, value: u32) {
        asm!("outl %eax, %dx" :: "{dx}"(port), "{eax}"(value) :: "volatile");
    }
}

/// An I/O port.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Port<T: PortReadWrite> {
    port: u16,
    phantom: PhantomData<T>,
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
    pub unsafe fn read(&self) -> T {
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
