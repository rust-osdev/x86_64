#![feature(const_fn)]
#![no_std] // don't link the Rust standard library
#![cfg_attr(not(test), no_main)] // disable all Rust-level entry points

#[cfg(not(test))]
use core::panic::PanicInfo;
use testing::{exit_qemu, serial_println};
use x86_64::instructions::port::{PortReadOnly, PortWriteOnly, Port};

// This port tells the data port which index to read from
const CRT_INDEX_PORT: u16 = 0x3D4;

// This port stores the data for the index set by the index port
const CRT_DATA_PORT: u16 = 0x3D5;

// The offset crt register is used because it's a port with no reserved
// bits that won't crash the system when written to
const OFFSET_REGISTER: u8 = 0x0A;

// A randomly chosen value to test againts
const TEST_VALUE: u8 = 0b10101010;

/// This function is the entry point, since the linker looks for a function
/// named `_start_` by default.
#[cfg(not(test))]
#[no_mangle] // don't mangle the name of this function
pub extern "C" fn _start() -> ! {
    let mut crt_index_port = PortWriteOnly::<u8>::new(CRT_INDEX_PORT);
    let mut crt_read_write_data_port = Port::<u8>::new(CRT_DATA_PORT);
    let mut crt_data_read_only_port = PortReadOnly::<u8>::new(CRT_DATA_PORT);

    unsafe {
        // Set the offset register as the index using PortWriteOnly
        crt_index_port.write(OFFSET_REGISTER);

        // Write the test value to the data port using Port
        crt_read_write_data_port.write(TEST_VALUE);

        // Read the test value using PortReadOnly
        let read_only_test_value = crt_data_read_only_port.read() & 0xFF;

        // Read the test value using PortReadWrite
        let read_write_test_value = crt_read_write_data_port.read() & 0xFF;
        
        if read_only_test_value != TEST_VALUE {
            panic!("PortReadOnly: {} does not match expected value {}", read_only_test_value, TEST_VALUE);
        }

        if read_write_test_value != TEST_VALUE {
            panic!("PortReadWrite: {} does not match expected value {}", read_write_test_value, TEST_VALUE);
        }
    }

    serial_println!("ok");

    unsafe {
        exit_qemu();
    }

    loop {}
}

/// This function is called on panic.
#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    serial_println!("failed");

    serial_println!("{}", info);

    unsafe {
        exit_qemu();
    }
    loop {}
}