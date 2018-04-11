//! Access to I/O ports

use core::marker::PhantomData;

pub trait PortReadWrite {
    unsafe fn read(port: u16) -> Self;
    unsafe fn write(port: u16, value: Self);
}

impl PortReadWrite for u8 {
    #[inline]
    unsafe fn read(port: u16) -> u8 {
        let value: u8;
        asm!("inb %dx, %al" : "={ax}"(value) : "{dx}"(port) :: "volatile");
        value
    }

    #[inline]
    unsafe fn write(port: u16, value: u8) {
        asm!("outb %al, %dx" :: "{dx}"(port), "{al}"(value) :: "volatile");
    }
}

impl PortReadWrite for u16 {
    #[inline]
    unsafe fn read(port: u16) -> u16 {
        let value: u16;
        asm!("inw %dx, %ax" : "={ax}"(value) : "{dx}"(port) :: "volatile");
        value
    }

    #[inline]
    unsafe fn write(port: u16, value: u16) {
        asm!("outw %ax, %dx" :: "{dx}"(port), "{al}"(value) :: "volatile");
    }
}

impl PortReadWrite for u32 {
    #[inline]
    unsafe fn read(port: u16) -> u32 {
        let value: u32;
        asm!("inl %dx, %eax" : "={ax}"(value) : "{dx}"(port) :: "volatile");
        value
    }

    #[inline]
    unsafe fn write(port: u16, value: u32) {
        asm!("outl %eax, %dx" :: "{dx}"(port), "{al}"(value) :: "volatile");
    }
}

pub struct Port<T: PortReadWrite> {
    port: u16,
    phantom: PhantomData<T>,
}

impl<T: PortReadWrite> Port<T> {
    pub const fn new(port: u16) -> Port<T> {
        Port {
            port: port,
            phantom: PhantomData,
        }
    }

    #[inline]
    pub unsafe fn read(&self) -> T {
        T::read(self.port)
    }

    #[inline]
    pub unsafe fn write(&mut self, value: T) {
        T::write(self.port, value)
    }
}