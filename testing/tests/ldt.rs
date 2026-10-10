#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![reexport_test_harness_main = "test_main"]
#![test_runner(testing::test_runner)]

use core::panic::PanicInfo;
use core::ptr::addr_of;
use lazy_static::lazy_static;
use testing::{serial_print, serial_println};
use x86_64::instructions::segmentation::{Segment, CS, FS};
use x86_64::instructions::tables::{lldt, sldt};
use x86_64::registers::model_specific::FsBase;
use x86_64::structures::gdt::{
    Descriptor, DescriptorFlags, GlobalDescriptorTable, SegmentSelector,
};
use x86_64::PrivilegeLevel;

const LDT_BASE: u32 = 0x1234_5000;
const LDT_LIMIT: u16 = 15;

static mut LDT: [u64; 2] = [0, data_segment(LDT_BASE)];

lazy_static! {
    static ref GDT: (GlobalDescriptorTable, Selectors) = {
        let mut gdt = GlobalDescriptorTable::new();
        // Add an unused segment so we get a different value for CS
        gdt.append(Descriptor::kernel_data_segment());
        let code_selector = gdt.append(Descriptor::kernel_code_segment());
        let ldt_selector = gdt.append(ldt_segment(addr_of!(LDT) as u64));
        (
            gdt,
            Selectors {
                code_selector,
                ldt_selector,
            },
        )
    };
}

struct Selectors {
    code_selector: SegmentSelector,
    ldt_selector: SegmentSelector,
}

// Type 0b0010 is an LDT descriptor. It occupies two entries.
const fn ldt_segment(base: u64) -> Descriptor {
    let mut low = LDT_LIMIT as u64;
    low |= (base & 0xff_ffff) << 16;
    low |= 0b0010 << 40;
    low |= DescriptorFlags::PRESENT.bits();
    low |= ((base >> 24) & 0xff) << 56;
    Descriptor::SystemSegment(low, base >> 32)
}

// ACCESSED keeps the CPU from writing the LDT when `fs` is first loaded from it.
const fn data_segment(base: u32) -> u64 {
    let mut low = DescriptorFlags::KERNEL_DATA.bits() | DescriptorFlags::ACCESSED.bits();
    low |= 0xffff;
    low |= (base as u64 & 0xff_ffff) << 16;
    low |= (base as u64 >> 24) << 56;
    low
}

fn init() {
    GDT.0.load();
    unsafe { CS::set_reg(GDT.1.code_selector) };
}

#[no_mangle]
pub extern "C" fn _start() -> ! {
    init();
    test_main();

    loop {}
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    testing::test_panic_handler(info)
}

#[test_case]
fn lldt_and_sldt() {
    serial_print!("lldt_and_sldt... ");
    unsafe { lldt(GDT.1.ldt_selector) };
    assert_eq!(sldt(), GDT.1.ldt_selector);
    serial_println!("[ok]");
}

#[test_case]
fn lldt_does_not_modify_gdt() {
    serial_print!("lldt_does_not_modify_gdt... ");
    // Unlike `ltr`, `lldt` marks nothing in the GDT, which is why the same LDT
    // may be loaded on several CPUs.
    let index = usize::from(GDT.1.ldt_selector.index());
    let before = GDT.0.entries()[index].clone();
    unsafe { lldt(GDT.1.ldt_selector) };
    assert_eq!(GDT.0.entries()[index], before);
    serial_println!("[ok]");
}

#[test_case]
fn lldt_null() {
    serial_print!("lldt_null... ");
    unsafe {
        lldt(GDT.1.ldt_selector);
        lldt(SegmentSelector::NULL);
    }
    assert_eq!(sldt(), SegmentSelector::NULL);
    serial_println!("[ok]");
}

#[test_case]
fn ldt_segment_load() {
    serial_print!("ldt_segment_load... ");
    unsafe { lldt(GDT.1.ldt_selector) };

    // The LDT is only really loaded if a segment load through it finds the
    // descriptor there. `fs` is otherwise unused in this test binary.
    let data = SegmentSelector::new(1, PrivilegeLevel::Ring0);
    let data = SegmentSelector(data.0 | 0b100);
    unsafe { FS::set_reg(data) };
    assert_eq!(FS::get_reg(), data);
    assert_eq!(FsBase::read().as_u64(), u64::from(LDT_BASE));
    unsafe { FS::set_reg(SegmentSelector::NULL) };
    serial_println!("[ok]");
}
