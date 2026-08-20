//! Representations of various x86 specific structures and descriptor tables.

use crate::addr::{DefaultVirtAddrValidity, VirtAddrGeneric, VirtAddrValidity};

pub mod gdt;

pub mod idt;

#[cfg(feature = "memory_encryption")]
pub mod mem_encrypt;
pub mod paging;
pub mod port;
pub mod tss;

/// A struct describing a pointer to a descriptor table (GDT / IDT).
/// This is in a format suitable for giving to 'lgdt' or 'lidt'.
#[repr(C, packed(2))]
pub struct DescriptorTablePointer<V: VirtAddrValidity = DefaultVirtAddrValidity> {
    /// Size of the DT in bytes - 1.
    pub limit: u16,
    /// Pointer to the memory region containing the DT.
    pub base: VirtAddrGeneric<V>,
}

// These traits are implemented manually because Rust 1.59 has limited derive support for generic
// packed structs. They can use derive once the MSRV is raised to Rust 1.69.
impl<V: VirtAddrValidity> Copy for DescriptorTablePointer<V> {}

impl<V: VirtAddrValidity> Clone for DescriptorTablePointer<V> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<V: VirtAddrValidity> core::fmt::Debug for DescriptorTablePointer<V> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let limit = self.limit;
        let base = self.base;

        f.debug_struct("DescriptorTablePointer")
            .field("limit", &limit)
            .field("base", &base)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::VirtAddr;
    use std::mem::size_of;

    #[test]
    pub fn check_descriptor_pointer_size() {
        // Per the SDM, a descriptor pointer has to be 2+8=10 bytes
        assert_eq!(size_of::<DescriptorTablePointer>(), 10);
        // Make sure that we can reference a pointer's limit
        let p: DescriptorTablePointer = DescriptorTablePointer {
            limit: 5,
            base: VirtAddr::zero(),
        };
        let _: &u16 = &p.limit;

        let _: DescriptorTablePointer<crate::addr::DefaultVirtAddrValidity> = p;

        #[cfg(feature = "virt_addr_57")]
        assert_eq!(
            size_of::<DescriptorTablePointer<crate::addr::FixedValidity<57>>>(),
            10
        );
        #[cfg(feature = "virt_addr_rt")]
        assert_eq!(
            size_of::<DescriptorTablePointer<crate::addr::RuntimeValidity>>(),
            10
        );
    }
}
