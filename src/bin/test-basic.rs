#![feature(panic_implementation)] // required for defining the panic handler
#![no_std] // don't link the Rust standard library
#![cfg_attr(not(test), no_main)] // disable all Rust-level entry points

extern crate x86_64;

use core::fmt::Write;
#[cfg(not(test))]
use core::panic::PanicInfo;
use x86_64::testing::{exit_qemu, serial};

/// This function is the entry point, since the linker looks for a function
/// named `_start_` by default.
#[cfg(not(test))]
#[no_mangle] // don't mangle the name of this function
pub extern "C" fn _start() -> ! {
    let mut serial = unsafe { serial() };
    writeln!(serial, "ok").unwrap();

    unsafe {
        exit_qemu();
    }
    loop {}
}

/// This function is called on panic.
#[cfg(not(test))]
#[panic_implementation]
#[no_mangle]
pub fn panic(info: &PanicInfo) -> ! {
    let mut serial = unsafe { serial() };
    writeln!(serial, "failed").unwrap();

    writeln!(serial, "{}", info).unwrap();

    unsafe {
        exit_qemu();
    }
    loop {}
}
