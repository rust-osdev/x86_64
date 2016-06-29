//! Functions and data-structures to load descriptor tables.

use core::mem::size_of;

use current::irq::IdtEntry;
use shared::segmentation::SegmentDescriptor;

/// A struct describing a pointer to a descriptor table (GDT / IDT).
/// This is in a format suitable for giving to 'lgdt' or 'lidt'.
#[derive(Debug)]
#[repr(C, packed)]
pub struct DescriptorTablePointer<Entry> {
    /// Size of the DT.
    pub limit: u16,
    /// Pointer to the memory region containing the DT.
    pub base: *const Entry,
}

impl<T> DescriptorTablePointer<T> {
    fn new(slice: &[T]) -> Self {
        let len = slice.len() * size_of::<T>();
        assert!(len < 0x10000);
        DescriptorTablePointer {
            base: slice.as_ptr(),
            limit: len as u16,
        }
    }
}

impl DescriptorTablePointer<SegmentDescriptor> {
    pub fn new_gdtp(gdt: &[SegmentDescriptor]) -> Self {
        let mut p = Self::new(gdt);
        p.limit -= 1;
        p
    }
    pub fn new_ldtp(ldt: &[SegmentDescriptor]) -> Self {
        Self::new(ldt)
    }
}

impl DescriptorTablePointer<IdtEntry> {
    pub fn new_idtp(idt: &[IdtEntry]) -> Self {
        Self::new(idt)
    }
}


/// Load GDT table.
pub unsafe fn lgdt(gdt: &DescriptorTablePointer<SegmentDescriptor>) {
    asm!("lgdt ($0)" :: "r" (gdt) : "memory");
}

/// Load LDT table.
pub unsafe fn lldt(ldt: &DescriptorTablePointer<SegmentDescriptor>) {
    asm!("lldt ($0)" :: "r" (ldt) : "memory");
}

/// Load IDT table.
pub unsafe fn lidt(idt: &DescriptorTablePointer<IdtEntry>) {
    asm!("lidt ($0)" :: "r" (idt) : "memory");
}
