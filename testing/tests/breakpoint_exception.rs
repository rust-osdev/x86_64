#![feature(abi_x86_interrupt)]
#![no_std]
#![no_main]

use core::panic::PanicInfo;
use core::sync::atomic::{AtomicUsize, Ordering};
use lazy_static::lazy_static;
use testing::{exit_qemu, serial_print, serial_println, QemuExitCode};

static BREAKPOINT_HANDLER_CALLED: AtomicUsize = AtomicUsize::new(0);

#[no_mangle]
pub extern "C" fn _start() -> ! {
    serial_print!("exception_breakpoint... ");

    init_test_idt();

    // invoke a breakpoint exception
    x86_64::instructions::interrupts::int3();

    match BREAKPOINT_HANDLER_CALLED.load(Ordering::SeqCst) {
        1 => {
            serial_println!("[ok]");
            exit_qemu(QemuExitCode::Success);
        }
        0 => {
            serial_println!("[failed]");
            serial_println!("    Breakpoint handler was not called.");
            exit_qemu(QemuExitCode::Failed);
        }
        other => {
            serial_println!("[failed]");
            serial_println!("    Breakpoint handler was called {} times", other);
            exit_qemu(QemuExitCode::Failed);
        }
    }

    loop {}
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    testing::test_panic_handler(info)
}

use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};

lazy_static! {
    static ref TEST_IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        idt.breakpoint.set_handler_fn(breakpoint_handler);
        idt
    };
}

pub fn init_test_idt() {
    TEST_IDT.load();
}

extern "x86-interrupt" fn breakpoint_handler(_stack_frame: InterruptStackFrame) {
    BREAKPOINT_HANDLER_CALLED.fetch_add(1, Ordering::SeqCst);
}
