#![cfg(target_pointer_width = "64")]

use crate::structures::paging::{mapper::*, PageTable};

/// A Mapper implementation that requires that the complete physical memory is mapped at some
/// offset in the virtual address space.
pub type OffsetPageTable<'a> = MappedPageTable<'a, PhysOffset>;

impl<'a> OffsetPageTable<'a> {
    /// Creates a new `OffsetPageTable` that uses the given offset for converting virtual
    /// to physical addresses.
    ///
    /// The complete physical memory must be mapped in the virtual address space starting at
    /// address `phys_offset`. This means that for example physical address `0x5000` can be
    /// accessed through virtual address `phys_offset + 0x5000`. This mapping is required because
    /// the mapper needs to access page tables, which are not mapped into the virtual address
    /// space by default.
    ///
    /// ## Safety
    ///
    /// This function is unsafe because the caller must guarantee that the passed `phys_offset`
    /// is correct. Also, the passed `level_4_table` must point to the level 4 page table
    /// of a valid page table hierarchy. Otherwise this function might break memory safety, e.g.
    /// by writing to an illegal memory location.
    #[inline]
    pub unsafe fn from_phys_offset(
        level_4_table: &'a mut PageTable,
        phys_offset: VirtAddr,
    ) -> Self {
        let phys_offset = unsafe { PhysOffset::new(phys_offset) };
        unsafe { MappedPageTable::new(level_4_table, phys_offset) }
    }

    /// Returns the offset used for converting virtual to physical addresses.
    pub fn phys_offset(&self) -> VirtAddr {
        self.page_table_frame_mapping().phys_offset()
    }
}

/// A [`PageTableFrameMapping`] implementation that requires that the complete physical memory is mapped at some
/// offset in the virtual address space.
#[derive(Debug)]
pub struct PhysOffset {
    phys_offset: VirtAddr,
}

impl PhysOffset {
    /// Creates a new `PhysOffset` that uses the given offset for converting virtual
    /// to physical addresses.
    ///
    /// The complete physical memory must be mapped in the virtual address space starting at
    /// address `phys_offset`. This means that for example physical address `0x5000` can be
    /// accessed through virtual address `phys_offset + 0x5000`. This mapping is required because
    /// the mapper needs to access page tables, which are not mapped into the virtual address
    /// space by default.
    ///
    /// ## Safety
    ///
    /// This function is unsafe because the caller must guarantee that the passed `phys_offset`
    /// is correct. Otherwise this function might break memory safety, e.g. by writing to an
    /// illegal memory location.
    #[inline]
    pub unsafe fn new(phys_offset: VirtAddr) -> Self {
        Self { phys_offset }
    }

    /// Returns the offset used for converting virtual to physical addresses.
    pub fn phys_offset(&self) -> VirtAddr {
        self.phys_offset
    }
}

unsafe impl PageTableFrameMapping for PhysOffset {
    fn frame_to_pointer(&self, frame: PhysFrame) -> *mut PageTable {
        let virt = self.phys_offset + frame.start_address().as_u64();
        virt.as_mut_ptr()
    }
}
