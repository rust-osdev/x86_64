#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![reexport_test_harness_main = "test_main"]
#![test_runner(testing::test_runner)]

use core::panic::PanicInfo;
use testing::{serial_print, serial_println};

#[no_mangle] // don't mangle the name of this function
pub extern "C" fn _start() -> ! {
    test_main();

    loop {}
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    testing::test_panic_handler(info)
}

#[test_case]
fn basic_boot() {
    serial_print!("basic_boot... ");
    assert_eq!(0, 0);
    serial_println!("[ok]");
}
