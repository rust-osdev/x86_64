#![no_std]

extern crate spin;
extern crate uart_16550;
extern crate x86_64;
#[macro_use]
extern crate lazy_static;

pub mod serial;

use x86_64::instructions::port::Port;

pub unsafe fn exit_qemu() {
    let mut port = Port::<u32>::new(0xf4);
    port.write(0);
}
