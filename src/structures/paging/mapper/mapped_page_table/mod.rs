mod display;
mod iter;
mod offset_page_table;
mod range_iter;

pub use self::display::Display;
#[cfg(target_pointer_width = "64")]
pub use self::offset_page_table::{OffsetPageTable, PhysOffset};
use crate::addr::{FixedValidity, VirtAddrGeneric, VirtAddrValidity};
use crate::structures::paging::{
    mapper::*,
    page::AddressNotAligned,
    page_table::{FrameError, PageTable, PageTableEntry, PageTableLevel, PageTableRootLevel},
};

/// A mapper for a fixed four-level or five-level page-table hierarchy.
///
/// BITS is the canonical address width of the tree: 48 (the default) or 57.
/// The latter requires the feature virt_addr_57. It describes the tree being edited,
/// not the currently active CPU mode. Construction, iteration, and operations on supplied
/// pages/addresses do not read CR3/CR4 or consult the runtime address-width cache.
/// Convenience methods which create new runtime addresses (such as identity_map with
/// RuntimeValidity) still follow that address policy's construction requirements.
///
/// Four-level mappers implement Mapper and Translate only for FixedValidity<48>.
/// Five-level mappers also support FixedValidity<57> and, with virt_addr_rt, RuntimeValidity.
/// These are exactly the policies with an infallible conversion to the root's fixed
/// address type. Runtime addresses are consumed as values, not revalidated against CR4.
/// Iterator and display addresses always use the root's fixed width.
///
/// Every physical page-table frame must be accessible through the supplied
/// PageTableFrameMapping. For example, a physical-memory mapping at a fixed offset
/// can be represented by OffsetPageTable.
///
/// The type parameter selects the root; the old runtime-level constructors are unnecessary:
///
/// ```
/// use x86_64::structures::paging::{MappedPageTable, PageTable, PageTableRootLevel};
/// use x86_64::structures::paging::mapper::PageTableFrameMapping;
/// unsafe fn example<P: PageTableFrameMapping>(root: &mut PageTable, mapping: P) {
///     let l4 = unsafe { MappedPageTable::<_, 48>::new(root, mapping) };
///     assert_eq!(l4.root_level(), PageTableRootLevel::Four);
/// }
/// ```
///
/// Other address widths cannot be instantiated:
///
/// ```compile_fail
/// use x86_64::structures::paging::MappedPageTable;
/// use x86_64::structures::paging::mapper::PageTableFrameMapping;
/// fn invalid<P: PageTableFrameMapping>(_: Option<MappedPageTable<'_, P, 49>>) {}
/// ```
#[derive(Debug)]
pub struct MappedPageTable<'a, P: PageTableFrameMapping, const BITS: usize = 48>
where
    FixedValidity<BITS>: VirtAddrValidity,
{
    page_table_walker: PageTableWalker<P>,
    root_table: &'a mut PageTable,
}

impl<'a, P: PageTableFrameMapping, const BITS: usize> MappedPageTable<'a, P, BITS>
where
    FixedValidity<BITS>: VirtAddrValidity,
{
    #[inline]
    fn p4_table<S: PageSize, V: VirtAddrValidity>(
        &self,
        page: Page<S, V>,
    ) -> Result<*const PageTable, PageTableWalkError> {
        if BITS == 48 {
            Ok(self.root_table as *const _)
        } else {
            Ok(self
                .page_table_walker
                .next_table(&self.root_table[page.p5_index()])? as *const _)
        }
    }

    #[inline]
    fn p4_table_mut<S: PageSize, V: VirtAddrValidity>(
        &mut self,
        page: Page<S, V>,
    ) -> Result<*mut PageTable, PageTableWalkError> {
        if BITS == 48 {
            Ok(self.root_table as *mut _)
        } else {
            Ok(self
                .page_table_walker
                .next_table_mut(&mut self.root_table[page.p5_index()])? as *mut _)
        }
    }

    #[inline]
    unsafe fn create_p4_table<A, S: PageSize, V: VirtAddrValidity>(
        &mut self,
        page: Page<S, V>,
        flags: PageTableFlags,
        allocator: &mut A,
    ) -> Result<*mut PageTable, PageTableCreateError>
    where
        A: FrameAllocator<Size4KiB> + ?Sized,
    {
        if BITS == 48 {
            Ok(self.root_table as *mut _)
        } else {
            Ok(self.page_table_walker.create_next_table(
                &mut self.root_table[page.p5_index()],
                flags,
                allocator,
            )? as *mut _)
        }
    }

    /// Creates a mapper for a root whose address width is BITS.
    ///
    /// Use BITS = 48 for a four-level root and BITS = 57 for a five-level root.
    /// This does not inspect CR3, CR4, or the runtime address-width cache.
    ///
    /// # Safety
    ///
    /// The supplied table must root a valid hierarchy of the selected level. The frame
    /// mapping must make all child tables accessible for the duration of the mapper's use.
    /// The tree need not be active, and its level need not match the current CPU mode.
    #[inline]
    pub unsafe fn new(root_table: &'a mut PageTable, page_table_frame_mapping: P) -> Self {
        Self {
            root_table,
            page_table_walker: unsafe { PageTableWalker::new(page_table_frame_mapping) },
        }
    }

    /// Returns the root level determined by BITS.
    pub const fn root_level(&self) -> PageTableRootLevel {
        if BITS == 48 {
            PageTableRootLevel::Four
        } else {
            // The sealed validity bound permits only 48 and 57.
            PageTableRootLevel::Five
        }
    }

    /// Returns the wrapped root page table.
    #[inline]
    pub fn root_table(&self) -> &PageTable {
        self.root_table
    }

    /// Returns a mutable reference to the wrapped root page table.
    #[inline]
    pub fn root_table_mut(&mut self) -> &mut PageTable {
        self.root_table
    }

    /// Returns the `PageTableFrameMapping` used for converting virtual to physical addresses.
    pub fn page_table_frame_mapping(&self) -> &P {
        &self.page_table_walker.page_table_frame_mapping
    }
}

impl<P: PageTableFrameMapping> MappedPageTable<'_, P, 48> {
    /// Returns the level-4 root table.
    pub fn level_4_table(&self) -> &PageTable {
        self.root_table()
    }

    /// Returns the level-4 root table mutably.
    pub fn level_4_table_mut(&mut self) -> &mut PageTable {
        self.root_table_mut()
    }
}

impl<P: PageTableFrameMapping, V: VirtAddrValidity, const BITS: usize> Mapper<Size1GiB, V>
    for MappedPageTable<'_, P, BITS>
where
    FixedValidity<BITS>: VirtAddrValidity,
    VirtAddrGeneric<FixedValidity<BITS>>: From<VirtAddrGeneric<V>>,
{
    #[inline]
    unsafe fn map_to_with_table_flags<A>(
        &mut self,
        page: Page<Size1GiB, V>,
        frame: PhysFrame<Size1GiB>,
        flags: PageTableFlags,
        parent_table_flags: PageTableFlags,
        allocator: &mut A,
    ) -> Result<MapperFlush<Size1GiB, V>, MapToError<Size1GiB>>
    where
        A: FrameAllocator<Size4KiB> + ?Sized,
    {
        let p4 = unsafe { self.create_p4_table(page, parent_table_flags, allocator) }
            .map_err(MapToError::from)?;
        let p4 = unsafe { &mut *p4 };
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

    fn unmap(
        &mut self,
        page: Page<Size1GiB, V>,
    ) -> Result<
        (
            PhysFrame<Size1GiB>,
            PageTableFlags,
            MapperFlush<Size1GiB, V>,
        ),
        UnmapError,
    > {
        let p4 = self.p4_table_mut(page).map_err(UnmapError::from)?;
        let p4 = unsafe { &mut *p4 };
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
        Ok((frame, flags, MapperFlush::new(page)))
    }

    fn clear(&mut self, page: Page<Size1GiB, V>) -> Result<UnmappedFrame<Size1GiB, V>, UnmapError> {
        let p4 = self.p4_table_mut(page).map_err(UnmapError::from)?;
        let p4 = unsafe { &mut *p4 };
        let p3 = self
            .page_table_walker
            .next_table_mut(&mut p4[page.p4_index()])?;

        let p3_entry = &mut p3[page.p3_index()];
        let flags = p3_entry.flags();

        if !flags.contains(PageTableFlags::HUGE_PAGE) {
            return Err(UnmapError::ParentEntryHugePage);
        }

        if !flags.contains(PageTableFlags::PRESENT) {
            let cloned = p3_entry.clone();
            p3_entry.set_unused();
            return Ok(UnmappedFrame::NotPresent { entry: cloned });
        }

        let frame = PhysFrame::from_start_address(p3_entry.addr())
            .map_err(|AddressNotAligned| UnmapError::InvalidFrameAddress(p3_entry.addr()))?;
        let flags = p3_entry.flags();

        p3_entry.set_unused();

        Ok(UnmappedFrame::Present {
            frame,
            flags,
            flush: MapperFlush::new(page),
        })
    }

    unsafe fn update_flags(
        &mut self,
        page: Page<Size1GiB, V>,
        flags: PageTableFlags,
    ) -> Result<MapperFlush<Size1GiB, V>, FlagUpdateError> {
        let p4 = self.p4_table_mut(page).map_err(FlagUpdateError::from)?;
        let p4 = unsafe { &mut *p4 };
        let p3 = self
            .page_table_walker
            .next_table_mut(&mut p4[page.p4_index()])?;

        if p3[page.p3_index()].is_unused() {
            return Err(FlagUpdateError::PageNotMapped);
        }
        p3[page.p3_index()].set_flags(flags | PageTableFlags::HUGE_PAGE);

        Ok(MapperFlush::new(page))
    }

    unsafe fn set_flags_p5_entry(
        &mut self,
        page: Page<Size1GiB, V>,
        flags: PageTableFlags,
    ) -> Result<MapperFlushAll, FlagUpdateError> {
        if BITS != 57 {
            return Err(FlagUpdateError::PageTableLevelNotPresent);
        }
        let entry = &mut self.root_table[page.p5_index()];
        if entry.is_unused() {
            return Err(FlagUpdateError::PageNotMapped);
        }
        entry.set_flags(flags);
        Ok(MapperFlushAll::new())
    }

    unsafe fn set_flags_p4_entry(
        &mut self,
        page: Page<Size1GiB, V>,
        flags: PageTableFlags,
    ) -> Result<MapperFlushAll, FlagUpdateError> {
        let p4 = self.p4_table_mut(page).map_err(FlagUpdateError::from)?;
        let p4 = unsafe { &mut *p4 };
        let p4_entry = &mut p4[page.p4_index()];

        if p4_entry.is_unused() {
            return Err(FlagUpdateError::PageNotMapped);
        }

        p4_entry.set_flags(flags);

        Ok(MapperFlushAll::new())
    }

    unsafe fn set_flags_p3_entry(
        &mut self,
        _page: Page<Size1GiB, V>,
        _flags: PageTableFlags,
    ) -> Result<MapperFlushAll, FlagUpdateError> {
        Err(FlagUpdateError::ParentEntryHugePage)
    }

    unsafe fn set_flags_p2_entry(
        &mut self,
        _page: Page<Size1GiB, V>,
        _flags: PageTableFlags,
    ) -> Result<MapperFlushAll, FlagUpdateError> {
        Err(FlagUpdateError::ParentEntryHugePage)
    }

    fn translate_page(
        &self,
        page: Page<Size1GiB, V>,
    ) -> Result<PhysFrame<Size1GiB>, TranslateError> {
        let p4 = self.p4_table(page).map_err(TranslateError::from)?;
        let p4 = unsafe { &*p4 };
        let p3 = self.page_table_walker.next_table(&p4[page.p4_index()])?;

        let p3_entry = &p3[page.p3_index()];

        if !p3_entry
            .flags()
            .contains(PageTableFlags::PRESENT | PageTableFlags::HUGE_PAGE)
        {
            return Err(TranslateError::PageNotMapped);
        }

        PhysFrame::from_start_address(p3_entry.addr())
            .map_err(|AddressNotAligned| TranslateError::InvalidFrameAddress(p3_entry.addr()))
    }
}

impl<P: PageTableFrameMapping, V: VirtAddrValidity, const BITS: usize> Mapper<Size2MiB, V>
    for MappedPageTable<'_, P, BITS>
where
    FixedValidity<BITS>: VirtAddrValidity,
    VirtAddrGeneric<FixedValidity<BITS>>: From<VirtAddrGeneric<V>>,
{
    #[inline]
    unsafe fn map_to_with_table_flags<A>(
        &mut self,
        page: Page<Size2MiB, V>,
        frame: PhysFrame<Size2MiB>,
        flags: PageTableFlags,
        parent_table_flags: PageTableFlags,
        allocator: &mut A,
    ) -> Result<MapperFlush<Size2MiB, V>, MapToError<Size2MiB>>
    where
        A: FrameAllocator<Size4KiB> + ?Sized,
    {
        let p4 = unsafe { self.create_p4_table(page, parent_table_flags, allocator) }
            .map_err(MapToError::from)?;
        let p4 = unsafe { &mut *p4 };
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

    fn unmap(
        &mut self,
        page: Page<Size2MiB, V>,
    ) -> Result<
        (
            PhysFrame<Size2MiB>,
            PageTableFlags,
            MapperFlush<Size2MiB, V>,
        ),
        UnmapError,
    > {
        let p4 = self.p4_table_mut(page).map_err(UnmapError::from)?;
        let p4 = unsafe { &mut *p4 };
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
        Ok((frame, flags, MapperFlush::new(page)))
    }

    fn clear(&mut self, page: Page<Size2MiB, V>) -> Result<UnmappedFrame<Size2MiB, V>, UnmapError> {
        let p4 = self.p4_table_mut(page).map_err(UnmapError::from)?;
        let p4 = unsafe { &mut *p4 };
        let p3 = self
            .page_table_walker
            .next_table_mut(&mut p4[page.p4_index()])?;
        let p2 = self
            .page_table_walker
            .next_table_mut(&mut p3[page.p3_index()])?;

        let p2_entry = &mut p2[page.p2_index()];
        let flags = p2_entry.flags();

        if !flags.contains(PageTableFlags::HUGE_PAGE) {
            return Err(UnmapError::ParentEntryHugePage);
        }

        if !flags.contains(PageTableFlags::PRESENT) {
            let cloned = p2_entry.clone();
            p2_entry.set_unused();
            return Ok(UnmappedFrame::NotPresent { entry: cloned });
        }
        let frame = PhysFrame::from_start_address(p2_entry.addr())
            .map_err(|AddressNotAligned| UnmapError::InvalidFrameAddress(p2_entry.addr()))?;
        let flags = p2_entry.flags();

        p2_entry.set_unused();
        Ok(UnmappedFrame::Present {
            frame,
            flags,
            flush: MapperFlush::new(page),
        })
    }

    unsafe fn update_flags(
        &mut self,
        page: Page<Size2MiB, V>,
        flags: PageTableFlags,
    ) -> Result<MapperFlush<Size2MiB, V>, FlagUpdateError> {
        let p4 = self.p4_table_mut(page).map_err(FlagUpdateError::from)?;
        let p4 = unsafe { &mut *p4 };
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

    unsafe fn set_flags_p5_entry(
        &mut self,
        page: Page<Size2MiB, V>,
        flags: PageTableFlags,
    ) -> Result<MapperFlushAll, FlagUpdateError> {
        if BITS != 57 {
            return Err(FlagUpdateError::PageTableLevelNotPresent);
        }
        let entry = &mut self.root_table[page.p5_index()];
        if entry.is_unused() {
            return Err(FlagUpdateError::PageNotMapped);
        }
        entry.set_flags(flags);
        Ok(MapperFlushAll::new())
    }

    unsafe fn set_flags_p4_entry(
        &mut self,
        page: Page<Size2MiB, V>,
        flags: PageTableFlags,
    ) -> Result<MapperFlushAll, FlagUpdateError> {
        let p4 = self.p4_table_mut(page).map_err(FlagUpdateError::from)?;
        let p4 = unsafe { &mut *p4 };
        let p4_entry = &mut p4[page.p4_index()];

        if p4_entry.is_unused() {
            return Err(FlagUpdateError::PageNotMapped);
        }

        p4_entry.set_flags(flags);

        Ok(MapperFlushAll::new())
    }

    unsafe fn set_flags_p3_entry(
        &mut self,
        page: Page<Size2MiB, V>,
        flags: PageTableFlags,
    ) -> Result<MapperFlushAll, FlagUpdateError> {
        let p4 = self.p4_table_mut(page).map_err(FlagUpdateError::from)?;
        let p4 = unsafe { &mut *p4 };
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
        _page: Page<Size2MiB, V>,
        _flags: PageTableFlags,
    ) -> Result<MapperFlushAll, FlagUpdateError> {
        Err(FlagUpdateError::ParentEntryHugePage)
    }

    fn translate_page(
        &self,
        page: Page<Size2MiB, V>,
    ) -> Result<PhysFrame<Size2MiB>, TranslateError> {
        let p4 = self.p4_table(page).map_err(TranslateError::from)?;
        let p4 = unsafe { &*p4 };
        let p3 = self.page_table_walker.next_table(&p4[page.p4_index()])?;
        let p2 = self.page_table_walker.next_table(&p3[page.p3_index()])?;

        let p2_entry = &p2[page.p2_index()];

        if !p2_entry
            .flags()
            .contains(PageTableFlags::PRESENT | PageTableFlags::HUGE_PAGE)
        {
            return Err(TranslateError::PageNotMapped);
        }

        PhysFrame::from_start_address(p2_entry.addr())
            .map_err(|AddressNotAligned| TranslateError::InvalidFrameAddress(p2_entry.addr()))
    }
}

impl<P: PageTableFrameMapping, V: VirtAddrValidity, const BITS: usize> Mapper<Size4KiB, V>
    for MappedPageTable<'_, P, BITS>
where
    FixedValidity<BITS>: VirtAddrValidity,
    VirtAddrGeneric<FixedValidity<BITS>>: From<VirtAddrGeneric<V>>,
{
    #[inline]
    unsafe fn map_to_with_table_flags<A>(
        &mut self,
        page: Page<Size4KiB, V>,
        frame: PhysFrame<Size4KiB>,
        flags: PageTableFlags,
        parent_table_flags: PageTableFlags,
        allocator: &mut A,
    ) -> Result<MapperFlush<Size4KiB, V>, MapToError<Size4KiB>>
    where
        A: FrameAllocator<Size4KiB> + ?Sized,
    {
        let p4 = unsafe { self.create_p4_table(page, parent_table_flags, allocator) }
            .map_err(MapToError::from)?;
        let p4 = unsafe { &mut *p4 };
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

    fn unmap(
        &mut self,
        page: Page<Size4KiB, V>,
    ) -> Result<
        (
            PhysFrame<Size4KiB>,
            PageTableFlags,
            MapperFlush<Size4KiB, V>,
        ),
        UnmapError,
    > {
        let p4 = self.p4_table_mut(page).map_err(UnmapError::from)?;
        let p4 = unsafe { &mut *p4 };
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

        let frame = p1_entry.frame(true).map_err(|err| match err {
            FrameError::FrameNotPresent => UnmapError::PageNotMapped,
            FrameError::HugeFrame => unreachable!(),
        })?;
        let flags = p1_entry.flags();

        p1_entry.set_unused();
        Ok((frame, flags, MapperFlush::new(page)))
    }

    fn clear(&mut self, page: Page<Size4KiB, V>) -> Result<UnmappedFrame<Size4KiB, V>, UnmapError> {
        let p4 = self.p4_table_mut(page).map_err(UnmapError::from)?;
        let p4 = unsafe { &mut *p4 };
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

        let frame = match p1_entry.frame(true) {
            Ok(frame) => frame,
            Err(FrameError::HugeFrame) => unreachable!(),
            Err(FrameError::FrameNotPresent) => {
                let cloned = p1_entry.clone();
                p1_entry.set_unused();
                return Ok(UnmappedFrame::NotPresent { entry: cloned });
            }
        };
        let flags = p1_entry.flags();

        p1_entry.set_unused();
        Ok(UnmappedFrame::Present {
            frame,
            flags,
            flush: MapperFlush::new(page),
        })
    }

    unsafe fn update_flags(
        &mut self,
        page: Page<Size4KiB, V>,
        flags: PageTableFlags,
    ) -> Result<MapperFlush<Size4KiB, V>, FlagUpdateError> {
        let p4 = self.p4_table_mut(page).map_err(FlagUpdateError::from)?;
        let p4 = unsafe { &mut *p4 };
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

    unsafe fn set_flags_p5_entry(
        &mut self,
        page: Page<Size4KiB, V>,
        flags: PageTableFlags,
    ) -> Result<MapperFlushAll, FlagUpdateError> {
        if BITS != 57 {
            return Err(FlagUpdateError::PageTableLevelNotPresent);
        }
        let entry = &mut self.root_table[page.p5_index()];
        if entry.is_unused() {
            return Err(FlagUpdateError::PageNotMapped);
        }
        entry.set_flags(flags);
        Ok(MapperFlushAll::new())
    }

    unsafe fn set_flags_p4_entry(
        &mut self,
        page: Page<Size4KiB, V>,
        flags: PageTableFlags,
    ) -> Result<MapperFlushAll, FlagUpdateError> {
        let p4 = self.p4_table_mut(page).map_err(FlagUpdateError::from)?;
        let p4 = unsafe { &mut *p4 };
        let p4_entry = &mut p4[page.p4_index()];

        if p4_entry.is_unused() {
            return Err(FlagUpdateError::PageNotMapped);
        }

        p4_entry.set_flags(flags);

        Ok(MapperFlushAll::new())
    }

    unsafe fn set_flags_p3_entry(
        &mut self,
        page: Page<Size4KiB, V>,
        flags: PageTableFlags,
    ) -> Result<MapperFlushAll, FlagUpdateError> {
        let p4 = self.p4_table_mut(page).map_err(FlagUpdateError::from)?;
        let p4 = unsafe { &mut *p4 };
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
        page: Page<Size4KiB, V>,
        flags: PageTableFlags,
    ) -> Result<MapperFlushAll, FlagUpdateError> {
        let p4 = self.p4_table_mut(page).map_err(FlagUpdateError::from)?;
        let p4 = unsafe { &mut *p4 };
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

    fn translate_page(
        &self,
        page: Page<Size4KiB, V>,
    ) -> Result<PhysFrame<Size4KiB>, TranslateError> {
        let p4 = self.p4_table(page).map_err(TranslateError::from)?;
        let p4 = unsafe { &*p4 };
        let p3 = self.page_table_walker.next_table(&p4[page.p4_index()])?;
        let p2 = self.page_table_walker.next_table(&p3[page.p3_index()])?;
        let p1 = self.page_table_walker.next_table(&p2[page.p2_index()])?;

        let p1_entry = &p1[page.p1_index()];

        if !p1_entry.flags().contains(PageTableFlags::PRESENT) {
            return Err(TranslateError::PageNotMapped);
        }

        PhysFrame::from_start_address(p1_entry.addr())
            .map_err(|AddressNotAligned| TranslateError::InvalidFrameAddress(p1_entry.addr()))
    }
}

impl<P: PageTableFrameMapping, V: VirtAddrValidity, const BITS: usize> Translate<V>
    for MappedPageTable<'_, P, BITS>
where
    FixedValidity<BITS>: VirtAddrValidity,
    VirtAddrGeneric<FixedValidity<BITS>>: From<VirtAddrGeneric<V>>,
{
    #[allow(clippy::inconsistent_digit_grouping)]
    fn translate(&self, addr: VirtAddrGeneric<V>) -> TranslateResult {
        let p4_ptr = match self.p4_table(Page::<Size4KiB, V>::containing_address(addr)) {
            Ok(table) => table,
            Err(_) => return TranslateResult::NotMapped,
        };
        let p4 = unsafe { &*p4_ptr };
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
                #[allow(clippy::unusual_byte_groupings)]
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
                #[allow(clippy::unusual_byte_groupings)]
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

        if !p1_entry.flags().contains(PageTableFlags::PRESENT) {
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

impl<P: PageTableFrameMapping, const BITS: usize> CleanUp for MappedPageTable<'_, P, BITS>
where
    FixedValidity<BITS>: VirtAddrValidity,
{
    #[inline]
    unsafe fn clean_up<D>(&mut self, frame_deallocator: &mut D)
    where
        D: FrameDeallocator<Size4KiB>,
    {
        unsafe fn clean_all<P: PageTableFrameMapping>(
            table: &mut PageTable,
            walker: &PageTableWalker<P>,
            level: PageTableLevel,
            deallocator: &mut impl FrameDeallocator<Size4KiB>,
        ) -> bool {
            if let Some(next) = level.next_lower_level() {
                for entry in table.iter_mut() {
                    if entry.is_unused() || entry.flags().contains(PageTableFlags::HUGE_PAGE) {
                        continue;
                    }
                    let Ok(frame) = entry.frame(false) else {
                        continue;
                    };
                    let Ok(child) = walker.next_table_mut(entry) else {
                        continue;
                    };
                    if unsafe { clean_all(child, walker, next, deallocator) } {
                        entry.set_unused();
                        unsafe {
                            deallocator.deallocate_frame(frame);
                        }
                    }
                }
            }
            table.iter().all(PageTableEntry::is_unused)
        }
        unsafe {
            clean_all(
                self.root_table,
                &self.page_table_walker,
                self.root_level().as_page_table_level(),
                frame_deallocator,
            );
        }
    }

    unsafe fn clean_up_addr_range<D, V: VirtAddrValidity>(
        &mut self,
        range: PageRangeInclusive<Size4KiB, V>,
        frame_deallocator: &mut D,
    ) where
        D: FrameDeallocator<Size4KiB>,
        Self: Mapper<Size4KiB, V>,
    {
        if range.is_empty() {
            return;
        }

        // Walk using the root's fixed width, independently of the input range policy.
        unsafe fn clean_range<P: PageTableFrameMapping>(
            table: &mut PageTable,
            walker: &PageTableWalker<P>,
            level: PageTableLevel,
            table_address: u64,
            bits: usize,
            range: (u64, u64),
            deallocator: &mut impl FrameDeallocator<Size4KiB>,
        ) -> Result<bool, ()> {
            if let Some(next_level) = level.next_lower_level() {
                let span = level.entry_address_space_alignment();
                for (index, entry) in table.iter_mut().enumerate() {
                    let Ok(frame) = entry.frame(false) else {
                        continue;
                    };
                    let offset = span.checked_mul(index as u64).ok_or(())?;
                    let start = crate::addr::forward_checked_with_bits(table_address, offset, bits)
                        .ok_or(())?;
                    let end =
                        crate::addr::forward_checked_with_bits(start, span - 1, bits).ok_or(())?;
                    let child_range = (start.max(range.0), end.min(range.1));
                    if child_range.0 > child_range.1 {
                        continue;
                    }
                    let Ok(child) = walker.next_table_mut(entry) else {
                        continue;
                    };
                    if unsafe {
                        clean_range(
                            child,
                            walker,
                            next_level,
                            start,
                            bits,
                            child_range,
                            deallocator,
                        )?
                    } {
                        entry.set_unused();
                        unsafe { deallocator.deallocate_frame(frame) };
                    }
                }
            }
            Ok(table.is_empty())
        }

        // The root covers the whole canonical address space.
        let table_address = 0;
        let bits = BITS;
        unsafe {
            clean_range(
                self.root_table,
                &self.page_table_walker,
                self.root_level().as_page_table_level(),
                table_address,
                bits,
                (
                    range.start.start_address().as_u64(),
                    range.end.start_address().as_u64(),
                ),
                frame_deallocator,
            )
        }
        .expect("cleanup arithmetic must stay inside the root's canonical address space");
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
            .frame_to_pointer(entry.frame(false)?);
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
            .frame_to_pointer(entry.frame(false)?);
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
/// The returned pointer must be accessible in the currently active address space,
/// must point to the requested frame's page table, and must remain valid for the
/// duration of the mapper operation. This requirement concerns the active address
/// space only; an offline root being edited need not match the current `CR4.LA57`
/// mode when using `MappedPageTable`.
pub unsafe trait PageTableFrameMapping {
    /// Translate the given physical frame to a virtual page table pointer.
    fn frame_to_pointer(&self, frame: PhysFrame) -> *mut PageTable;
}

unsafe impl<P: PageTableFrameMapping + ?Sized> PageTableFrameMapping for &P {
    #[inline]
    fn frame_to_pointer(&self, frame: PhysFrame) -> *mut PageTable {
        (**self).frame_to_pointer(frame)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::PhysAddr;
    use crate::addr::VirtAddr;
    use crate::structures::paging::{
        FrameAllocator, FrameDeallocator, Mapper, PhysFrame, Size4KiB,
    };

    #[cfg(feature = "virt_addr_57")]
    use crate::structures::paging::{PageTableIndex, Size1GiB, Size2MiB};

    #[derive(Debug, Clone, Copy)]
    struct IdentityFrameMapping;

    unsafe impl PageTableFrameMapping for IdentityFrameMapping {
        fn frame_to_pointer(&self, frame: PhysFrame) -> *mut PageTable {
            frame.start_address().as_u64() as *mut PageTable
        }
    }

    #[derive(Debug, Default)]
    struct BoxFrameAllocator;

    unsafe impl FrameAllocator<Size4KiB> for BoxFrameAllocator {
        fn allocate_frame(&mut self) -> Option<PhysFrame<Size4KiB>> {
            let pointer = Box::into_raw(Box::new(PageTable::new()));
            PhysFrame::from_start_address(PhysAddr::new(pointer as u64)).ok()
        }
    }

    #[cfg(feature = "virt_addr_57")]
    #[derive(Debug, Default)]
    struct NoopFrameDeallocator;

    #[cfg(feature = "virt_addr_57")]
    impl FrameDeallocator<Size4KiB> for NoopFrameDeallocator {
        unsafe fn deallocate_frame(&mut self, _frame: PhysFrame<Size4KiB>) {}
    }

    fn frame(address: u64) -> PhysFrame<Size4KiB> {
        PhysFrame::from_start_address(PhysAddr::new(address)).unwrap()
    }

    // Deliberately align the P2 and P1 frames to huge-page boundaries: alignment
    // alone must not make a child-table pointer look like a huge-page mapping.
    const TRANSLATION_TABLE_FRAMES: [u64; 4] = [0x1000, 0x2000, 0x4000_0000, 0x20_0000];

    struct TranslationMapping([*mut PageTable; 4]);

    unsafe impl PageTableFrameMapping for TranslationMapping {
        fn frame_to_pointer(&self, frame: PhysFrame) -> *mut PageTable {
            let index = TRANSLATION_TABLE_FRAMES
                .iter()
                .position(|&address| address == frame.start_address().as_u64())
                .expect("only fixture page-table frames may be traversed");
            self.0[index]
        }
    }

    fn with_translation_tree<const BITS: usize>(
        leaf: PageTableLevel,
        flags: PageTableFlags,
        check: impl FnOnce(&MappedPageTable<'_, TranslationMapping, BITS>),
    ) where
        FixedValidity<BITS>: VirtAddrValidity,
    {
        let mut tables = Box::new([const { PageTable::new() }; 4]);
        for index in 0..3 {
            tables[index][0].set_addr(
                PhysAddr::new(TRANSLATION_TABLE_FRAMES[index + 1]),
                PageTableFlags::PRESENT,
            );
        }
        tables[4 - leaf as usize][0].set_addr(PhysAddr::new(0x8000_0000), flags);
        let mapping = TranslationMapping(tables.each_mut().map(|table| table as *mut PageTable));
        let mut root = PageTable::new();
        root[0].set_addr(
            PhysAddr::new(TRANSLATION_TABLE_FRAMES[if BITS == 57 { 0 } else { 1 }]),
            PageTableFlags::PRESENT,
        );
        // The fixture owns every table for the mapper's entire lifetime. It is offline.
        let mapper = unsafe { MappedPageTable::<_, BITS>::new(&mut root, mapping) };
        check(&mapper);
    }

    fn check_translation_rejects_child_tables<const BITS: usize>()
    where
        FixedValidity<BITS>: VirtAddrValidity,
    {
        with_translation_tree::<BITS>(PageTableLevel::One, PageTableFlags::PRESENT, |mapper| {
            let address = VirtAddrGeneric::<FixedValidity<BITS>>::zero();
            assert!(matches!(
                mapper.translate_page(Page::<Size2MiB, _>::containing_address(address)),
                Err(TranslateError::PageNotMapped)
            ));
            assert!(matches!(
                mapper.translate_page(Page::<Size1GiB, _>::containing_address(address)),
                Err(TranslateError::PageNotMapped)
            ));
            assert_eq!(
                mapper.translate_addr(address),
                Some(PhysAddr::new(0x8000_0000))
            );
        });
    }

    #[test]
    fn translation_rejects_child_tables_as_huge_pages() {
        check_translation_rejects_child_tables::<48>();
        #[cfg(feature = "virt_addr_57")]
        check_translation_rejects_child_tables::<57>();
    }

    fn check_translation_leaf<const BITS: usize, S: PageSize>(
        level: PageTableLevel,
        flags: PageTableFlags,
        present: bool,
    ) where
        FixedValidity<BITS>: VirtAddrValidity,
        for<'a> MappedPageTable<'a, TranslationMapping, BITS>: Mapper<S, FixedValidity<BITS>>,
    {
        with_translation_tree::<BITS>(level, flags, |mapper| {
            let address = VirtAddrGeneric::<FixedValidity<BITS>>::new(0x123);
            let page = Page::<S, _>::containing_address(address);
            if present {
                assert_eq!(
                    mapper.translate_page(page).unwrap().start_address(),
                    PhysAddr::new(0x8000_0000)
                );
                match mapper.translate(address) {
                    TranslateResult::Mapped {
                        frame,
                        offset,
                        flags: actual_flags,
                    } => {
                        assert_eq!(frame.start_address(), PhysAddr::new(0x8000_0000));
                        assert_eq!(frame.size(), S::SIZE);
                        assert_eq!(offset, 0x123);
                        assert_eq!(actual_flags, flags);
                    }
                    result => panic!("present leaf was not translated: {result:?}"),
                }
                assert_eq!(
                    mapper.translate_addr(address),
                    Some(PhysAddr::new(0x8000_0123))
                );
            } else {
                assert!(matches!(
                    mapper.translate_page(page),
                    Err(TranslateError::PageNotMapped)
                ));
                assert!(matches!(
                    mapper.translate(address),
                    TranslateResult::NotMapped
                ));
                assert_eq!(mapper.translate_addr(address), None);
            }
        });
    }

    fn check_translation_leaves<const BITS: usize>(present: bool)
    where
        FixedValidity<BITS>: VirtAddrValidity,
    {
        let flags = PageTableFlags::BIT_9
            | if present {
                PageTableFlags::PRESENT
            } else {
                PageTableFlags::empty()
            };
        check_translation_leaf::<BITS, Size4KiB>(PageTableLevel::One, flags, present);
        // Bit 7 is PAT at P1, so it must not make a 4KiB leaf invalid.
        let flags = flags | PageTableFlags::HUGE_PAGE;
        check_translation_leaf::<BITS, Size4KiB>(PageTableLevel::One, flags, present);
        check_translation_leaf::<BITS, Size2MiB>(PageTableLevel::Two, flags, present);
        check_translation_leaf::<BITS, Size1GiB>(PageTableLevel::Three, flags, present);
    }

    #[test]
    fn translation_rejects_non_present_leaves() {
        check_translation_leaves::<48>(false);
        #[cfg(feature = "virt_addr_57")]
        check_translation_leaves::<57>(false);
    }

    #[test]
    fn translation_preserves_present_leaves() {
        check_translation_leaves::<48>(true);
        #[cfg(feature = "virt_addr_57")]
        check_translation_leaves::<57>(true);
    }

    #[cfg(feature = "virt_addr_57")]
    #[test]
    fn l5_48_bit_addresses_use_distinct_p5_entries() {
        let root = Box::leak(Box::new(PageTable::new()));
        let mut mapper = unsafe { MappedPageTable::<_, 57>::new(root, IdentityFrameMapping) };
        let mut allocator = BoxFrameAllocator;
        let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;

        let low = Page::<Size4KiB>::containing_address(VirtAddr::new(0x4000));
        let high = Page::<Size4KiB>::containing_address(VirtAddr::new(0xffff_8000_0000_4000));
        unsafe {
            <MappedPageTable<'_, IdentityFrameMapping, 57> as Mapper<Size4KiB>>::map_to(
                &mut mapper,
                low,
                frame(0x20_0000),
                flags,
                &mut allocator,
            )
            .unwrap()
            .ignore();
            <MappedPageTable<'_, IdentityFrameMapping, 57> as Mapper<Size4KiB>>::map_to(
                &mut mapper,
                high,
                frame(0x30_0000),
                flags,
                &mut allocator,
            )
            .unwrap()
            .ignore();
        }

        assert!(
            mapper.root_table()[PageTableIndex::new(0)]
                .flags()
                .contains(PageTableFlags::PRESENT)
        );
        assert!(
            mapper.root_table()[PageTableIndex::new(511)]
                .flags()
                .contains(PageTableFlags::PRESENT)
        );
        assert_eq!(
            mapper.translate_addr(low.start_address()),
            Some(PhysAddr::new(0x20_0000))
        );
        assert_eq!(
            mapper.translate_addr(high.start_address()),
            Some(PhysAddr::new(0x30_0000))
        );
    }

    #[cfg(feature = "virt_addr_57")]
    #[test]
    fn l5_maps_and_translates_all_page_sizes() {
        let root = Box::leak(Box::new(PageTable::new()));
        let mut mapper = unsafe { MappedPageTable::<_, 57>::new(root, IdentityFrameMapping) };
        let mut allocator = BoxFrameAllocator;
        let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;

        let page_1g = Page::<Size1GiB>::containing_address(VirtAddr::new(0x4000_0000));
        let page_2m = Page::<Size2MiB>::containing_address(VirtAddr::new(0x8000_0000));
        let page_4k = Page::<Size4KiB>::containing_address(VirtAddr::new(0x1000));
        unsafe {
            <MappedPageTable<'_, IdentityFrameMapping, 57> as Mapper<Size1GiB>>::map_to(
                &mut mapper,
                page_1g,
                PhysFrame::from_start_address(PhysAddr::new(0x4000_0000)).unwrap(),
                flags,
                &mut allocator,
            )
            .unwrap()
            .ignore();
            <MappedPageTable<'_, IdentityFrameMapping, 57> as Mapper<Size2MiB>>::map_to(
                &mut mapper,
                page_2m,
                PhysFrame::from_start_address(PhysAddr::new(0x8000_0000)).unwrap(),
                flags,
                &mut allocator,
            )
            .unwrap()
            .ignore();
            <MappedPageTable<'_, IdentityFrameMapping, 57> as Mapper<Size4KiB>>::map_to(
                &mut mapper,
                page_4k,
                frame(0x1000),
                flags,
                &mut allocator,
            )
            .unwrap()
            .ignore();
            <MappedPageTable<'_, IdentityFrameMapping, 57> as Mapper<Size4KiB>>::set_flags_p5_entry(
                &mut mapper,
                page_4k,
                flags,
            )
            .unwrap()
            .ignore();
        }

        assert_eq!(
            mapper.translate_addr(page_1g.start_address()),
            Some(PhysAddr::new(0x4000_0000))
        );
        assert_eq!(
            mapper.translate_addr(page_2m.start_address()),
            Some(PhysAddr::new(0x8000_0000))
        );
        assert_eq!(
            mapper.translate_addr(page_4k.start_address()),
            Some(PhysAddr::new(0x1000))
        );

        unsafe {
            <MappedPageTable<'_, IdentityFrameMapping, 57> as Mapper<Size2MiB>>::update_flags(
                &mut mapper,
                page_2m,
                PageTableFlags::PRESENT,
            )
            .unwrap()
            .ignore();
        }
        match mapper.translate(page_2m.start_address()) {
            TranslateResult::Mapped { flags, .. } => {
                assert!(flags.contains(PageTableFlags::PRESENT))
            }
            _ => panic!("2MiB mapping disappeared after flag update"),
        }

        let (_, _, flush) =
            <MappedPageTable<'_, IdentityFrameMapping, 57> as Mapper<Size2MiB>>::unmap(
                &mut mapper,
                page_2m,
            )
            .unwrap();
        flush.ignore();
        assert_eq!(mapper.translate_addr(page_2m.start_address()), None);
    }

    #[cfg(feature = "virt_addr_57")]
    #[test]
    fn l5_cleanup_walks_tree_and_spans_canonical_halves() {
        let root = Box::leak(Box::new(PageTable::new()));
        let mut mapper = unsafe { MappedPageTable::<_, 57>::new(root, IdentityFrameMapping) };
        let mut allocator = BoxFrameAllocator;
        let page = Page::<Size4KiB>::containing_address(VirtAddr::new(0x1000));
        unsafe {
            <MappedPageTable<'_, IdentityFrameMapping, 57> as Mapper<Size4KiB>>::map_to(
                &mut mapper,
                page,
                frame(0x60_0000),
                PageTableFlags::PRESENT,
                &mut allocator,
            )
            .unwrap()
            .ignore();
        }
        let _ = <MappedPageTable<'_, IdentityFrameMapping, 57> as Mapper<Size4KiB>>::clear(
            &mut mapper,
            page,
        )
        .unwrap();
        let mut deallocator = NoopFrameDeallocator;
        unsafe { mapper.clean_up(&mut deallocator) };
        assert!(mapper.root_table().iter().all(PageTableEntry::is_unused));

        let root = Box::leak(Box::new(PageTable::new()));
        let mut mapper = unsafe { MappedPageTable::<_, 57>::new(root, IdentityFrameMapping) };
        let lower = Page::<Size4KiB>::containing_address(VirtAddr::new(0));
        let upper = Page::<Size4KiB>::containing_address(VirtAddr::new(0xffff_8000_0000_0000));
        let range = Page::range_inclusive(lower, upper);
        unsafe { mapper.clean_up_addr_range(range, &mut deallocator) };
        assert!(mapper.root_table().is_empty());
    }

    #[test]
    fn range_cleanup_frees_both_halves_of_l4_and_l5_roots() {
        struct CountingDeallocator(usize);
        impl FrameDeallocator<Size4KiB> for CountingDeallocator {
            unsafe fn deallocate_frame(&mut self, _frame: PhysFrame<Size4KiB>) {
                self.0 += 1;
            }
        }
        fn check<const BITS: usize>(expected_frames: usize)
        where
            FixedValidity<BITS>: VirtAddrValidity,
            VirtAddrGeneric<FixedValidity<BITS>>: From<VirtAddr>,
        {
            let root = Box::leak(Box::new(PageTable::new()));
            let mut mapper = unsafe { MappedPageTable::<_, BITS>::new(root, IdentityFrameMapping) };
            let mut allocator = BoxFrameAllocator;
            for address in [0x1000, 0xffff_8000_0000_1000] {
                let page = Page::<Size4KiB>::containing_address(VirtAddr::new(address));
                unsafe {
                    mapper
                        .map_to(
                            page,
                            frame(0x70_0000),
                            PageTableFlags::PRESENT,
                            &mut allocator,
                        )
                        .unwrap()
                        .ignore();
                }
                let (_, _, flush) = mapper.unmap(page).unwrap();
                flush.ignore();
            }
            let range = Page::<Size4KiB>::range_inclusive(
                Page::containing_address(VirtAddr::zero()),
                Page::containing_address(VirtAddr::new(u64::MAX)),
            );
            let mut deallocator = CountingDeallocator(0);
            unsafe { mapper.clean_up_addr_range(range, &mut deallocator) };
            assert!(mapper.root_table().is_empty());
            assert_eq!(deallocator.0, expected_frames);
        }
        check::<48>(6);
        #[cfg(feature = "virt_addr_57")]
        check::<57>(8);
    }

    #[test]
    fn l4_iterator_keeps_upper_half_canonical() {
        let root = Box::leak(Box::new(PageTable::new()));
        let mut mapper = unsafe { MappedPageTable::new(root, IdentityFrameMapping) };
        let mut allocator = BoxFrameAllocator;
        let page = Page::<Size4KiB>::containing_address(VirtAddr::new(0xffff_8000_0000_0000));
        unsafe {
            <MappedPageTable<'_, IdentityFrameMapping> as Mapper<Size4KiB>>::map_to(
                &mut mapper,
                page,
                frame(0x50_0000),
                PageTableFlags::PRESENT,
                &mut allocator,
            )
            .unwrap()
            .ignore();
        }
        let item = mapper.iter().next().unwrap();
        let address = match item {
            iter::MappedPageItem::Size4KiB(mapped) => mapped.page.start_address().as_u64(),
            _ => panic!("unexpected page size"),
        };
        assert_eq!(address, 0xffff_8000_0000_0000);
    }
    #[test]
    fn roots_are_fixed_by_the_type() {
        let mut root = PageTable::new();
        let mapper: MappedPageTable<'_, _> =
            unsafe { MappedPageTable::new(&mut root, IdentityFrameMapping) };
        assert_eq!(mapper.root_level(), PageTableRootLevel::Four);
        #[cfg(feature = "virt_addr_57")]
        {
            let mut root = PageTable::new();
            let mapper = unsafe { MappedPageTable::<_, 57>::new(&mut root, IdentityFrameMapping) };
            assert_eq!(mapper.root_level(), PageTableRootLevel::Five);
        }
    }

    /// Reclaims only the Box allocations made by BoxFrameAllocator, after unmapping.
    struct BoxFrameDeallocator;
    impl FrameDeallocator<Size4KiB> for BoxFrameDeallocator {
        unsafe fn deallocate_frame(&mut self, frame: PhysFrame) {
            unsafe {
                drop(Box::from_raw(
                    frame.start_address().as_u64() as *mut PageTable
                ))
            };
        }
    }

    #[cfg(feature = "virt_addr_57")]
    fn exercise_l5_mapping<S: PageSize + core::fmt::Debug, V: VirtAddrValidity>(page: Page<S, V>)
    where
        crate::addr::VirtAddr57: From<VirtAddrGeneric<V>>,
        for<'a> MappedPageTable<'a, IdentityFrameMapping, 57>: Mapper<S, V>,
    {
        let mut root = PageTable::new();
        let mut mapper = unsafe { MappedPageTable::<_, 57>::new(&mut root, IdentityFrameMapping) };
        let mut allocator = BoxFrameAllocator;
        let physical = PhysFrame::<S>::from_start_address(PhysAddr::new(Size1GiB::SIZE)).unwrap();
        let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;
        let flush = unsafe { mapper.map_to(page, physical, flags, &mut allocator) }.unwrap();
        assert_eq!(
            flush.page().start_address().as_u64(),
            page.start_address().as_u64()
        );
        flush.ignore();
        assert_eq!(mapper.translate_page(page).unwrap(), physical);
        assert_eq!(
            mapper.translate_addr(page.start_address()),
            Some(physical.start_address())
        );
        // The same tree is translated using its fixed VA57 type, regardless of input policy.
        let fixed = crate::addr::VirtAddr57::from(page.start_address());
        assert_eq!(
            Translate::<FixedValidity<57>>::translate_addr(&mapper, fixed),
            Some(physical.start_address())
        );
        let mut iter = mapper.iter();
        let item: iter::MappedPageItem<FixedValidity<57>> = iter.next().unwrap();
        let (iter_addr, iter_size) = match item {
            iter::MappedPageItem::Size4KiB(mapping) => {
                (mapping.page.start_address(), Size4KiB::SIZE)
            }
            iter::MappedPageItem::Size2MiB(mapping) => {
                (mapping.page.start_address(), Size2MiB::SIZE)
            }
            iter::MappedPageItem::Size1GiB(mapping) => {
                (mapping.page.start_address(), Size1GiB::SIZE)
            }
        };
        assert_eq!(iter_addr, fixed);
        assert_eq!(iter_size, S::SIZE);
        assert!(iter.next().is_none());
        let rendered = std::format!("{:#}", mapper.display());
        assert!(rendered.contains(S::DEBUG_STR));
        assert!(rendered.contains(&std::format!("{:x}", fixed.as_u64())));
        let flags = PageTableFlags::PRESENT;
        unsafe { mapper.update_flags(page, flags) }
            .unwrap()
            .ignore();
        match Translate::<FixedValidity<57>>::translate(&mapper, fixed) {
            TranslateResult::Mapped {
                frame,
                flags: actual,
                ..
            } => {
                assert_eq!(frame.size(), S::SIZE);
                assert!(!actual.contains(PageTableFlags::WRITABLE));
            }
            _ => panic!("mapping disappeared"),
        }
        let (unmapped, _, flush) = mapper.unmap(page).unwrap();
        flush.ignore();
        assert_eq!(unmapped, physical);
        assert_eq!(
            Translate::<FixedValidity<57>>::translate_addr(&mapper, fixed),
            None
        );
        unsafe { mapper.clean_up(&mut BoxFrameDeallocator) };
        assert!(mapper.root_table().is_empty());
    }

    #[cfg(feature = "virt_addr_57")]
    #[test]
    fn l5_supports_every_fixed_policy_and_page_size() {
        for raw in [0x4000_0000, 0xffff_8000_4000_0000] {
            let address = VirtAddr::new(raw);
            exercise_l5_mapping(Page::<Size4KiB>::containing_address(address));
            exercise_l5_mapping(Page::<Size2MiB>::containing_address(address));
            exercise_l5_mapping(Page::<Size1GiB>::containing_address(address));
        }
        for raw in [0x0001_0000_4000_0000, 0xff00_0000_4000_0000] {
            let address = crate::addr::VirtAddr57::new(raw);
            exercise_l5_mapping(Page::<Size4KiB, FixedValidity<57>>::containing_address(
                address,
            ));
            exercise_l5_mapping(Page::<Size2MiB, FixedValidity<57>>::containing_address(
                address,
            ));
            exercise_l5_mapping(Page::<Size1GiB, FixedValidity<57>>::containing_address(
                address,
            ));
        }
    }

    #[cfg(all(
        target_arch = "x86_64",
        feature = "virt_addr_57",
        feature = "virt_addr_rt"
    ))]
    #[test]
    fn l5_runtime_inputs_do_not_consult_the_active_mode() {
        use crate::addr::{RuntimeValidity, VirtAddrRT};
        // The infallible VA48 conversion never reads CR4 or initializes its cache.
        for raw in [0x4000_0000, 0xffff_8000_4000_0000] {
            let address = VirtAddrRT::from(VirtAddr::new(raw));
            exercise_l5_mapping(Page::<Size4KiB, RuntimeValidity>::containing_address(
                address,
            ));
            exercise_l5_mapping(Page::<Size2MiB, RuntimeValidity>::containing_address(
                address,
            ));
            exercise_l5_mapping(Page::<Size1GiB, RuntimeValidity>::containing_address(
                address,
            ));
        }
        let mut root = PageTable::new();
        let mut mapper = unsafe { MappedPageTable::<_, 57>::new(&mut root, IdentityFrameMapping) };
        let address = VirtAddrRT::from(VirtAddr::new(0x1000));
        let page = Page::<Size4KiB, RuntimeValidity>::containing_address(address);
        unsafe {
            mapper
                .map_to(
                    page,
                    frame(0x2000),
                    PageTableFlags::PRESENT,
                    &mut BoxFrameAllocator,
                )
                .unwrap()
                .ignore();
        }
        mapper.unmap(page).unwrap().2.ignore();
        unsafe {
            mapper.clean_up_addr_range(Page::range_inclusive(page, page), &mut BoxFrameDeallocator)
        };
        assert!(mapper.root_table().is_empty());
    }

    fn assert_range_addresses<const BITS: usize>(addresses: &[(u64, u64)], expected: &[(u64, u64)])
    where
        FixedValidity<BITS>: VirtAddrValidity,
    {
        use super::range_iter::MappedPageRangeInclusiveItem;
        let mut root = PageTable::new();
        let mut mapper =
            unsafe { MappedPageTable::<_, BITS>::new(&mut root, IdentityFrameMapping) };
        for &(virtual_address, physical_address) in addresses {
            let page =
                Page::<Size4KiB, FixedValidity<BITS>>::containing_address(VirtAddrGeneric::<
                    FixedValidity<BITS>,
                >::new(
                    virtual_address
                ));
            unsafe {
                mapper
                    .map_to(
                        page,
                        frame(physical_address),
                        PageTableFlags::PRESENT,
                        &mut BoxFrameAllocator,
                    )
                    .unwrap()
                    .ignore();
            }
        }
        let mut ranges = mapper.range_iter();
        for &(start, end) in expected {
            let Some(MappedPageRangeInclusiveItem::Size4KiB(range)) = ranges.next() else {
                panic!("missing 4KiB range");
            };
            let pages: PageRangeInclusive<Size4KiB, FixedValidity<BITS>> = range.page_range();
            assert_eq!(pages.start.start_address().as_u64(), start);
            assert_eq!(pages.end.start_address().as_u64(), end);
            assert_eq!(range.len(), (end - start) / Size4KiB::SIZE + 1);
            assert_eq!(range.frame_range().len(), range.len());
        }
        assert!(ranges.next().is_none());
        assert!(ranges.next().is_none());
        let rendered = std::format!("{:#}", mapper.display());
        assert_eq!(rendered.lines().count(), expected.len() + 1);
        for &(start, _) in expected {
            assert!(rendered.contains(&std::format!("{start:x}")));
        }
        for &(virtual_address, _) in addresses {
            let page =
                Page::<Size4KiB, FixedValidity<BITS>>::containing_address(VirtAddrGeneric::<
                    FixedValidity<BITS>,
                >::new(
                    virtual_address
                ));
            mapper.unmap(page).unwrap().2.ignore();
        }
        unsafe { mapper.clean_up(&mut BoxFrameDeallocator) };
    }

    #[test]
    fn typed_range_iteration_splits_holes_and_handles_last_frames() {
        // Two contiguous mappings, a physically noncontiguous mapping, both canonical
        // halves, and the highest virtual page. The first frame after a gap is the
        // highest physical frame, so advancing it while searching must not panic.
        assert_range_addresses::<48>(
            &[
                (0x1000, 0x2000),
                (0x2000, 0x3000),
                (0x3000, 0x6000),
                (0x7fff_ffff_f000, 0x000f_ffff_ffff_f000),
                (0xffff_8000_0000_0000, 0x7000),
                (0xffff_ffff_ffff_f000, 0x8000),
            ],
            &[
                (0x1000, 0x2000),
                (0x3000, 0x3000),
                (0x7fff_ffff_f000, 0x7fff_ffff_f000),
                (0xffff_8000_0000_0000, 0xffff_8000_0000_0000),
                (0xffff_ffff_ffff_f000, 0xffff_ffff_ffff_f000),
            ],
        );
        #[cfg(feature = "virt_addr_57")]
        assert_range_addresses::<57>(
            &[
                (0x0001_0000_0000_0000, 0x2000),
                (0x0001_0000_0000_1000, 0x3000),
                (0x00ff_ffff_ffff_f000, 0x000f_ffff_ffff_f000),
                (0xff00_0000_0000_0000, 0x4000),
                (0xffff_ffff_ffff_f000, 0x5000),
            ],
            &[
                (0x0001_0000_0000_0000, 0x0001_0000_0000_1000),
                (0x00ff_ffff_ffff_f000, 0x00ff_ffff_ffff_f000),
                (0xff00_0000_0000_0000, 0xff00_0000_0000_0000),
                (0xffff_ffff_ffff_f000, 0xffff_ffff_ffff_f000),
            ],
        );
    }
    #[test]
    fn range_iteration_splits_different_page_sizes_and_flags() {
        use super::range_iter::MappedPageRangeInclusiveItem;
        use crate::structures::paging::{Size1GiB, Size2MiB};

        fn check<const BITS: usize>()
        where
            FixedValidity<BITS>: VirtAddrValidity,
        {
            let mut root = PageTable::new();
            let mut mapper =
                unsafe { MappedPageTable::<_, BITS>::new(&mut root, IdentityFrameMapping) };
            let mut allocator = BoxFrameAllocator;
            let present = PageTableFlags::PRESENT;
            let writable = present | PageTableFlags::WRITABLE;
            let giant =
                Page::<Size1GiB, FixedValidity<BITS>>::containing_address(VirtAddrGeneric::<
                    FixedValidity<BITS>,
                >::zero(
                ));
            let huge =
                Page::<Size2MiB, FixedValidity<BITS>>::containing_address(VirtAddrGeneric::<
                    FixedValidity<BITS>,
                >::new(
                    0x4000_0000
                ));
            // All mappings are contiguous in both address spaces. Only page sizes
            // and flags separate the four output ranges.
            unsafe {
                mapper
                    .map_to(
                        giant,
                        PhysFrame::containing_address(PhysAddr::zero()),
                        present,
                        &mut allocator,
                    )
                    .unwrap()
                    .ignore();
                mapper
                    .map_to(
                        huge,
                        PhysFrame::containing_address(PhysAddr::new(0x4000_0000)),
                        present,
                        &mut allocator,
                    )
                    .unwrap()
                    .ignore();
            }
            for (address, flags) in [
                (0x4020_0000, present),
                (0x4020_1000, writable),
                (0x4020_2000, writable),
            ] {
                let page =
                    Page::<Size4KiB, FixedValidity<BITS>>::containing_address(VirtAddrGeneric::<
                        FixedValidity<BITS>,
                    >::new(
                        address
                    ));
                unsafe { mapper.map_to(page, frame(address), flags, &mut allocator) }
                    .unwrap()
                    .ignore();
            }

            let mut ranges = mapper.range_iter();
            let Some(MappedPageRangeInclusiveItem::Size1GiB(range)) = ranges.next() else {
                panic!("expected a separate 1GiB range");
            };
            assert_eq!(range.len(), 1);
            assert_eq!(range.page_range().start, giant);
            let Some(MappedPageRangeInclusiveItem::Size2MiB(range)) = ranges.next() else {
                panic!("expected a separate 2MiB range");
            };
            assert_eq!(range.len(), 1);
            assert_eq!(range.page_range().start, huge);
            for (start, end, flags, len) in [
                (0x4020_0000, 0x4020_0000, present, 1),
                (0x4020_1000, 0x4020_2000, writable, 2),
            ] {
                let Some(MappedPageRangeInclusiveItem::Size4KiB(range)) = ranges.next() else {
                    panic!("expected a separate 4KiB range for each flag set");
                };
                assert_eq!(range.page_range().start.start_address().as_u64(), start);
                assert_eq!(range.page_range().end.start_address().as_u64(), end);
                assert_eq!(range.flags(), flags);
                assert_eq!(range.len(), len);
            }
            assert!(ranges.next().is_none());
            assert!(ranges.next().is_none());

            mapper.unmap(giant).unwrap().2.ignore();
            mapper.unmap(huge).unwrap().2.ignore();
            for address in [0x4020_0000, 0x4020_1000, 0x4020_2000] {
                let page =
                    Page::<Size4KiB, FixedValidity<BITS>>::containing_address(VirtAddrGeneric::<
                        FixedValidity<BITS>,
                    >::new(
                        address
                    ));
                mapper.unmap(page).unwrap().2.ignore();
            }
            unsafe { mapper.clean_up(&mut BoxFrameDeallocator) };
            assert!(mapper.root_table().is_empty());
        }

        check::<48>();
        #[cfg(feature = "virt_addr_57")]
        check::<57>();
    }
}
