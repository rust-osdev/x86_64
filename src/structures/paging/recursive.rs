use instructions::tlb;
use registers::control::Cr3;
use structures::paging::page_table::{FrameError, PageTable, PageTableEntry, PageTableFlags};
use structures::paging::{NotGiantPageSize, Page, PageSize, PhysFrame, Size1GiB, Size2MiB, Size4KiB};
use ux::u9;
use {PhysAddr, VirtAddr};

/// This type represents a page whose mapping has changed in the page table.
///
/// The old mapping might be still cached in the translation lookaside buffer (TLB), so it needs
/// to be flushed from the TLB before it's accessed. This type is returned from function that
/// change the mapping of a page to ensure that the TLB flush is not forgotten.
#[must_use = "Page Table changes must be flushed or ignored."]
pub struct MapperFlush<S: PageSize>(Page<S>);

impl<S: PageSize> MapperFlush<S> {
    /// Create a new flush promise
    fn new(page: Page<S>) -> Self {
        MapperFlush(page)
    }

    /// Flush the page from the TLB to ensure that the newest mapping is used.
    pub fn flush(self) {
        tlb::flush(self.0.start_address());
    }

    /// Don't flush the TLB and silence the “must be used” warning.
    pub fn ignore(self) {}
}

/// A trait for common page table operations.
pub trait Mapper<S: PageSize> {
    /// Creates a new mapping in the page table.
    ///
    /// This function might need additional physical frames to create new page tables. These
    /// frames are allocated from the `allocator` argument. At most three frames are required.
    fn map_to<A>(
        &mut self,
        page: Page<S>,
        frame: PhysFrame<S>,
        flags: PageTableFlags,
        allocator: A,
    ) -> Result<MapperFlush<S>, MapToError>
    where
        A: FnMut() -> Option<PhysFrame>;

    /// Removes a mapping from the page table.
    ///
    /// If this function is successful, it deallocates the mapped frame via the passed `allocator`.
    fn unmap<A>(&mut self, page: Page<S>, allocator: A) -> Result<MapperFlush<S>, UnmapError>
    where
        A: FnOnce(PhysFrame<S>);

    /// Updates the flags of an existing mapping.
    fn update_flags(
        &mut self,
        page: Page<S>,
        flags: PageTableFlags,
    ) -> Result<MapperFlush<S>, FlagUpdateError>;

    /// Return the frame that the specified page is mapped to.
    fn translate_page(&self, page: Page<S>) -> Option<PhysFrame<S>>;

    /// Maps the given frame to the virtual page with the same address.
    fn identity_map<A>(
        &mut self,
        frame: PhysFrame<S>,
        flags: PageTableFlags,
        allocator: &mut A,
    ) -> Result<MapperFlush<S>, MapToError>
    where
        A: FnMut() -> Option<PhysFrame>,
        S: PageSize,
        Self: Mapper<S>,
    {
        let page = Page::containing_address(VirtAddr::new(frame.start_address().as_u64()));
        self.map_to(page, frame, flags, allocator)
    }
}

/// A recursive page table is a last level page table with an entry mapped to the table itself.
///
/// This recursive mapping allows accessing all page tables in the hierarchy:
///
/// - To access the level 4 page table, we “loop“ (i.e. follow the recursively mapped entry) four
///   times.
/// - To access a level 3 page table, we “loop” three times and then use the level 4 index.
/// - To access a level 2 page table, we “loop” two times, then use the level 4 index, then the
///   level 3 index.
/// - To access a level 1 page table, we “loop” once, then use the level 4 index, then the
///   level 3 index, then the level 2 index.
///
/// This struct implements the `Mapper` trait.
pub struct RecursivePageTable<'a> {
    p4: &'a mut PageTable,
    recursive_index: u9,
}

/// An error indicating that the given page table is not recursively mapped.
///
/// Returned from `RecursivePageTable::new`.
#[derive(Debug)]
pub struct NotRecursivelyMapped;

/// This error is returned from `map_to` and similar methods.
#[derive(Debug)]
pub enum MapToError {
    /// An additional frame was needed for the mapping process, but the frame allocator
    /// returned `None`.
    FrameAllocationFailed,
    /// An upper level page table entry has the `HUGE_PAGE` flag set, which means that the
    /// given page is part of an already mapped huge page.
    ParentEntryHugePage,
    /// The given page is already mapped to a physical frame.
    PageAlreadyMapped,
}

/// An error indicating that an `unmap` call failed.
#[derive(Debug)]
pub enum UnmapError {
    /// An upper level page table entry has the `HUGE_PAGE` flag set, which means that the
    /// given page is part of a huge page and can't be freed individually.
    ParentEntryHugePage,
    /// The given page is not mapped to a physical frame.
    PageNotMapped,
    /// The page table entry for the given page points to an invalid physical address.
    InvalidFrameAddress(PhysAddr),
}

/// An error indicating that an `update_flags` call failed.
#[derive(Debug)]
pub enum FlagUpdateError {
    /// The given page is not mapped to a physical frame.
    PageNotMapped,
}

impl<'a> RecursivePageTable<'a> {
    /// Creates a new RecursivePageTable from the passed level 4 PageTable.
    ///
    /// The page table must be recursively mapped, that means:
    ///
    /// - The page table must have one recursive entry, i.e. an entry that points to the table
    ///   itself.
    ///     - The reference must use that “loop”, i.e. be of the form `0o_xxx_xxx_xxx_xxx_0000`
    ///       where `xxx` is the recursive entry.
    /// - The page table must be active, i.e. the CR3 register must contain its physical address.
    ///
    /// Otherwise `Err(NotRecursivelyMapped)` is returned.
    pub fn new(table: &'a mut PageTable) -> Result<Self, NotRecursivelyMapped> {
        let page = Page::containing_address(VirtAddr::new(table as *const _ as u64));
        let recursive_index = page.p4_index();

        if page.p3_index() != recursive_index
            || page.p2_index() != recursive_index
            || page.p1_index() != recursive_index
        {
            return Err(NotRecursivelyMapped);
        }
        if Ok(Cr3::read().0) != table[recursive_index].frame() {
            return Err(NotRecursivelyMapped);
        }

        Ok(RecursivePageTable {
            p4: table,
            recursive_index,
        })
    }

    /// Creates a new RecursivePageTable without performing any checks.
    ///
    /// The `recursive_index` parameter must be the index of the recursively mapped entry.
    pub unsafe fn new_unchecked(table: &'a mut PageTable, recursive_index: u9) -> Self {
        RecursivePageTable {
            p4: table,
            recursive_index,
        }
    }

    fn create_next_table<'b, A>(
        entry: &'b mut PageTableEntry,
        next_table_page: Page,
        mut allocator: A,
    ) -> Result<&'b mut PageTable, MapToError>
    where
        A: FnMut() -> Option<PhysFrame>,
    {
        use structures::paging::PageTableFlags as Flags;

        let created;

        if entry.is_unused() {
            if let Some(frame) = allocator() {
                entry.set_frame(frame, Flags::PRESENT | Flags::WRITABLE);
                created = true;
            } else {
                return Err(MapToError::FrameAllocationFailed);
            }
        } else {
            created = false;
        }
        if entry.flags().contains(Flags::HUGE_PAGE) {
            return Err(MapToError::ParentEntryHugePage);
        }

        let page_table_ptr = next_table_page.start_address().as_mut_ptr();
        let page_table: &mut PageTable = unsafe { &mut *(page_table_ptr) };
        if created {
            page_table.zero();
        }
        Ok(page_table)
    }
}

impl<'a> Mapper<Size1GiB> for RecursivePageTable<'a> {
    fn map_to<A>(
        &mut self,
        page: Page<Size1GiB>,
        frame: PhysFrame<Size1GiB>,
        flags: PageTableFlags,
        mut allocator: A,
    ) -> Result<MapperFlush<Size1GiB>, MapToError>
    where
        A: FnMut() -> Option<PhysFrame>,
    {
        use structures::paging::PageTableFlags as Flags;
        let p4 = &mut self.p4;

        let p3_page = p3_page(page, self.recursive_index);
        let p3 = Self::create_next_table(&mut p4[page.p4_index()], p3_page, &mut allocator)?;

        if !p3[page.p3_index()].is_unused() {
            return Err(MapToError::PageAlreadyMapped);
        }
        p3[page.p3_index()].set_addr(frame.start_address(), flags | Flags::HUGE_PAGE);

        Ok(MapperFlush::new(page))
    }

    fn unmap<A>(
        &mut self,
        page: Page<Size1GiB>,
        allocator: A,
    ) -> Result<MapperFlush<Size1GiB>, UnmapError>
    where
        A: FnOnce(PhysFrame<Size1GiB>),
    {
        let p4 = &mut self.p4;
        let p4_entry = &p4[page.p4_index()];

        p4_entry.frame().map_err(|err| match err {
            FrameError::FrameNotPresent => UnmapError::PageNotMapped,
            FrameError::HugeFrame => UnmapError::ParentEntryHugePage,
        })?;

        let p3 = unsafe { &mut *(p3_ptr(page, self.recursive_index)) };
        let p3_entry = &mut p3[page.p3_index()];
        let flags = p3_entry.flags();

        if !flags.contains(PageTableFlags::PRESENT) {
            return Err(UnmapError::PageNotMapped);
        }
        if !flags.contains(PageTableFlags::HUGE_PAGE) {
            return Err(UnmapError::ParentEntryHugePage);
        }

        let frame = PhysFrame::from_start_address(p3_entry.addr())
            .map_err(|()| UnmapError::InvalidFrameAddress(p3_entry.addr()))?;
        allocator(frame);
        p3_entry.set_unused();
        Ok(MapperFlush::new(page))
    }

    fn update_flags(
        &mut self,
        page: Page<Size1GiB>,
        flags: PageTableFlags,
    ) -> Result<MapperFlush<Size1GiB>, FlagUpdateError> {
        use structures::paging::PageTableFlags as Flags;
        let p4 = &mut self.p4;

        if p4[page.p4_index()].is_unused() {
            return Err(FlagUpdateError::PageNotMapped);
        }

        let p3 = unsafe { &mut *(p3_ptr(page, self.recursive_index)) };

        if p3[page.p3_index()].is_unused() {
            return Err(FlagUpdateError::PageNotMapped);
        }
        p3[page.p3_index()].set_flags(flags | Flags::HUGE_PAGE);

        Ok(MapperFlush::new(page))
    }

    fn translate_page(&self, page: Page<Size1GiB>) -> Option<PhysFrame<Size1GiB>> {
        let p4 = &self.p4;

        if p4[page.p4_index()].is_unused() {
            return None;
        }

        let p3 = unsafe { &*(p3_ptr(page, self.recursive_index)) };
        let p3_entry = &p3[page.p3_index()];

        if p3_entry.is_unused() {
            return None;
        }

        PhysFrame::from_start_address(p3_entry.addr()).ok()
    }
}

impl<'a> Mapper<Size2MiB> for RecursivePageTable<'a> {
    fn map_to<A>(
        &mut self,
        page: Page<Size2MiB>,
        frame: PhysFrame<Size2MiB>,
        flags: PageTableFlags,
        mut allocator: A,
    ) -> Result<MapperFlush<Size2MiB>, MapToError>
    where
        A: FnMut() -> Option<PhysFrame>,
    {
        use structures::paging::PageTableFlags as Flags;
        let p4 = &mut self.p4;

        let p3_page = p3_page(page, self.recursive_index);
        let p3 = Self::create_next_table(&mut p4[page.p4_index()], p3_page, &mut allocator)?;

        let p2_page = p2_page(page, self.recursive_index);
        let p2 = Self::create_next_table(&mut p3[page.p3_index()], p2_page, &mut allocator)?;

        if !p2[page.p2_index()].is_unused() {
            return Err(MapToError::PageAlreadyMapped);
        }
        p2[page.p2_index()].set_addr(frame.start_address(), flags | Flags::HUGE_PAGE);

        Ok(MapperFlush::new(page))
    }

    fn unmap<A>(
        &mut self,
        page: Page<Size2MiB>,
        allocator: A,
    ) -> Result<MapperFlush<Size2MiB>, UnmapError>
    where
        A: FnOnce(PhysFrame<Size2MiB>),
    {
        let p4 = &mut self.p4;
        let p4_entry = &p4[page.p4_index()];
        p4_entry.frame().map_err(|err| match err {
            FrameError::FrameNotPresent => UnmapError::PageNotMapped,
            FrameError::HugeFrame => UnmapError::ParentEntryHugePage,
        })?;

        let p3 = unsafe { &mut *(p3_ptr(page, self.recursive_index)) };
        let p3_entry = &p3[page.p3_index()];
        p3_entry.frame().map_err(|err| match err {
            FrameError::FrameNotPresent => UnmapError::PageNotMapped,
            FrameError::HugeFrame => UnmapError::ParentEntryHugePage,
        })?;

        let p2 = unsafe { &mut *(p2_ptr(page, self.recursive_index)) };
        let p2_entry = &mut p2[page.p2_index()];
        let flags = p2_entry.flags();

        if !flags.contains(PageTableFlags::PRESENT) {
            return Err(UnmapError::PageNotMapped);
        }
        if !flags.contains(PageTableFlags::HUGE_PAGE) {
            return Err(UnmapError::ParentEntryHugePage);
        }

        let frame = PhysFrame::from_start_address(p2_entry.addr())
            .map_err(|()| UnmapError::InvalidFrameAddress(p2_entry.addr()))?;
        allocator(frame);
        p2_entry.set_unused();
        Ok(MapperFlush::new(page))
    }

    fn update_flags(
        &mut self,
        page: Page<Size2MiB>,
        flags: PageTableFlags,
    ) -> Result<MapperFlush<Size2MiB>, FlagUpdateError> {
        use structures::paging::PageTableFlags as Flags;
        let p4 = &mut self.p4;

        if p4[page.p4_index()].is_unused() {
            return Err(FlagUpdateError::PageNotMapped);
        }

        let p3 = unsafe { &mut *(p3_ptr(page, self.recursive_index)) };

        if p3[page.p3_index()].is_unused() {
            return Err(FlagUpdateError::PageNotMapped);
        }

        let p2 = unsafe { &mut *(p2_ptr(page, self.recursive_index)) };

        if p2[page.p2_index()].is_unused() {
            return Err(FlagUpdateError::PageNotMapped);
        }

        p2[page.p2_index()].set_flags(flags | Flags::HUGE_PAGE);

        Ok(MapperFlush::new(page))
    }

    fn translate_page(&self, page: Page<Size2MiB>) -> Option<PhysFrame<Size2MiB>> {
        let p4 = &self.p4;

        if p4[page.p4_index()].is_unused() {
            return None;
        }

        let p3 = unsafe { &*(p3_ptr(page, self.recursive_index)) };
        let p3_entry = &p3[page.p3_index()];

        if p3_entry.is_unused() {
            return None;
        }

        let p2 = unsafe { &*(p2_ptr(page, self.recursive_index)) };
        let p2_entry = &p2[page.p2_index()];

        if p2_entry.is_unused() {
            return None;
        }

        PhysFrame::from_start_address(p2_entry.addr()).ok()
    }
}

impl<'a> Mapper<Size4KiB> for RecursivePageTable<'a> {
    fn map_to<A>(
        &mut self,
        page: Page<Size4KiB>,
        frame: PhysFrame<Size4KiB>,
        flags: PageTableFlags,
        mut allocator: A,
    ) -> Result<MapperFlush<Size4KiB>, MapToError>
    where
        A: FnMut() -> Option<PhysFrame>,
    {
        let p4 = &mut self.p4;

        let p3_page = p3_page(page, self.recursive_index);
        let p3 = Self::create_next_table(&mut p4[page.p4_index()], p3_page, &mut allocator)?;

        let p2_page = p2_page(page, self.recursive_index);
        let p2 = Self::create_next_table(&mut p3[page.p3_index()], p2_page, &mut allocator)?;

        let p1_page = p1_page(page, self.recursive_index);
        let p1 = Self::create_next_table(&mut p2[page.p2_index()], p1_page, &mut allocator)?;

        if !p1[page.p1_index()].is_unused() {
            return Err(MapToError::PageAlreadyMapped);
        }
        p1[page.p1_index()].set_frame(frame, flags);

        Ok(MapperFlush::new(page))
    }

    fn unmap<A>(
        &mut self,
        page: Page<Size4KiB>,
        allocator: A,
    ) -> Result<MapperFlush<Size4KiB>, UnmapError>
    where
        A: FnOnce(PhysFrame<Size4KiB>),
    {
        let p4 = &mut self.p4;
        let p4_entry = &p4[page.p4_index()];
        p4_entry.frame().map_err(|err| match err {
            FrameError::FrameNotPresent => UnmapError::PageNotMapped,
            FrameError::HugeFrame => UnmapError::ParentEntryHugePage,
        })?;

        let p3 = unsafe { &mut *(p3_ptr(page, self.recursive_index)) };
        let p3_entry = &p3[page.p3_index()];
        p3_entry.frame().map_err(|err| match err {
            FrameError::FrameNotPresent => UnmapError::PageNotMapped,
            FrameError::HugeFrame => UnmapError::ParentEntryHugePage,
        })?;

        let p2 = unsafe { &mut *(p2_ptr(page, self.recursive_index)) };
        let p2_entry = &p2[page.p2_index()];
        p2_entry.frame().map_err(|err| match err {
            FrameError::FrameNotPresent => UnmapError::PageNotMapped,
            FrameError::HugeFrame => UnmapError::ParentEntryHugePage,
        })?;

        let p1 = unsafe { &mut *(p1_ptr(page, self.recursive_index)) };
        let p1_entry = &mut p1[page.p1_index()];

        let frame = p1_entry.frame().map_err(|err| match err {
            FrameError::FrameNotPresent => UnmapError::PageNotMapped,
            FrameError::HugeFrame => UnmapError::ParentEntryHugePage,
        })?;
        allocator(frame);
        p1_entry.set_unused();
        Ok(MapperFlush::new(page))
    }

    fn update_flags(
        &mut self,
        page: Page<Size4KiB>,
        flags: PageTableFlags,
    ) -> Result<MapperFlush<Size4KiB>, FlagUpdateError> {
        let p4 = &mut self.p4;

        if p4[page.p4_index()].is_unused() {
            return Err(FlagUpdateError::PageNotMapped);
        }

        let p3 = unsafe { &mut *(p3_ptr(page, self.recursive_index)) };

        if p3[page.p3_index()].is_unused() {
            return Err(FlagUpdateError::PageNotMapped);
        }

        let p2 = unsafe { &mut *(p2_ptr(page, self.recursive_index)) };

        if p2[page.p2_index()].is_unused() {
            return Err(FlagUpdateError::PageNotMapped);
        }

        let p1 = unsafe { &mut *(p1_ptr(page, self.recursive_index)) };

        if p1[page.p1_index()].is_unused() {
            return Err(FlagUpdateError::PageNotMapped);
        }

        p1[page.p1_index()].set_flags(flags);

        Ok(MapperFlush::new(page))
    }

    fn translate_page(&self, page: Page<Size4KiB>) -> Option<PhysFrame<Size4KiB>> {
        let p4 = &self.p4;

        if p4[page.p4_index()].is_unused() {
            return None;
        }

        let p3 = unsafe { &*(p3_ptr(page, self.recursive_index)) };
        let p3_entry = &p3[page.p3_index()];

        if p3_entry.is_unused() {
            return None;
        }

        let p2 = unsafe { &*(p2_ptr(page, self.recursive_index)) };
        let p2_entry = &p2[page.p2_index()];

        if p2_entry.is_unused() {
            return None;
        }

        let p1 = unsafe { &*(p1_ptr(page, self.recursive_index)) };
        let p1_entry = &p1[page.p1_index()];

        if p1_entry.is_unused() {
            return None;
        }

        PhysFrame::from_start_address(p1_entry.addr()).ok()
    }
}

fn p3_ptr<S: PageSize>(page: Page<S>, recursive_index: u9) -> *mut PageTable {
    p3_page(page, recursive_index).start_address().as_mut_ptr()
}

fn p3_page<S: PageSize>(page: Page<S>, recursive_index: u9) -> Page {
    Page::from_page_table_indices(
        recursive_index,
        recursive_index,
        recursive_index,
        page.p4_index(),
    )
}

fn p2_ptr<S: NotGiantPageSize>(page: Page<S>, recursive_index: u9) -> *mut PageTable {
    p2_page(page, recursive_index).start_address().as_mut_ptr()
}

fn p2_page<S: NotGiantPageSize>(page: Page<S>, recursive_index: u9) -> Page {
    Page::from_page_table_indices(
        recursive_index,
        recursive_index,
        page.p4_index(),
        page.p3_index(),
    )
}

fn p1_ptr(page: Page<Size4KiB>, recursive_index: u9) -> *mut PageTable {
    p1_page(page, recursive_index).start_address().as_mut_ptr()
}

fn p1_page(page: Page<Size4KiB>, recursive_index: u9) -> Page {
    Page::from_page_table_indices(
        recursive_index,
        page.p4_index(),
        page.p3_index(),
        page.p2_index(),
    )
}
