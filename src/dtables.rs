//! Functions and data-structures to load descriptor tables.

use core::mem::size_of;

use irq::IdtEntry;
use VirtualAddress;

/// A struct describing a pointer to a descriptor table (GDT / IDT).
/// This is in a format suitable for giving to 'lgdt' or 'lidt'.
#[derive(Debug)]
#[repr(C, packed)]
pub struct DescriptorTablePointer {
    /// Size of the DT.
    pub limit: u16,
    /// Address of the memory region containing the DT.
    pub base: VirtualAddress,
}

impl DescriptorTablePointer {
    fn new<T>(slice: &[T]) -> Self {
        let len = slice.len() * size_of::<T>();
        assert!(len < 0x10000);
        DescriptorTablePointer {
            base: VirtualAddress::from(slice.as_ptr() as usize),
            limit: len as u16,
        }
    }
}

impl DescriptorTablePointer {
    pub fn new_idtp(idt: &[IdtEntry]) -> Self {
        Self::new(idt)
    }
}

/// Load GDT table.
pub unsafe fn lgdt(gdt: &DescriptorTablePointer) {
    asm!("lgdt ($0)" :: "r" (gdt) : "memory");
}

/// Load LDT table.
pub unsafe fn lldt(ldt: &DescriptorTablePointer) {
    asm!("lldt ($0)" :: "r" (ldt) : "memory");
}

/// Load IDT table.
pub unsafe fn lidt(idt: &DescriptorTablePointer) {
    asm!("lidt ($0)" :: "r" (idt) : "memory");
}
