//! Access to I/O ports

use core::marker::PhantomData;

/// A helper trait that implements the actual port operations.
///
/// On x86, I/O ports operate on either `u8` (via `inb`/`outb`), `u16` (via `inw`/`outw`),
/// or `u32` (via `inl`/`outl`). Therefore this trait is implemented for exactly these types.
pub trait PortReadWrite {
    /// Reads a `Self` value from the given port.
    ///
    /// This function is unsafe because the I/O port could have side effects that violate memory
    /// safety.
    unsafe fn read_from_port(port: u16) -> Self;

    /// Writes a `Self` value to the given port.
    ///
    /// This function is unsafe because the I/O port could have side effects that violate memory
    /// safety.
    unsafe fn write_to_port(port: u16, value: Self);
}

impl PortReadWrite for u8 {
    #[inline]
    unsafe fn read_from_port(port: u16) -> u8 {
        let value: u8;
        asm!("inb %dx, %al" : "={ax}"(value) : "{dx}"(port) :: "volatile");
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
        asm!("outw %ax, %dx" :: "{dx}"(port), "{al}"(value) :: "volatile");
    }
}

impl PortReadWrite for u32 {
    #[inline]
    unsafe fn read_from_port(port: u16) -> u32 {
        let value: u32;
        asm!("inl %dx, %eax" : "={ax}"(value) : "{dx}"(port) :: "volatile");
        value
    }

    #[inline]
    unsafe fn write_to_port(port: u16, value: u32) {
        asm!("outl %eax, %dx" :: "{dx}"(port), "{al}"(value) :: "volatile");
    }
}

/// An I/O port.
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
