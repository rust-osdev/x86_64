//! Representations of various x86 specific structures and descriptor tables.

use crate::VirtPtr;
use core::fmt;

pub mod gdt;

pub mod idt;

pub mod paging;
pub mod port;
pub mod tss;

/// A struct describing a pointer to a descriptor table (GDT / IDT).
/// This is in a format suitable for giving to 'lgdt' or 'lidt'.
#[repr(C, packed(2))]
pub struct DescriptorTablePointer<T> {
    /// Size of the DT.
    pub limit: u16,
    /// Pointer to the memory region containing the DT.
    pub base: VirtPtr<T>,
}

impl<T> Copy for DescriptorTablePointer<T> {}
impl<T> Clone for DescriptorTablePointer<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> fmt::Debug for DescriptorTablePointer<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { limit, base } = *self;
        f.debug_struct("DescriptorTablePointer")
            .field("limit", &limit)
            .field("base", &base)
            .finish()
    }
}

/// A pointer to a [`GlobalDescriptorTable`](gdt::GlobalDescriptorTable)
pub type GdtPointer = DescriptorTablePointer<gdt::Entry>;
/// A pointer to a [`InterruptDescriptorTable`](idt::InterruptDescriptorTable)
pub type IdtPointer = DescriptorTablePointer<idt::InterruptDescriptorTable>;

#[cfg(test)]
mod tests {
    use super::*;
    use std::mem::size_of;

    #[test]
    pub fn check_descriptor_pointer_size() {
        // Per the SDM, a descriptor pointer has to be 2+8=10 bytes
        assert_eq!(size_of::<GdtPointer>(), 10);
        // Make sure that we can reference a pointer's limit
        let p = GdtPointer {
            limit: 5,
            base: VirtPtr::null(),
        };
        let _: &u16 = &p.limit;
    }
}
