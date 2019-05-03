//! Functions to read and write control registers.

pub use x86_64_types::registers::{Cr0, Cr3, Cr4};
use crate::structures::paging::PhysFrame;

impl super::RegReader for Cr0 {
    fn read() -> Self {
        let r: Self;
        unsafe { asm!("mov %cr0, $0" : "=r" (r)); }
        r
    }
}

impl super::RegWriter for Cr0 {
    unsafe fn write(self) {
        asm!("mov $0, %cr0" :: "r" (self) : "memory");
    }
}

/// Contains the Page Fault Linear Address (PFLA).
///
/// When page fault occurs, the CPU sets this register to the accessed address.
#[repr(transparent)]
#[derive(Copy, Clone, Debug)]
pub struct Cr2(u64);

impl From<Cr2> for crate::VirtAddr {
    fn from(cr2: Cr2) -> Self {
        Self::new(cr2.0)
    }
}

impl super::RegReader for Cr2 {
    fn read() -> Self {
        let r: Self;
        unsafe { asm!("mov %cr2, $0" : "=r" (r)); }
        r
    }
}

impl From<Cr3> for PhysFrame {
    fn from(cr3: Cr3) -> PhysFrame {
        PhysFrame::containing_address(crate::PhysAddr::new(cr3.pml4()))
    }
}

impl super::RegReader for Cr3 {
    fn read() -> Self {
        let r: Self;
        unsafe { asm!("mov %cr3, $0" : "=r" (r)); }
        r
    }
}

impl super::RegWriter for Cr3 {
    unsafe fn write(self) {
        asm!("mov $0, %cr3" :: "r" (self) : "memory");
    }
}

impl super::RegReader for Cr4 {
    fn read() -> Self {
        let r: Self;
        unsafe { asm!("mov %cr4, $0" : "=r" (r)); }
        r
    }
}

impl super::RegWriter for Cr4 {
    unsafe fn write(self) {
        asm!("mov $0, %cr4" :: "r" (self) : "memory");
    }
}
