//! Functions and data-structures to load descriptor tables.
use core::fmt;
use core::mem::size_of;

/// A struct describing a pointer to a descriptor table (GDT / IDT).
/// This is in a format suitable for giving to 'lgdt' or 'lidt'.
#[repr(C, packed)]
pub struct DescriptorTablePointer<Entry> {
    /// Size of the DT.
    pub limit: u16,
    /// Pointer to the memory region containing the DT.
    pub base: *const Entry,
}

impl<T> DescriptorTablePointer<T> {
    pub fn new(tbl: &T) -> Self {
        // GDT, LDT, and IDT all expect the limit to be set to "one less".
        // See Intel 3a, Section 3.5.1 "Segment Descriptor Tables" and
        // Section 6.10 "Interrupt Descriptor Table (IDT)".
        let len = size_of::<T>() - 1;
        assert!(len < 0x10000);
        DescriptorTablePointer {
            base: tbl as *const T,
            limit: len as u16,
        }
    }

    pub fn new_from_slice(slice: &[T]) -> Self {
        // GDT, LDT, and IDT all expect the limit to be set to "one less".
        // See Intel 3a, Section 3.5.1 "Segment Descriptor Tables" and
        // Section 6.10 "Interrupt Descriptor Table (IDT)".
        let len = slice.len() * size_of::<T>() - 1;
        assert!(len < 0x10000);
        DescriptorTablePointer {
            base: slice.as_ptr(),
            limit: len as u16,
        }
    }
}

impl<T> fmt::Debug for DescriptorTablePointer<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        unsafe { write!(f, "DescriptorTablePointer ({} {:?})", self.limit, self.base) }
    }
}

/// Load GDT table with 32bit descriptors
pub unsafe fn lgdt<T>(gdt: &DescriptorTablePointer<T>) {
    asm!("lgdt ($0)" :: "r" (gdt) : "memory");
}

/// Load LDT table with 32bit descriptors.
pub unsafe fn lldt<T>(ldt: &DescriptorTablePointer<T>) {
    asm!("lldt ($0)" :: "r" (ldt) : "memory");
}

/// Load IDT table with 32bit descriptors.
pub unsafe fn lidt<T>(idt: &DescriptorTablePointer<T>) {
    asm!("lidt ($0)" :: "r" (idt) : "memory");
}
