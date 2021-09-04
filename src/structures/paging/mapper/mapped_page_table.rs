use crate::structures::paging::{
    frame::PhysFrame,
    frame_alloc::{FrameAllocator, FrameDeallocator},
    mapper::*,
    page::{AddressNotAligned, Page, PageRangeInclusive, Size1GiB, Size2MiB, Size4KiB},
    page_table::{FrameError, PageTable, PageTableEntry, PageTableFlags, PageTableLevel},
    PageTableIndex,
};

/// A Mapper implementation that relies on a PhysAddr to VirtAddr conversion function.
///
/// This type requires that the all physical page table frames are mapped to some virtual
/// address. Normally, this is done by mapping the complete physical address space into
/// the virtual address space at some offset. Other mappings between physical and virtual
/// memory are possible too, as long as they can be calculated as an `PhysAddr` to
/// `VirtAddr` closure.
#[derive(Debug)]
pub struct MappedPageTable<'a, P: PageTableFrameMapping> {
    page_table_walker: PageTableWalker<P>,
    level_4_table: &'a mut PageTable,
}

impl<'a, P: PageTableFrameMapping> MappedPageTable<'a, P> {
    /// Creates a new `MappedPageTable` that uses the passed closure for converting virtual
    /// to physical addresses.
    ///
    /// ## Safety
    ///
    /// This function is unsafe because the caller must guarantee that the passed `page_table_frame_mapping`
    /// closure is correct. Also, the passed `level_4_table` must point to the level 4 page table
    /// of a valid page table hierarchy. Otherwise this function might break memory safety, e.g.
    /// by writing to an illegal memory location.
    #[inline]
    pub unsafe fn new(level_4_table: &'a mut PageTable, page_table_frame_mapping: P) -> Self {
        Self {
            level_4_table,
            page_table_walker: unsafe { PageTableWalker::new(page_table_frame_mapping) },
        }
    }

    /// Returns a mutable reference to the wrapped level 4 `PageTable` instance.
    pub fn level_4_table(&mut self) -> &mut PageTable {
        &mut self.level_4_table
    }

    /// Helper function for implementing Mapper. Safe to limit the scope of unsafe, see
    /// https://github.com/rust-lang/rfcs/pull/2585.
    fn map_to_1gib<A>(
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
        let p4 = &mut self.level_4_table;
        let p3 = self.page_table_walker.create_next_table(
            &mut p4[page.p4_index()],
            parent_table_flags,
            allocator,
        )?;

        if !p3[page.p3_index()].is_unused() {
            return Err(MapToError::PageAlreadyMapped(frame));
        }
        p3[page.p3_index()].set_addr(frame.start_address(), flags | PageTableFlags::HUGE_PAGE);

        Ok(MapperFlush::new(page))
    }

    /// Helper function for implementing Mapper. Safe to limit the scope of unsafe, see
    /// https://github.com/rust-lang/rfcs/pull/2585.
    fn map_to_2mib<A>(
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
        let p4 = &mut self.level_4_table;
        let p3 = self.page_table_walker.create_next_table(
            &mut p4[page.p4_index()],
            parent_table_flags,
            allocator,
        )?;
        let p2 = self.page_table_walker.create_next_table(
            &mut p3[page.p3_index()],
            parent_table_flags,
            allocator,
        )?;

        if !p2[page.p2_index()].is_unused() {
            return Err(MapToError::PageAlreadyMapped(frame));
        }
        p2[page.p2_index()].set_addr(frame.start_address(), flags | PageTableFlags::HUGE_PAGE);

        Ok(MapperFlush::new(page))
    }

    /// Helper function for implementing Mapper. Safe to limit the scope of unsafe, see
    /// https://github.com/rust-lang/rfcs/pull/2585.
    fn map_to_4kib<A>(
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
        let p4 = &mut self.level_4_table;
        let p3 = self.page_table_walker.create_next_table(
            &mut p4[page.p4_index()],
            parent_table_flags,
            allocator,
        )?;
        let p2 = self.page_table_walker.create_next_table(
            &mut p3[page.p3_index()],
            parent_table_flags,
            allocator,
        )?;
        let p1 = self.page_table_walker.create_next_table(
            &mut p2[page.p2_index()],
            parent_table_flags,
            allocator,
        )?;

        if !p1[page.p1_index()].is_unused() {
            return Err(MapToError::PageAlreadyMapped(frame));
        }
        p1[page.p1_index()].set_frame(frame, flags);

        Ok(MapperFlush::new(page))
    }

    #[inline]
    fn next_table_fn_create_next_table<'b, A>(
        (flags, allocator): &mut (PageTableFlags, &mut A),
        entry: &'b mut PageTableEntry,
        walker: &PageTableWalker<P>,
    ) -> Result<&'b mut PageTable, PageTableCreateError>
    where
        A: FrameAllocator<Size4KiB> + ?Sized,
    {
        walker
            .create_next_table(entry, *flags, *allocator)
            .map_err(Into::into)
    }

    #[inline]
    fn next_table_fn_next_table_mut<'b, T>(
        _: &mut T,
        entry: &'b mut PageTableEntry,
        walker: &PageTableWalker<P>,
    ) -> Result<&'b mut PageTable, PageTableWalkError> {
        walker.next_table_mut(entry)
    }

    fn modify_range_1gib<ModifyFn, ModifyInfo, Err, NextTableFn, NextTableFnErr>(
        &mut self,
        pages: PageRange<Size1GiB>,
        modify: ModifyFn,
        mut info: ModifyInfo,
        next_table: NextTableFn,
    ) -> Result<MapperFlushRange<Size1GiB>, (Err, MapperFlushRange<Size1GiB>)>
    where
        ModifyFn: Fn(&mut PageTableEntry, Page<Size1GiB>, &mut ModifyInfo) -> Result<(), Err>,
        NextTableFn: for<'b> Fn(
            &mut ModifyInfo,
            &'b mut PageTableEntry,
            &PageTableWalker<P>,
        ) -> Result<&'b mut PageTable, NextTableFnErr>,
        NextTableFnErr: Into<Err>,
    {
        if pages.is_empty() {
            return Ok(MapperFlushRange::empty());
        }

        let p4 = &mut self.level_4_table;
        let page_table_walker = &mut self.page_table_walker;

        (pages.start.p4_index().into()..=pages.end.p4_index().into())
            .map(PageTableIndex::new)
            .try_for_each(|p4_index| {
                let p4_start = Page::from_page_table_indices_1gib(p4_index, PageTableIndex::new(0));
                let p4_start = p4_start.max(pages.start);
                let p4_end = Page::from_page_table_indices_1gib(p4_index, PageTableIndex::new(511));
                let p4_end = p4_end.min(pages.end);

                if p4_start == p4_end {
                    return Ok(());
                }

                let p3 = next_table(&mut info, &mut p4[p4_index], page_table_walker)
                    .map_err(|e| (e.into(), p4_start))?;

                let start_p3_index = p4_start.p3_index().into();
                let mut end_p3_index = p4_end.p3_index().into();

                if p4_end != pages.end {
                    end_p3_index += 1;
                }

                (start_p3_index..end_p3_index)
                    .map(PageTableIndex::new)
                    .map(move |p3_index| Page::from_page_table_indices_1gib(p4_index, p3_index))
                    .try_for_each(|page| {
                        let entry = &mut p3[page.p3_index()];
                        modify(entry, page, &mut info).map_err(|e| (e, page))
                    })
            })
            .map(|_| MapperFlushRange::new(pages))
            .map_err(|(e, page)| {
                (
                    e,
                    MapperFlushRange::new(PageRange {
                        start: pages.start,
                        end: page,
                    }),
                )
            })
    }

    #[inline]
    fn map_to_range_1gib<F, A>(
        &mut self,
        pages: PageRange<Size1GiB>,
        frames: F,
        flags: PageTableFlags,
        parent_table_flags: PageTableFlags,
        allocator: &mut A,
    ) -> Result<MapperFlushRange<Size1GiB>, (MapToError<Size1GiB>, MapperFlushRange<Size1GiB>)>
    where
        F: Fn(Page<Size1GiB>, &mut A) -> Option<PhysFrame<Size1GiB>>,
        A: FrameAllocator<Size4KiB> + ?Sized,
    {
        self.modify_range_1gib(
            pages,
            |entry, page, (_, allocator)| {
                let frame = frames(page, allocator).ok_or(MapToError::FrameAllocationFailed)?;
                if !entry.is_unused() {
                    return Err(MapToError::PageAlreadyMapped(frame));
                }
                entry.set_addr(frame.start_address(), flags | PageTableFlags::HUGE_PAGE);
                Ok(())
            },
            (parent_table_flags, allocator),
            Self::next_table_fn_create_next_table,
        )
    }

    fn modify_range_2mib<ModifyFn, ModifyInfo, Err, NextTableFn, NextTableFnErr>(
        &mut self,
        pages: PageRange<Size2MiB>,
        modify: ModifyFn,
        mut info: ModifyInfo,
        next_table: NextTableFn,
    ) -> Result<MapperFlushRange<Size2MiB>, (Err, MapperFlushRange<Size2MiB>)>
    where
        ModifyFn: Fn(&mut PageTableEntry, Page<Size2MiB>, &mut ModifyInfo) -> Result<(), Err>,
        NextTableFn: for<'b> Fn(
            &mut ModifyInfo,
            &'b mut PageTableEntry,
            &PageTableWalker<P>,
        ) -> Result<&'b mut PageTable, NextTableFnErr>,
        NextTableFnErr: Into<Err>,
    {
        if pages.is_empty() {
            return Ok(MapperFlushRange::empty());
        }

        let p4 = &mut self.level_4_table;
        let page_table_walker = &mut self.page_table_walker;

        (pages.start.p4_index().into()..=pages.end.p4_index().into())
            .map(PageTableIndex::new)
            .try_for_each(|p4_index| {
                let p4_start = Page::from_page_table_indices_2mib(
                    p4_index,
                    PageTableIndex::new(0),
                    PageTableIndex::new(0),
                );
                let p4_start = p4_start.max(pages.start);
                let p4_end = Page::from_page_table_indices_2mib(
                    p4_index,
                    PageTableIndex::new(511),
                    PageTableIndex::new(511),
                );
                let p4_end = p4_end.min(pages.end);

                if p4_start == p4_end {
                    return Ok(());
                }

                let p3 = next_table(&mut info, &mut p4[p4_index], page_table_walker)
                    .map_err(|e| (e.into(), p4_start))?;

                let start_p3_index = p4_start.p3_index();
                let end_p3_index = p4_end.p3_index();

                (start_p3_index.into()..=end_p3_index.into())
                    .map(PageTableIndex::new)
                    .try_for_each(|p3_index| {
                        let p3_start = Page::from_page_table_indices_2mib(
                            p4_index,
                            p3_index,
                            PageTableIndex::new(0),
                        );
                        let p3_start = p3_start.max(p4_start);
                        let p3_end = Page::from_page_table_indices_2mib(
                            p4_index,
                            p3_index,
                            PageTableIndex::new(511),
                        );
                        let p3_end = p3_end.min(p4_end);

                        if p3_start == p3_end {
                            return Ok(());
                        }

                        let p2 = next_table(&mut info, &mut p3[p3_index], page_table_walker)
                            .map_err(|e| (e.into(), p3_start))?;

                        let start_p2_index = p3_start.p2_index().into();
                        let mut end_p2_index = p3_end.p2_index().into();

                        if p3_end != pages.end {
                            end_p2_index += 1;
                        }

                        (start_p2_index..end_p2_index)
                            .map(PageTableIndex::new)
                            .map(move |p2_index| {
                                Page::from_page_table_indices_2mib(p4_index, p3_index, p2_index)
                            })
                            .try_for_each(|page| {
                                let entry = &mut p2[page.p2_index()];
                                modify(entry, page, &mut info).map_err(|e| (e, page))
                            })
                    })
            })
            .map(|_| MapperFlushRange::new(pages))
            .map_err(|(e, page)| {
                (
                    e,
                    MapperFlushRange::new(PageRange {
                        start: pages.start,
                        end: page,
                    }),
                )
            })
    }

    #[inline]
    fn map_to_range_2mib<F, A>(
        &mut self,
        pages: PageRange<Size2MiB>,
        frames: F,
        flags: PageTableFlags,
        parent_table_flags: PageTableFlags,
        allocator: &mut A,
    ) -> Result<MapperFlushRange<Size2MiB>, (MapToError<Size2MiB>, MapperFlushRange<Size2MiB>)>
    where
        F: Fn(Page<Size2MiB>, &mut A) -> Option<PhysFrame<Size2MiB>>,
        A: FrameAllocator<Size4KiB> + ?Sized,
    {
        self.modify_range_2mib(
            pages,
            |entry, page, (_, allocator)| {
                let frame = frames(page, allocator).ok_or(MapToError::FrameAllocationFailed)?;
                if !entry.is_unused() {
                    return Err(MapToError::PageAlreadyMapped(frame));
                }
                entry.set_addr(frame.start_address(), flags | PageTableFlags::HUGE_PAGE);
                Ok(())
            },
            (parent_table_flags, allocator),
            Self::next_table_fn_create_next_table,
        )
    }

    fn modify_range_4kib<ModifyFn, ModifyInfo, Err, NextTableFn, NextTableFnErr>(
        &mut self,
        pages: PageRange<Size4KiB>,
        modify: ModifyFn,
        mut info: ModifyInfo,
        next_table: NextTableFn,
    ) -> Result<MapperFlushRange<Size4KiB>, (Err, MapperFlushRange<Size4KiB>)>
    where
        ModifyFn: Fn(&mut PageTableEntry, Page<Size4KiB>, &mut ModifyInfo) -> Result<(), Err>,
        NextTableFn: for<'b> Fn(
            &mut ModifyInfo,
            &'b mut PageTableEntry,
            &PageTableWalker<P>,
        ) -> Result<&'b mut PageTable, NextTableFnErr>,
        NextTableFnErr: Into<Err>,
    {
        if pages.is_empty() {
            return Ok(MapperFlushRange::empty());
        }

        let p4 = &mut self.level_4_table;
        let page_table_walker = &mut self.page_table_walker;

        (pages.start.p4_index().into()..=pages.end.p4_index().into())
            .map(PageTableIndex::new)
            .try_for_each(|p4_index| {
                let p4_start = Page::from_page_table_indices(
                    p4_index,
                    PageTableIndex::new(0),
                    PageTableIndex::new(0),
                    PageTableIndex::new(0),
                );
                let p4_start = p4_start.max(pages.start);
                let p4_end = Page::from_page_table_indices(
                    p4_index,
                    PageTableIndex::new(511),
                    PageTableIndex::new(511),
                    PageTableIndex::new(511),
                );
                let p4_end = p4_end.min(pages.end);

                if p4_start == p4_end {
                    return Ok(());
                }

                let p3 = next_table(&mut info, &mut p4[p4_index], page_table_walker)
                    .map_err(|e| (e.into(), p4_start))?;

                let start_p3_index = p4_start.p3_index();
                let end_p3_index = p4_end.p3_index();

                (start_p3_index.into()..=end_p3_index.into())
                    .map(PageTableIndex::new)
                    .try_for_each(|p3_index| {
                        let p3_start = Page::from_page_table_indices(
                            p4_index,
                            p3_index,
                            PageTableIndex::new(0),
                            PageTableIndex::new(0),
                        );
                        let p3_start = p3_start.max(p4_start);
                        let p3_end = Page::from_page_table_indices(
                            p4_index,
                            p3_index,
                            PageTableIndex::new(511),
                            PageTableIndex::new(511),
                        );
                        let p3_end = p3_end.min(p4_end);

                        if p3_start == p3_end {
                            return Ok(());
                        }

                        let p2 = next_table(&mut info, &mut p3[p3_index], page_table_walker)
                            .map_err(|e| (e.into(), p3_start))?;

                        let start_p2_index = p3_start.p2_index();
                        let end_p2_index = p3_end.p2_index();

                        (start_p2_index.into()..=end_p2_index.into())
                            .map(PageTableIndex::new)
                            .try_for_each(|p2_index| {
                                let p2_start = Page::from_page_table_indices(
                                    p4_index,
                                    p3_index,
                                    p2_index,
                                    PageTableIndex::new(0),
                                );
                                let p2_start = p2_start.max(p4_start);
                                let p2_end = Page::from_page_table_indices(
                                    p4_index,
                                    p3_index,
                                    p2_index,
                                    PageTableIndex::new(511),
                                );
                                let p2_end = p2_end.min(p4_end);

                                if p2_start == p2_end {
                                    return Ok(());
                                }

                                let p1 =
                                    next_table(&mut info, &mut p2[p2_index], page_table_walker)
                                        .map_err(|e| (e.into(), p2_start))?;

                                let start_p1_index = p2_start.p1_index().into();
                                let mut end_p1_index = p2_end.p1_index().into();

                                if p2_end != pages.end {
                                    end_p1_index += 1;
                                }

                                (start_p1_index..end_p1_index)
                                    .map(PageTableIndex::new)
                                    .map(move |p1_index| {
                                        Page::from_page_table_indices(
                                            p4_index, p3_index, p2_index, p1_index,
                                        )
                                    })
                                    .try_for_each(|page| {
                                        let entry = &mut p1[page.p1_index()];
                                        modify(entry, page, &mut info).map_err(|e| (e, page))
                                    })
                            })
                    })
            })
            .map(|_| MapperFlushRange::new(pages))
            .map_err(|(e, page)| {
                (
                    e,
                    MapperFlushRange::new(PageRange {
                        start: pages.start,
                        end: page,
                    }),
                )
            })
    }

    #[inline]
    fn map_to_range_4kib<F, A>(
        &mut self,
        pages: PageRange<Size4KiB>,
        frames: F,
        flags: PageTableFlags,
        parent_table_flags: PageTableFlags,
        allocator: &mut A,
    ) -> Result<MapperFlushRange<Size4KiB>, (MapToError<Size4KiB>, MapperFlushRange<Size4KiB>)>
    where
        F: Fn(Page<Size4KiB>, &mut A) -> Option<PhysFrame<Size4KiB>>,
        A: FrameAllocator<Size4KiB> + ?Sized,
    {
        self.modify_range_4kib(
            pages,
            |entry, page, (_, allocator)| {
                let frame = frames(page, allocator).ok_or(MapToError::FrameAllocationFailed)?;
                if !entry.is_unused() {
                    return Err(MapToError::PageAlreadyMapped(frame));
                }
                entry.set_addr(frame.start_address(), flags);
                Ok(())
            },
            (parent_table_flags, allocator),
            Self::next_table_fn_create_next_table,
        )
    }
}

impl<'a, P: PageTableFrameMapping> Mapper<Size1GiB> for MappedPageTable<'a, P> {
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
        self.map_to_1gib(page, frame, flags, parent_table_flags, allocator)
    }

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
        assert_eq!(pages.count(), frames.count());
        self.map_to_range_1gib(
            pages,
            |page, _| {
                let offset = page - pages.start;
                Some(frames.start + offset)
            },
            flags,
            parent_table_flags,
            allocator,
        )
    }

    #[inline]
    fn map_range_with_table_flags<A>(
        &mut self,
        pages: PageRange<Size1GiB>,
        flags: PageTableFlags,
        parent_table_flags: PageTableFlags,
        allocator: &mut A,
    ) -> Result<MapperFlushRange<Size1GiB>, (MapToError<Size1GiB>, MapperFlushRange<Size1GiB>)>
    where
        Self: Sized,
        A: FrameAllocator<Size4KiB> + FrameAllocator<Size1GiB> + ?Sized,
    {
        self.map_to_range_1gib(
            pages,
            |_, allocator| allocator.allocate_frame(),
            flags,
            parent_table_flags,
            allocator,
        )
    }

    fn unmap(
        &mut self,
        page: Page<Size1GiB>,
    ) -> Result<(PhysFrame<Size1GiB>, MapperFlush<Size1GiB>), UnmapError> {
        let p4 = &mut self.level_4_table;
        let p3 = self
            .page_table_walker
            .next_table_mut(&mut p4[page.p4_index()])?;

        let p3_entry = &mut p3[page.p3_index()];
        let flags = p3_entry.flags();

        if !flags.contains(PageTableFlags::PRESENT) {
            return Err(UnmapError::PageNotMapped);
        }
        if !flags.contains(PageTableFlags::HUGE_PAGE) {
            return Err(UnmapError::ParentEntryHugePage);
        }

        let frame = PhysFrame::from_start_address(p3_entry.addr())
            .map_err(|AddressNotAligned| UnmapError::InvalidFrameAddress(p3_entry.addr()))?;

        p3_entry.set_unused();
        Ok((frame, MapperFlush::new(page)))
    }

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
        self.modify_range_1gib(
            pages,
            |entry, _, deallocator| {
                let frame = PhysFrame::from_start_address(entry.addr())
                    .map_err(|AddressNotAligned| UnmapError::InvalidFrameAddress(entry.addr()))?;
                unsafe {
                    deallocator.deallocate_frame(frame);
                }

                entry.set_unused();
                Ok(())
            },
            deallocator,
            Self::next_table_fn_next_table_mut,
        )
    }

    unsafe fn update_flags(
        &mut self,
        page: Page<Size1GiB>,
        flags: PageTableFlags,
    ) -> Result<MapperFlush<Size1GiB>, FlagUpdateError> {
        let p4 = &mut self.level_4_table;
        let p3 = self
            .page_table_walker
            .next_table_mut(&mut p4[page.p4_index()])?;

        if p3[page.p3_index()].is_unused() {
            return Err(FlagUpdateError::PageNotMapped);
        }
        p3[page.p3_index()].set_flags(flags | PageTableFlags::HUGE_PAGE);

        Ok(MapperFlush::new(page))
    }

    #[inline]
    unsafe fn update_flags_range(
        &mut self,
        pages: PageRange<Size1GiB>,
        flags: PageTableFlags,
    ) -> Result<MapperFlushRange<Size1GiB>, (FlagUpdateError, MapperFlushRange<Size1GiB>)> {
        self.modify_range_1gib(
            pages,
            |entry, _, _| {
                if entry.is_unused() {
                    return Err(FlagUpdateError::PageNotMapped);
                }

                entry.set_flags(flags);
                Ok(())
            },
            (),
            Self::next_table_fn_next_table_mut,
        )
    }

    unsafe fn set_flags_p4_entry(
        &mut self,
        page: Page<Size1GiB>,
        flags: PageTableFlags,
    ) -> Result<MapperFlushAll, FlagUpdateError> {
        let p4 = &mut self.level_4_table;
        let p4_entry = &mut p4[page.p4_index()];

        if p4_entry.is_unused() {
            return Err(FlagUpdateError::PageNotMapped);
        }

        p4_entry.set_flags(flags);

        Ok(MapperFlushAll::new())
    }

    unsafe fn set_flags_p3_entry(
        &mut self,
        _page: Page<Size1GiB>,
        _flags: PageTableFlags,
    ) -> Result<MapperFlushAll, FlagUpdateError> {
        Err(FlagUpdateError::ParentEntryHugePage)
    }

    unsafe fn set_flags_p2_entry(
        &mut self,
        _page: Page<Size1GiB>,
        _flags: PageTableFlags,
    ) -> Result<MapperFlushAll, FlagUpdateError> {
        Err(FlagUpdateError::ParentEntryHugePage)
    }

    fn translate_page(&self, page: Page<Size1GiB>) -> Result<PhysFrame<Size1GiB>, TranslateError> {
        let p4 = &self.level_4_table;
        let p3 = self.page_table_walker.next_table(&p4[page.p4_index()])?;

        let p3_entry = &p3[page.p3_index()];

        if p3_entry.is_unused() {
            return Err(TranslateError::PageNotMapped);
        }

        PhysFrame::from_start_address(p3_entry.addr())
            .map_err(|AddressNotAligned| TranslateError::InvalidFrameAddress(p3_entry.addr()))
    }
}

impl<'a, P: PageTableFrameMapping> Mapper<Size2MiB> for MappedPageTable<'a, P> {
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
        self.map_to_2mib(page, frame, flags, parent_table_flags, allocator)
    }

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
        assert_eq!(pages.count(), frames.count());
        self.map_to_range_2mib(
            pages,
            |page, _| {
                let offset = page - pages.start;
                Some(frames.start + offset)
            },
            flags,
            parent_table_flags,
            allocator,
        )
    }

    #[inline]
    fn map_range_with_table_flags<A>(
        &mut self,
        pages: PageRange<Size2MiB>,
        flags: PageTableFlags,
        parent_table_flags: PageTableFlags,
        allocator: &mut A,
    ) -> Result<MapperFlushRange<Size2MiB>, (MapToError<Size2MiB>, MapperFlushRange<Size2MiB>)>
    where
        Self: Sized,
        A: FrameAllocator<Size4KiB> + FrameAllocator<Size2MiB> + ?Sized,
    {
        self.map_to_range_2mib(
            pages,
            |_, allocator| allocator.allocate_frame(),
            flags,
            parent_table_flags,
            allocator,
        )
    }

    fn unmap(
        &mut self,
        page: Page<Size2MiB>,
    ) -> Result<(PhysFrame<Size2MiB>, MapperFlush<Size2MiB>), UnmapError> {
        let p4 = &mut self.level_4_table;
        let p3 = self
            .page_table_walker
            .next_table_mut(&mut p4[page.p4_index()])?;
        let p2 = self
            .page_table_walker
            .next_table_mut(&mut p3[page.p3_index()])?;

        let p2_entry = &mut p2[page.p2_index()];
        let flags = p2_entry.flags();

        if !flags.contains(PageTableFlags::PRESENT) {
            return Err(UnmapError::PageNotMapped);
        }
        if !flags.contains(PageTableFlags::HUGE_PAGE) {
            return Err(UnmapError::ParentEntryHugePage);
        }

        let frame = PhysFrame::from_start_address(p2_entry.addr())
            .map_err(|AddressNotAligned| UnmapError::InvalidFrameAddress(p2_entry.addr()))?;

        p2_entry.set_unused();
        Ok((frame, MapperFlush::new(page)))
    }

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
        self.modify_range_2mib(
            pages,
            |entry, _, deallocator| {
                let frame = PhysFrame::from_start_address(entry.addr())
                    .map_err(|AddressNotAligned| UnmapError::InvalidFrameAddress(entry.addr()))?;
                unsafe {
                    deallocator.deallocate_frame(frame);
                }

                entry.set_unused();
                Ok(())
            },
            deallocator,
            Self::next_table_fn_next_table_mut,
        )
    }

    unsafe fn update_flags(
        &mut self,
        page: Page<Size2MiB>,
        flags: PageTableFlags,
    ) -> Result<MapperFlush<Size2MiB>, FlagUpdateError> {
        let p4 = &mut self.level_4_table;
        let p3 = self
            .page_table_walker
            .next_table_mut(&mut p4[page.p4_index()])?;
        let p2 = self
            .page_table_walker
            .next_table_mut(&mut p3[page.p3_index()])?;

        if p2[page.p2_index()].is_unused() {
            return Err(FlagUpdateError::PageNotMapped);
        }

        p2[page.p2_index()].set_flags(flags | PageTableFlags::HUGE_PAGE);

        Ok(MapperFlush::new(page))
    }

    #[inline]
    unsafe fn update_flags_range(
        &mut self,
        pages: PageRange<Size2MiB>,
        flags: PageTableFlags,
    ) -> Result<MapperFlushRange<Size2MiB>, (FlagUpdateError, MapperFlushRange<Size2MiB>)> {
        self.modify_range_2mib(
            pages,
            |entry, _, _| {
                if entry.is_unused() {
                    return Err(FlagUpdateError::PageNotMapped);
                }

                entry.set_flags(flags);
                Ok(())
            },
            (),
            Self::next_table_fn_next_table_mut,
        )
    }

    unsafe fn set_flags_p4_entry(
        &mut self,
        page: Page<Size2MiB>,
        flags: PageTableFlags,
    ) -> Result<MapperFlushAll, FlagUpdateError> {
        let p4 = &mut self.level_4_table;
        let p4_entry = &mut p4[page.p4_index()];

        if p4_entry.is_unused() {
            return Err(FlagUpdateError::PageNotMapped);
        }

        p4_entry.set_flags(flags);

        Ok(MapperFlushAll::new())
    }

    unsafe fn set_flags_p3_entry(
        &mut self,
        page: Page<Size2MiB>,
        flags: PageTableFlags,
    ) -> Result<MapperFlushAll, FlagUpdateError> {
        let p4 = &mut self.level_4_table;
        let p3 = self
            .page_table_walker
            .next_table_mut(&mut p4[page.p4_index()])?;
        let p3_entry = &mut p3[page.p3_index()];

        if p3_entry.is_unused() {
            return Err(FlagUpdateError::PageNotMapped);
        }

        p3_entry.set_flags(flags);

        Ok(MapperFlushAll::new())
    }

    unsafe fn set_flags_p2_entry(
        &mut self,
        _page: Page<Size2MiB>,
        _flags: PageTableFlags,
    ) -> Result<MapperFlushAll, FlagUpdateError> {
        Err(FlagUpdateError::ParentEntryHugePage)
    }

    fn translate_page(&self, page: Page<Size2MiB>) -> Result<PhysFrame<Size2MiB>, TranslateError> {
        let p4 = &self.level_4_table;
        let p3 = self.page_table_walker.next_table(&p4[page.p4_index()])?;
        let p2 = self.page_table_walker.next_table(&p3[page.p3_index()])?;

        let p2_entry = &p2[page.p2_index()];

        if p2_entry.is_unused() {
            return Err(TranslateError::PageNotMapped);
        }

        PhysFrame::from_start_address(p2_entry.addr())
            .map_err(|AddressNotAligned| TranslateError::InvalidFrameAddress(p2_entry.addr()))
    }
}

impl<'a, P: PageTableFrameMapping> Mapper<Size4KiB> for MappedPageTable<'a, P> {
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
        self.map_to_4kib(page, frame, flags, parent_table_flags, allocator)
    }

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
        assert_eq!(pages.count(), frames.count());
        self.map_to_range_4kib(
            pages,
            |page, _| {
                let offset = page - pages.start;
                Some(frames.start + offset)
            },
            flags,
            parent_table_flags,
            allocator,
        )
    }

    #[inline]
    fn map_range_with_table_flags<A>(
        &mut self,
        pages: PageRange<Size4KiB>,
        flags: PageTableFlags,
        parent_table_flags: PageTableFlags,
        allocator: &mut A,
    ) -> Result<MapperFlushRange<Size4KiB>, (MapToError<Size4KiB>, MapperFlushRange<Size4KiB>)>
    where
        A: FrameAllocator<Size4KiB> + ?Sized,
    {
        self.map_to_range_4kib(
            pages,
            |_, allocator| allocator.allocate_frame(),
            flags,
            parent_table_flags,
            allocator,
        )
    }

    fn unmap(
        &mut self,
        page: Page<Size4KiB>,
    ) -> Result<(PhysFrame<Size4KiB>, MapperFlush<Size4KiB>), UnmapError> {
        let p4 = &mut self.level_4_table;
        let p3 = self
            .page_table_walker
            .next_table_mut(&mut p4[page.p4_index()])?;
        let p2 = self
            .page_table_walker
            .next_table_mut(&mut p3[page.p3_index()])?;
        let p1 = self
            .page_table_walker
            .next_table_mut(&mut p2[page.p2_index()])?;

        let p1_entry = &mut p1[page.p1_index()];

        let frame = p1_entry.frame().map_err(|err| match err {
            FrameError::FrameNotPresent => UnmapError::PageNotMapped,
            FrameError::HugeFrame => UnmapError::ParentEntryHugePage,
        })?;

        p1_entry.set_unused();
        Ok((frame, MapperFlush::new(page)))
    }

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
        self.modify_range_4kib(
            pages,
            |entry, _, deallocator| {
                let frame = entry.frame().map_err(|err| match err {
                    FrameError::FrameNotPresent => UnmapError::PageNotMapped,
                    FrameError::HugeFrame => UnmapError::ParentEntryHugePage,
                })?;
                unsafe {
                    deallocator.deallocate_frame(frame);
                }

                entry.set_unused();
                Ok(())
            },
            deallocator,
            Self::next_table_fn_next_table_mut,
        )
    }

    unsafe fn update_flags(
        &mut self,
        page: Page<Size4KiB>,
        flags: PageTableFlags,
    ) -> Result<MapperFlush<Size4KiB>, FlagUpdateError> {
        let p4 = &mut self.level_4_table;
        let p3 = self
            .page_table_walker
            .next_table_mut(&mut p4[page.p4_index()])?;
        let p2 = self
            .page_table_walker
            .next_table_mut(&mut p3[page.p3_index()])?;
        let p1 = self
            .page_table_walker
            .next_table_mut(&mut p2[page.p2_index()])?;

        if p1[page.p1_index()].is_unused() {
            return Err(FlagUpdateError::PageNotMapped);
        }

        p1[page.p1_index()].set_flags(flags);

        Ok(MapperFlush::new(page))
    }

    #[inline]
    unsafe fn update_flags_range(
        &mut self,
        pages: PageRange<Size4KiB>,
        flags: PageTableFlags,
    ) -> Result<MapperFlushRange<Size4KiB>, (FlagUpdateError, MapperFlushRange<Size4KiB>)> {
        self.modify_range_4kib(
            pages,
            |entry, _, _| {
                if entry.is_unused() {
                    return Err(FlagUpdateError::PageNotMapped);
                }

                entry.set_flags(flags);
                Ok(())
            },
            (),
            Self::next_table_fn_next_table_mut,
        )
    }

    unsafe fn set_flags_p4_entry(
        &mut self,
        page: Page<Size4KiB>,
        flags: PageTableFlags,
    ) -> Result<MapperFlushAll, FlagUpdateError> {
        let p4 = &mut self.level_4_table;
        let p4_entry = &mut p4[page.p4_index()];

        if p4_entry.is_unused() {
            return Err(FlagUpdateError::PageNotMapped);
        }

        p4_entry.set_flags(flags);

        Ok(MapperFlushAll::new())
    }

    unsafe fn set_flags_p3_entry(
        &mut self,
        page: Page<Size4KiB>,
        flags: PageTableFlags,
    ) -> Result<MapperFlushAll, FlagUpdateError> {
        let p4 = &mut self.level_4_table;
        let p3 = self
            .page_table_walker
            .next_table_mut(&mut p4[page.p4_index()])?;
        let p3_entry = &mut p3[page.p3_index()];

        if p3_entry.is_unused() {
            return Err(FlagUpdateError::PageNotMapped);
        }

        p3_entry.set_flags(flags);

        Ok(MapperFlushAll::new())
    }

    unsafe fn set_flags_p2_entry(
        &mut self,
        page: Page<Size4KiB>,
        flags: PageTableFlags,
    ) -> Result<MapperFlushAll, FlagUpdateError> {
        let p4 = &mut self.level_4_table;
        let p3 = self
            .page_table_walker
            .next_table_mut(&mut p4[page.p4_index()])?;
        let p2 = self
            .page_table_walker
            .next_table_mut(&mut p3[page.p3_index()])?;
        let p2_entry = &mut p2[page.p2_index()];

        if p2_entry.is_unused() {
            return Err(FlagUpdateError::PageNotMapped);
        }

        p2_entry.set_flags(flags);

        Ok(MapperFlushAll::new())
    }

    fn translate_page(&self, page: Page<Size4KiB>) -> Result<PhysFrame<Size4KiB>, TranslateError> {
        let p4 = &self.level_4_table;
        let p3 = self.page_table_walker.next_table(&p4[page.p4_index()])?;
        let p2 = self.page_table_walker.next_table(&p3[page.p3_index()])?;
        let p1 = self.page_table_walker.next_table(&p2[page.p2_index()])?;

        let p1_entry = &p1[page.p1_index()];

        if p1_entry.is_unused() {
            return Err(TranslateError::PageNotMapped);
        }

        PhysFrame::from_start_address(p1_entry.addr())
            .map_err(|AddressNotAligned| TranslateError::InvalidFrameAddress(p1_entry.addr()))
    }
}

impl<'a, P: PageTableFrameMapping> Translate for MappedPageTable<'a, P> {
    #[allow(clippy::inconsistent_digit_grouping)]
    fn translate(&self, addr: VirtAddr) -> TranslateResult {
        let p4 = &self.level_4_table;
        let p3 = match self.page_table_walker.next_table(&p4[addr.p4_index()]) {
            Ok(page_table) => page_table,
            Err(PageTableWalkError::NotMapped) => return TranslateResult::NotMapped,
            Err(PageTableWalkError::MappedToHugePage) => {
                panic!("level 4 entry has huge page bit set")
            }
        };
        let p2 = match self.page_table_walker.next_table(&p3[addr.p3_index()]) {
            Ok(page_table) => page_table,
            Err(PageTableWalkError::NotMapped) => return TranslateResult::NotMapped,
            Err(PageTableWalkError::MappedToHugePage) => {
                let entry = &p3[addr.p3_index()];
                let frame = PhysFrame::containing_address(entry.addr());
                let offset = addr.as_u64() & 0o_777_777_7777;
                let flags = entry.flags();
                return TranslateResult::Mapped {
                    frame: MappedFrame::Size1GiB(frame),
                    offset,
                    flags,
                };
            }
        };
        let p1 = match self.page_table_walker.next_table(&p2[addr.p2_index()]) {
            Ok(page_table) => page_table,
            Err(PageTableWalkError::NotMapped) => return TranslateResult::NotMapped,
            Err(PageTableWalkError::MappedToHugePage) => {
                let entry = &p2[addr.p2_index()];
                let frame = PhysFrame::containing_address(entry.addr());
                let offset = addr.as_u64() & 0o_777_7777;
                let flags = entry.flags();
                return TranslateResult::Mapped {
                    frame: MappedFrame::Size2MiB(frame),
                    offset,
                    flags,
                };
            }
        };

        let p1_entry = &p1[addr.p1_index()];

        if p1_entry.is_unused() {
            return TranslateResult::NotMapped;
        }

        let frame = match PhysFrame::from_start_address(p1_entry.addr()) {
            Ok(frame) => frame,
            Err(AddressNotAligned) => return TranslateResult::InvalidFrameAddress(p1_entry.addr()),
        };
        let offset = u64::from(addr.page_offset());
        let flags = p1_entry.flags();
        TranslateResult::Mapped {
            frame: MappedFrame::Size4KiB(frame),
            offset,
            flags,
        }
    }
}

impl<'a, P: PageTableFrameMapping> CleanUp for MappedPageTable<'a, P> {
    #[inline]
    unsafe fn clean_up<D>(&mut self, frame_deallocator: &mut D)
    where
        D: FrameDeallocator<Size4KiB>,
    {
        unsafe {
            self.clean_up_addr_range(
                PageRangeInclusive {
                    start: Page::from_start_address(VirtAddr::new(0)).unwrap(),
                    end: Page::from_start_address(VirtAddr::new(0xffff_ffff_ffff_f000)).unwrap(),
                },
                frame_deallocator,
            )
        }
    }

    unsafe fn clean_up_addr_range<D>(
        &mut self,
        range: PageRangeInclusive,
        frame_deallocator: &mut D,
    ) where
        D: FrameDeallocator<Size4KiB>,
    {
        unsafe fn clean_up<P: PageTableFrameMapping>(
            page_table: &mut PageTable,
            page_table_walker: &PageTableWalker<P>,
            level: PageTableLevel,
            range: PageRangeInclusive,
            frame_deallocator: &mut impl FrameDeallocator<Size4KiB>,
        ) -> bool {
            if range.is_empty() {
                return false;
            }

            let table_addr = range
                .start
                .start_address()
                .align_down(level.table_address_space_alignment());

            let start = range.start.page_table_index(level);
            let end = range.end.page_table_index(level);

            if let Some(next_level) = level.next_lower_level() {
                let offset_per_entry = level.entry_address_space_alignment();
                for (i, entry) in page_table
                    .iter_mut()
                    .enumerate()
                    .take(usize::from(end) + 1)
                    .skip(usize::from(start))
                {
                    if let Ok(page_table) = page_table_walker.next_table_mut(entry) {
                        let start = table_addr + (offset_per_entry * (i as u64));
                        let end = start + (offset_per_entry - 1);
                        let start = Page::<Size4KiB>::containing_address(start);
                        let start = start.max(range.start);
                        let end = Page::<Size4KiB>::containing_address(end);
                        let end = end.min(range.end);
                        unsafe {
                            if clean_up(
                                page_table,
                                page_table_walker,
                                next_level,
                                Page::range_inclusive(start, end),
                                frame_deallocator,
                            ) {
                                let frame = entry.frame().unwrap();
                                entry.set_unused();
                                frame_deallocator.deallocate_frame(frame);
                            }
                        }
                    }
                }
            }

            page_table.iter().all(PageTableEntry::is_unused)
        }

        unsafe {
            clean_up(
                self.level_4_table,
                &self.page_table_walker,
                PageTableLevel::Four,
                range,
                frame_deallocator,
            );
        }
    }
}

#[derive(Debug)]
struct PageTableWalker<P: PageTableFrameMapping> {
    page_table_frame_mapping: P,
}

impl<P: PageTableFrameMapping> PageTableWalker<P> {
    #[inline]
    pub unsafe fn new(page_table_frame_mapping: P) -> Self {
        Self {
            page_table_frame_mapping,
        }
    }

    /// Internal helper function to get a reference to the page table of the next level.
    ///
    /// Returns `PageTableWalkError::NotMapped` if the entry is unused. Returns
    /// `PageTableWalkError::MappedToHugePage` if the `HUGE_PAGE` flag is set
    /// in the passed entry.
    #[inline]
    fn next_table<'b>(
        &self,
        entry: &'b PageTableEntry,
    ) -> Result<&'b PageTable, PageTableWalkError> {
        let page_table_ptr = self
            .page_table_frame_mapping
            .frame_to_pointer(entry.frame()?);
        let page_table: &PageTable = unsafe { &*page_table_ptr };

        Ok(page_table)
    }

    /// Internal helper function to get a mutable reference to the page table of the next level.
    ///
    /// Returns `PageTableWalkError::NotMapped` if the entry is unused. Returns
    /// `PageTableWalkError::MappedToHugePage` if the `HUGE_PAGE` flag is set
    /// in the passed entry.
    #[inline]
    fn next_table_mut<'b>(
        &self,
        entry: &'b mut PageTableEntry,
    ) -> Result<&'b mut PageTable, PageTableWalkError> {
        let page_table_ptr = self
            .page_table_frame_mapping
            .frame_to_pointer(entry.frame()?);
        let page_table: &mut PageTable = unsafe { &mut *page_table_ptr };

        Ok(page_table)
    }

    /// Internal helper function to create the page table of the next level if needed.
    ///
    /// If the passed entry is unused, a new frame is allocated from the given allocator, zeroed,
    /// and the entry is updated to that address. If the passed entry is already mapped, the next
    /// table is returned directly.
    ///
    /// Returns `MapToError::FrameAllocationFailed` if the entry is unused and the allocator
    /// returned `None`. Returns `MapToError::ParentEntryHugePage` if the `HUGE_PAGE` flag is set
    /// in the passed entry.
    fn create_next_table<'b, A>(
        &self,
        entry: &'b mut PageTableEntry,
        insert_flags: PageTableFlags,
        allocator: &mut A,
    ) -> Result<&'b mut PageTable, PageTableCreateError>
    where
        A: FrameAllocator<Size4KiB> + ?Sized,
    {
        let created;

        if entry.is_unused() {
            if let Some(frame) = allocator.allocate_frame() {
                entry.set_frame(frame, insert_flags);
                created = true;
            } else {
                return Err(PageTableCreateError::FrameAllocationFailed);
            }
        } else {
            if !insert_flags.is_empty() && !entry.flags().contains(insert_flags) {
                entry.set_flags(entry.flags() | insert_flags);
            }
            created = false;
        }

        let page_table = match self.next_table_mut(entry) {
            Err(PageTableWalkError::MappedToHugePage) => {
                return Err(PageTableCreateError::MappedToHugePage);
            }
            Err(PageTableWalkError::NotMapped) => panic!("entry should be mapped at this point"),
            Ok(page_table) => page_table,
        };

        if created {
            page_table.zero();
        }
        Ok(page_table)
    }
}

#[derive(Debug)]
enum PageTableWalkError {
    NotMapped,
    MappedToHugePage,
}

#[derive(Debug)]
enum PageTableCreateError {
    MappedToHugePage,
    FrameAllocationFailed,
}

impl From<PageTableCreateError> for MapToError<Size4KiB> {
    #[inline]
    fn from(err: PageTableCreateError) -> Self {
        match err {
            PageTableCreateError::MappedToHugePage => MapToError::ParentEntryHugePage,
            PageTableCreateError::FrameAllocationFailed => MapToError::FrameAllocationFailed,
        }
    }
}

impl From<PageTableCreateError> for MapToError<Size2MiB> {
    #[inline]
    fn from(err: PageTableCreateError) -> Self {
        match err {
            PageTableCreateError::MappedToHugePage => MapToError::ParentEntryHugePage,
            PageTableCreateError::FrameAllocationFailed => MapToError::FrameAllocationFailed,
        }
    }
}

impl From<PageTableCreateError> for MapToError<Size1GiB> {
    #[inline]
    fn from(err: PageTableCreateError) -> Self {
        match err {
            PageTableCreateError::MappedToHugePage => MapToError::ParentEntryHugePage,
            PageTableCreateError::FrameAllocationFailed => MapToError::FrameAllocationFailed,
        }
    }
}

impl From<FrameError> for PageTableWalkError {
    #[inline]
    fn from(err: FrameError) -> Self {
        match err {
            FrameError::HugeFrame => PageTableWalkError::MappedToHugePage,
            FrameError::FrameNotPresent => PageTableWalkError::NotMapped,
        }
    }
}

impl From<PageTableWalkError> for UnmapError {
    #[inline]
    fn from(err: PageTableWalkError) -> Self {
        match err {
            PageTableWalkError::MappedToHugePage => UnmapError::ParentEntryHugePage,
            PageTableWalkError::NotMapped => UnmapError::PageNotMapped,
        }
    }
}

impl From<PageTableWalkError> for FlagUpdateError {
    #[inline]
    fn from(err: PageTableWalkError) -> Self {
        match err {
            PageTableWalkError::MappedToHugePage => FlagUpdateError::ParentEntryHugePage,
            PageTableWalkError::NotMapped => FlagUpdateError::PageNotMapped,
        }
    }
}

impl From<PageTableWalkError> for TranslateError {
    #[inline]
    fn from(err: PageTableWalkError) -> Self {
        match err {
            PageTableWalkError::MappedToHugePage => TranslateError::ParentEntryHugePage,
            PageTableWalkError::NotMapped => TranslateError::PageNotMapped,
        }
    }
}

/// Provides a virtual address mapping for physical page table frames.
///
/// This only works if the physical address space is somehow mapped to the virtual
/// address space, e.g. at an offset.
///
/// ## Safety
///
/// This trait is unsafe to implement because the implementer must ensure that
/// `frame_to_pointer` returns a valid page table pointer for any given physical frame.
pub unsafe trait PageTableFrameMapping {
    /// Translate the given physical frame to a virtual page table pointer.
    fn frame_to_pointer(&self, frame: PhysFrame) -> *mut PageTable;
}
