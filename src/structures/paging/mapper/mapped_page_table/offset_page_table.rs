#![cfg(target_pointer_width = "64")]

use crate::addr::{FixedValidity, VirtAddrValidity};
use crate::structures::paging::{PageTable, mapper::*};

/// A Mapper implementation that requires that the complete physical memory is mapped at some
/// offset in the virtual address space.
pub type OffsetPageTable<'a, const BITS: usize = 48> = MappedPageTable<'a, PhysOffset, BITS>;

impl<'a, const BITS: usize> OffsetPageTable<'a, BITS>
where
    FixedValidity<BITS>: VirtAddrValidity,
{
    /// Creates a new fixed-width `OffsetPageTable` that uses the given offset
    /// for converting physical frame addresses to virtual pointers.
    ///
    /// The complete physical memory must be mapped in the active address space
    /// starting at `phys_offset`. A physical address `p` is accessed at the pointer value
    /// `phys_offset.wrapping_add(p)`. This is arithmetic on the raw `u64` value, not a
    /// promise that the result is canonical; every resulting pointer must still be valid
    /// and dereferenceable in the active address space.
    ///
    /// ## Safety
    ///
    /// This function is unsafe because the caller must guarantee that the passed `phys_offset`
    /// is correct. The passed table must point to the selected root of a valid page-table
    /// hierarchy (four levels for BITS = 48, five for BITS = 57). Otherwise this function might break memory safety, e.g.
    /// by writing to an illegal memory location.
    #[inline]
    pub unsafe fn from_phys_offset(root_table: &'a mut PageTable, phys_offset: u64) -> Self {
        let phys_offset = unsafe { PhysOffset::new(phys_offset) };
        unsafe { MappedPageTable::new(root_table, phys_offset) }
    }

    /// Returns the offset used for converting virtual to physical addresses.
    pub fn phys_offset(&self) -> u64 {
        self.page_table_frame_mapping().phys_offset()
    }
}

/// A [`PageTableFrameMapping`] implementation that requires that the complete physical memory is mapped at some
/// offset in the virtual address space.
#[derive(Debug)]
pub struct PhysOffset {
    phys_offset: u64,
}

impl PhysOffset {
    /// Creates a new `PhysOffset` that uses the given offset for converting virtual
    /// to physical addresses.
    ///
    /// `frame_to_pointer` computes each page-table pointer with
    /// `phys_offset.wrapping_add(frame_start)`. Wrapping is intentional arithmetic only;
    /// it does not make a non-canonical or unmapped result safe to dereference.
    ///
    /// ## Safety
    ///
    /// This function is unsafe because the caller must guarantee that the passed `phys_offset`
    /// makes every resulting pointer canonical, mapped, and dereferenceable in the active
    /// address space. The displacement uses `u64::wrapping_add`; wrapping does not provide any
    /// of those guarantees and an invalid offset can therefore cause memory unsafety.
    #[inline]
    pub unsafe fn new(phys_offset: u64) -> Self {
        Self { phys_offset }
    }

    /// Returns the offset used for converting virtual to physical addresses.
    pub fn phys_offset(&self) -> u64 {
        self.phys_offset
    }
}

unsafe impl PageTableFrameMapping for PhysOffset {
    fn frame_to_pointer(&self, frame: PhysFrame) -> *mut PageTable {
        let virt = self
            .phys_offset
            .wrapping_add(frame.start_address().as_u64());
        virt as *mut PageTable
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{PhysAddr, structures::paging::Size4KiB};

    #[test]
    fn phys_offset_addition_wraps_u64() {
        let offset = unsafe { PhysOffset::new(u64::MAX) };
        let frame = PhysFrame::<Size4KiB>::from_start_address(PhysAddr::new(0x1000)).unwrap();
        let pointer = offset.frame_to_pointer(frame) as usize as u64;
        assert_eq!(pointer, 0xfff);
    }
    #[test]
    fn offset_mapper_uses_its_fixed_root_width() {
        use crate::structures::paging::PageTableRootLevel;
        let mut root = PageTable::new();
        let mapper: OffsetPageTable<'_> =
            unsafe { OffsetPageTable::from_phys_offset(&mut root, 0) };
        assert_eq!(mapper.root_level(), PageTableRootLevel::Four);
        assert_eq!(mapper.phys_offset(), 0);
        #[cfg(feature = "virt_addr_57")]
        {
            let mut root = PageTable::new();
            let mapper = unsafe { OffsetPageTable::<57>::from_phys_offset(&mut root, 0) };
            assert_eq!(mapper.root_level(), PageTableRootLevel::Five);
            assert_eq!(mapper.phys_offset(), 0);
            assert_eq!(std::format!("{}", mapper.display()), "");
        }
    }
}
