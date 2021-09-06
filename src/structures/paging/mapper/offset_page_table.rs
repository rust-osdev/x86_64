#![cfg(target_pointer_width = "64")]

use crate::structures::paging::{
    frame::PhysFrame, mapper::*, page::PageRangeInclusive, page_table::PageTable, FrameDeallocator,
    Page, PageTableFlags,
};

/// A Mapper implementation that requires that the complete physically memory is mapped at some
/// offset in the virtual address space.
#[derive(Debug)]
pub struct OffsetPageTable<'a> {
    inner: MappedPageTable<'a, PhysOffset>,
}

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
    pub unsafe fn new(level_4_table: &'a mut PageTable, phys_offset: VirtAddr) -> Self {
        let phys_offset = PhysOffset {
            offset: phys_offset,
        };
        Self {
            inner: unsafe { MappedPageTable::new(level_4_table, phys_offset) },
        }
    }

    /// Returns a mutable reference to the wrapped level 4 `PageTable` instance.
    pub fn level_4_table(&mut self) -> &mut PageTable {
        self.inner.level_4_table()
    }
}

#[derive(Debug)]
struct PhysOffset {
    offset: VirtAddr,
}

unsafe impl PageTableFrameMapping for PhysOffset {
    fn frame_to_pointer(&self, frame: PhysFrame) -> *mut PageTable {
        let virt = self.offset + frame.start_address().as_u64();
        virt.as_mut_ptr()
    }
}

// delegate all trait implementations to inner

impl<'a> Mapper<Size1GiB> for OffsetPageTable<'a> {
    #[inline]
    unsafe fn map_to_with_table_flags<A>(
        &mut self,
        page: Page<Size1GiB>,
        frame: PhysFrame<Size1GiB>,
        flags: PageTableFlags,
        parent_table_flags: PageTableFlags,
        allocator: &mut A,
    ) -> Result<MapperFlush<Size1GiB>, MapToError<Size1GiB>>
    where
        A: FrameAllocator<Size4KiB> + ?Sized,
    {
        unsafe {
            self.inner
                .map_to_with_table_flags(page, frame, flags, parent_table_flags, allocator)
        }
    }

    #[cfg(feature = "experimental")]
    #[inline]
    unsafe fn map_to_range_with_table_flags<A>(
        &mut self,
        pages: PageRange<Size1GiB>,
        frames: PhysFrameRange<Size1GiB>,
        flags: PageTableFlags,
        parent_table_flags: PageTableFlags,
        allocator: &mut A,
    ) -> Result<MapperFlushRange<Size1GiB>, (MapToError<Size1GiB>, MapperFlushRange<Size1GiB>)>
    where
        Self: Sized,
        A: FrameAllocator<Size4KiB> + ?Sized,
    {
        unsafe {
            self.inner.map_to_range_with_table_flags(
                pages,
                frames,
                flags,
                parent_table_flags,
                allocator,
            )
        }
    }

    #[cfg(feature = "experimental")]
    #[inline]
    unsafe fn map_range_with_table_flags<A>(
        &mut self,
        pages: PageRange<Size1GiB>,
        flags: PageTableFlags,
        parent_table_flags: PageTableFlags,
        allocator: &mut A,
    ) -> Result<MapperFlushRange<Size1GiB>, (MapToError<Size1GiB>, MapperFlushRange<Size1GiB>)>
    where
        A: FrameAllocator<Size4KiB> + FrameAllocator<Size1GiB> + ?Sized,
    {
        unsafe {
            self.inner
                .map_range_with_table_flags(pages, flags, parent_table_flags, allocator)
        }
    }

    #[inline]
    fn unmap(
        &mut self,
        page: Page<Size1GiB>,
    ) -> Result<(PhysFrame<Size1GiB>, MapperFlush<Size1GiB>), UnmapError> {
        self.inner.unmap(page)
    }

    #[cfg(feature = "experimental")]
    #[inline]
    unsafe fn unmap_range<D>(
        &mut self,
        pages: PageRange<Size1GiB>,
        deallocator: &mut D,
    ) -> Result<MapperFlushRange<Size1GiB>, (UnmapError, MapperFlushRange<Size1GiB>)>
    where
        Self: Sized,
        D: FrameDeallocator<Size1GiB> + ?Sized,
    {
        unsafe { self.inner.unmap_range(pages, deallocator) }
    }

    #[inline]
    unsafe fn update_flags(
        &mut self,
        page: Page<Size1GiB>,
        flags: PageTableFlags,
    ) -> Result<MapperFlush<Size1GiB>, FlagUpdateError> {
        unsafe { self.inner.update_flags(page, flags) }
    }

    #[inline]
    unsafe fn set_flags_p4_entry(
        &mut self,
        page: Page<Size1GiB>,
        flags: PageTableFlags,
    ) -> Result<MapperFlushAll, FlagUpdateError> {
        unsafe { self.inner.set_flags_p4_entry(page, flags) }
    }

    #[inline]
    unsafe fn set_flags_p3_entry(
        &mut self,
        page: Page<Size1GiB>,
        flags: PageTableFlags,
    ) -> Result<MapperFlushAll, FlagUpdateError> {
        unsafe { self.inner.set_flags_p3_entry(page, flags) }
    }

    #[inline]
    unsafe fn set_flags_p2_entry(
        &mut self,
        page: Page<Size1GiB>,
        flags: PageTableFlags,
    ) -> Result<MapperFlushAll, FlagUpdateError> {
        unsafe { self.inner.set_flags_p2_entry(page, flags) }
    }

    #[inline]
    fn translate_page(&self, page: Page<Size1GiB>) -> Result<PhysFrame<Size1GiB>, TranslateError> {
        self.inner.translate_page(page)
    }
}

impl<'a> Mapper<Size2MiB> for OffsetPageTable<'a> {
    #[inline]
    unsafe fn map_to_with_table_flags<A>(
        &mut self,
        page: Page<Size2MiB>,
        frame: PhysFrame<Size2MiB>,
        flags: PageTableFlags,
        parent_table_flags: PageTableFlags,
        allocator: &mut A,
    ) -> Result<MapperFlush<Size2MiB>, MapToError<Size2MiB>>
    where
        A: FrameAllocator<Size4KiB> + ?Sized,
    {
        unsafe {
            self.inner
                .map_to_with_table_flags(page, frame, flags, parent_table_flags, allocator)
        }
    }

    #[cfg(feature = "experimental")]
    #[inline]
    unsafe fn map_to_range_with_table_flags<A>(
        &mut self,
        pages: PageRange<Size2MiB>,
        frames: PhysFrameRange<Size2MiB>,
        flags: PageTableFlags,
        parent_table_flags: PageTableFlags,
        allocator: &mut A,
    ) -> Result<MapperFlushRange<Size2MiB>, (MapToError<Size2MiB>, MapperFlushRange<Size2MiB>)>
    where
        Self: Sized,
        A: FrameAllocator<Size4KiB> + ?Sized,
    {
        unsafe {
            self.inner.map_to_range_with_table_flags(
                pages,
                frames,
                flags,
                parent_table_flags,
                allocator,
            )
        }
    }

    #[cfg(feature = "experimental")]
    #[inline]
    unsafe fn map_range_with_table_flags<A>(
        &mut self,
        pages: PageRange<Size2MiB>,
        flags: PageTableFlags,
        parent_table_flags: PageTableFlags,
        allocator: &mut A,
    ) -> Result<MapperFlushRange<Size2MiB>, (MapToError<Size2MiB>, MapperFlushRange<Size2MiB>)>
    where
        A: FrameAllocator<Size4KiB> + FrameAllocator<Size2MiB> + ?Sized,
    {
        unsafe {
            self.inner
                .map_range_with_table_flags(pages, flags, parent_table_flags, allocator)
        }
    }

    #[inline]
    fn unmap(
        &mut self,
        page: Page<Size2MiB>,
    ) -> Result<(PhysFrame<Size2MiB>, MapperFlush<Size2MiB>), UnmapError> {
        self.inner.unmap(page)
    }

    #[cfg(feature = "experimental")]
    #[inline]
    unsafe fn unmap_range<D>(
        &mut self,
        pages: PageRange<Size2MiB>,
        deallocator: &mut D,
    ) -> Result<MapperFlushRange<Size2MiB>, (UnmapError, MapperFlushRange<Size2MiB>)>
    where
        Self: Sized,
        D: FrameDeallocator<Size2MiB> + ?Sized,
    {
        unsafe { self.inner.unmap_range(pages, deallocator) }
    }

    #[inline]
    unsafe fn update_flags(
        &mut self,
        page: Page<Size2MiB>,
        flags: PageTableFlags,
    ) -> Result<MapperFlush<Size2MiB>, FlagUpdateError> {
        unsafe { self.inner.update_flags(page, flags) }
    }

    #[inline]
    unsafe fn set_flags_p4_entry(
        &mut self,
        page: Page<Size2MiB>,
        flags: PageTableFlags,
    ) -> Result<MapperFlushAll, FlagUpdateError> {
        unsafe { self.inner.set_flags_p4_entry(page, flags) }
    }

    #[inline]
    unsafe fn set_flags_p3_entry(
        &mut self,
        page: Page<Size2MiB>,
        flags: PageTableFlags,
    ) -> Result<MapperFlushAll, FlagUpdateError> {
        unsafe { self.inner.set_flags_p3_entry(page, flags) }
    }

    #[inline]
    unsafe fn set_flags_p2_entry(
        &mut self,
        page: Page<Size2MiB>,
        flags: PageTableFlags,
    ) -> Result<MapperFlushAll, FlagUpdateError> {
        unsafe { self.inner.set_flags_p2_entry(page, flags) }
    }

    #[inline]
    fn translate_page(&self, page: Page<Size2MiB>) -> Result<PhysFrame<Size2MiB>, TranslateError> {
        self.inner.translate_page(page)
    }
}

impl<'a> Mapper<Size4KiB> for OffsetPageTable<'a> {
    #[inline]
    unsafe fn map_to_with_table_flags<A>(
        &mut self,
        page: Page<Size4KiB>,
        frame: PhysFrame<Size4KiB>,
        flags: PageTableFlags,
        parent_table_flags: PageTableFlags,
        allocator: &mut A,
    ) -> Result<MapperFlush<Size4KiB>, MapToError<Size4KiB>>
    where
        A: FrameAllocator<Size4KiB> + ?Sized,
    {
        unsafe {
            self.inner
                .map_to_with_table_flags(page, frame, flags, parent_table_flags, allocator)
        }
    }

    #[cfg(feature = "experimental")]
    #[inline]
    unsafe fn map_to_range_with_table_flags<A>(
        &mut self,
        pages: PageRange<Size4KiB>,
        frames: PhysFrameRange<Size4KiB>,
        flags: PageTableFlags,
        parent_table_flags: PageTableFlags,
        allocator: &mut A,
    ) -> Result<MapperFlushRange<Size4KiB>, (MapToError<Size4KiB>, MapperFlushRange<Size4KiB>)>
    where
        Self: Sized,
        A: FrameAllocator<Size4KiB> + ?Sized,
    {
        unsafe {
            self.inner.map_to_range_with_table_flags(
                pages,
                frames,
                flags,
                parent_table_flags,
                allocator,
            )
        }
    }

    #[cfg(feature = "experimental")]
    #[inline]
    unsafe fn map_range_with_table_flags<A>(
        &mut self,
        pages: PageRange<Size4KiB>,
        flags: PageTableFlags,
        parent_table_flags: PageTableFlags,
        allocator: &mut A,
    ) -> Result<MapperFlushRange<Size4KiB>, (MapToError<Size4KiB>, MapperFlushRange<Size4KiB>)>
    where
        A: FrameAllocator<Size4KiB> + ?Sized,
    {
        unsafe {
            self.inner
                .map_range_with_table_flags(pages, flags, parent_table_flags, allocator)
        }
    }

    #[inline]
    fn unmap(
        &mut self,
        page: Page<Size4KiB>,
    ) -> Result<(PhysFrame<Size4KiB>, MapperFlush<Size4KiB>), UnmapError> {
        self.inner.unmap(page)
    }

    #[cfg(feature = "experimental")]
    #[inline]
    unsafe fn unmap_range<D>(
        &mut self,
        pages: PageRange<Size4KiB>,
        deallocator: &mut D,
    ) -> Result<MapperFlushRange<Size4KiB>, (UnmapError, MapperFlushRange<Size4KiB>)>
    where
        Self: Sized,
        D: FrameDeallocator<Size4KiB> + ?Sized,
    {
        unsafe { self.inner.unmap_range(pages, deallocator) }
    }

    #[inline]
    unsafe fn update_flags(
        &mut self,
        page: Page<Size4KiB>,
        flags: PageTableFlags,
    ) -> Result<MapperFlush<Size4KiB>, FlagUpdateError> {
        unsafe { self.inner.update_flags(page, flags) }
    }

    #[inline]
    unsafe fn set_flags_p4_entry(
        &mut self,
        page: Page<Size4KiB>,
        flags: PageTableFlags,
    ) -> Result<MapperFlushAll, FlagUpdateError> {
        unsafe { self.inner.set_flags_p4_entry(page, flags) }
    }

    #[inline]
    unsafe fn set_flags_p3_entry(
        &mut self,
        page: Page<Size4KiB>,
        flags: PageTableFlags,
    ) -> Result<MapperFlushAll, FlagUpdateError> {
        unsafe { self.inner.set_flags_p3_entry(page, flags) }
    }

    #[inline]
    unsafe fn set_flags_p2_entry(
        &mut self,
        page: Page<Size4KiB>,
        flags: PageTableFlags,
    ) -> Result<MapperFlushAll, FlagUpdateError> {
        unsafe { self.inner.set_flags_p2_entry(page, flags) }
    }

    #[inline]
    fn translate_page(&self, page: Page<Size4KiB>) -> Result<PhysFrame<Size4KiB>, TranslateError> {
        self.inner.translate_page(page)
    }
}

impl<'a> Translate for OffsetPageTable<'a> {
    #[inline]
    fn translate(&self, addr: VirtAddr) -> TranslateResult {
        self.inner.translate(addr)
    }
}

impl<'a> CleanUp for OffsetPageTable<'a> {
    #[inline]
    unsafe fn clean_up<D>(&mut self, frame_deallocator: &mut D)
    where
        D: FrameDeallocator<Size4KiB>,
    {
        unsafe { self.inner.clean_up(frame_deallocator) }
    }

    #[inline]
    unsafe fn clean_up_addr_range<D>(
        &mut self,
        range: PageRangeInclusive,
        frame_deallocator: &mut D,
    ) where
        D: FrameDeallocator<Size4KiB>,
    {
        unsafe { self.inner.clean_up_addr_range(range, frame_deallocator) }
    }
}
