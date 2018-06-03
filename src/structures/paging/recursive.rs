use instructions::tlb;
use registers::control::Cr3;
use structures::paging::page_table::{FrameError, PageTable, PageTableEntry, PageTableFlags};
use structures::paging::{NotGiantPageSize, Page, PageSize, PhysFrame, Size1GB, Size2MB, Size4KB};
use ux::u9;
use VirtAddr;

/// This type must be used and will either flush the modified page or can be unsafely ignored.
#[must_use = "Page Table changes must be flushed or unsafely ignored."]
pub struct MapperFlush<S: PageSize>(Page<S>);

impl<S: PageSize> MapperFlush<S> {
    /// Create a new flush promise
    fn new(page: Page<S>) -> Self {
        MapperFlush(page)
    }

    // Flush
    pub fn flush(self) {
        tlb::flush(self.0.start_address());
    }

    pub unsafe fn ignore(self) {}
}

pub trait Mapper<S: PageSize> {
    fn map_to<A>(
        &mut self,
        page: Page<S>,
        frame: PhysFrame<S>,
        flags: PageTableFlags,
        allocator: &mut A,
    ) -> Result<MapperFlush<S>, MapToError>
    where
        A: FnMut() -> Option<PhysFrame>;

    fn unmap<A>(&mut self, page: Page<S>, allocator: &mut A) -> Result<MapperFlush<S>, UnmapError>
    where
        A: FnMut(PhysFrame<S>);

    fn update_flags(
        &mut self,
        page: Page<S>,
        flags: PageTableFlags,
    ) -> Result<MapperFlush<S>, FlagUpdateError>;

    fn translate(&self, page: Page<S>) -> Option<PhysFrame<S>>;
}

pub struct RecursivePageTable<'a> {
    p4: &'a mut PageTable,
    recursive_index: u9,
}

#[derive(Debug)]
pub struct NotRecursivelyMapped;

#[derive(Debug)]
pub enum MapToError {
    FrameAllocationFailed,
    EntryWithInvalidFlagsPresent,
    PageAlreadyInUse,
}

#[derive(Debug)]
pub enum UnmapError {
    EntryWithInvalidFlagsPresent(PageTableFlags),
    PageNotMapped,
    InvalidFrameAddressInPageTable,
}

#[derive(Debug)]
pub enum FlagUpdateError {
    PageNotMapped,
}

impl<'a> RecursivePageTable<'a> {
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

    pub unsafe fn new_unchecked(table: &'a mut PageTable, recursive_index: u9) -> Self {
        RecursivePageTable {
            p4: table,
            recursive_index,
        }
    }

    pub fn identity_map<A, S>(
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

    fn create_next_table<A>(
        entry: &mut PageTableEntry,
        allocator: &mut A,
    ) -> Result<bool, MapToError>
    where
        A: FnMut() -> Option<PhysFrame>,
    {
        use structures::paging::PageTableFlags as Flags;

        if entry.is_unused() {
            if let Some(frame) = allocator() {
                entry.set_frame(frame, Flags::PRESENT | Flags::WRITABLE);
                return Ok(true);
            } else {
                return Err(MapToError::FrameAllocationFailed);
            }
        }
        if entry.flags().contains(Flags::HUGE_PAGE) {
            return Err(MapToError::EntryWithInvalidFlagsPresent);
        }
        Ok(false)
    }
}

impl<'a> Mapper<Size1GB> for RecursivePageTable<'a> {
    fn map_to<A>(
        &mut self,
        page: Page<Size1GB>,
        frame: PhysFrame<Size1GB>,
        flags: PageTableFlags,
        allocator: &mut A,
    ) -> Result<MapperFlush<Size1GB>, MapToError>
    where
        A: FnMut() -> Option<PhysFrame>,
    {
        use structures::paging::PageTableFlags as Flags;
        let p4 = &mut self.p4;

        let p3_created = Self::create_next_table(&mut p4[page.p4_index()], allocator)?;
        let p3 = unsafe { &mut *(p3_ptr(page, self.recursive_index)) };
        if p3_created {
            p3.zero()
        }

        if !p3[page.p3_index()].is_unused() {
            return Err(MapToError::PageAlreadyInUse);
        }
        p3[page.p3_index()].set_addr(frame.start_address(), flags | Flags::HUGE_PAGE);

        Ok(MapperFlush::new(page))
    }

    fn unmap<A>(
        &mut self,
        page: Page<Size1GB>,
        allocator: &mut A,
    ) -> Result<MapperFlush<Size1GB>, UnmapError>
    where
        A: FnMut(PhysFrame<Size1GB>),
    {
        let p4 = &mut self.p4;
        let p4_entry = &p4[page.p4_index()];

        p4_entry.frame().map_err(|err| match err {
            FrameError::FrameNotPresent => UnmapError::PageNotMapped,
            FrameError::HugeFrame => UnmapError::EntryWithInvalidFlagsPresent(p4_entry.flags()),
        })?;

        let p3 = unsafe { &mut *(p3_ptr(page, self.recursive_index)) };
        let p3_entry = &mut p3[page.p3_index()];
        let flags = p3_entry.flags();

        if !flags.contains(PageTableFlags::PRESENT) {
            return Err(UnmapError::PageNotMapped);
        }
        if !flags.contains(PageTableFlags::HUGE_PAGE) {
            return Err(UnmapError::EntryWithInvalidFlagsPresent(p3_entry.flags()));
        }

        let frame = PhysFrame::from_start_address(p3_entry.addr())
            .map_err(|()| UnmapError::InvalidFrameAddressInPageTable)?;
        allocator(frame);
        p3_entry.set_unused();
        Ok(MapperFlush::new(page))
    }

    fn update_flags(
        &mut self,
        page: Page<Size1GB>,
        flags: PageTableFlags,
    ) -> Result<MapperFlush<Size1GB>, FlagUpdateError> {
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

    fn translate(&self, page: Page<Size1GB>) -> Option<PhysFrame<Size1GB>> {
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

impl<'a> Mapper<Size2MB> for RecursivePageTable<'a> {
    fn map_to<A>(
        &mut self,
        page: Page<Size2MB>,
        frame: PhysFrame<Size2MB>,
        flags: PageTableFlags,
        allocator: &mut A,
    ) -> Result<MapperFlush<Size2MB>, MapToError>
    where
        A: FnMut() -> Option<PhysFrame>,
    {
        use structures::paging::PageTableFlags as Flags;
        let p4 = &mut self.p4;

        let p3_created = Self::create_next_table(&mut p4[page.p4_index()], allocator)?;
        let p3 = unsafe { &mut *(p3_ptr(page, self.recursive_index)) };
        if p3_created {
            p3.zero()
        }

        let p2_created = Self::create_next_table(&mut p3[page.p3_index()], allocator)?;
        let p2 = unsafe { &mut *(p2_ptr(page, self.recursive_index)) };
        if p2_created {
            p2.zero()
        }

        if !p2[page.p2_index()].is_unused() {
            return Err(MapToError::PageAlreadyInUse);
        }
        p2[page.p2_index()].set_addr(frame.start_address(), flags | Flags::HUGE_PAGE);

        Ok(MapperFlush::new(page))
    }

    fn unmap<A>(
        &mut self,
        page: Page<Size2MB>,
        allocator: &mut A,
    ) -> Result<MapperFlush<Size2MB>, UnmapError>
    where
        A: FnMut(PhysFrame<Size2MB>),
    {
        let p4 = &mut self.p4;
        let p4_entry = &p4[page.p4_index()];
        p4_entry.frame().map_err(|err| match err {
            FrameError::FrameNotPresent => UnmapError::PageNotMapped,
            FrameError::HugeFrame => UnmapError::EntryWithInvalidFlagsPresent(p4_entry.flags()),
        })?;

        let p3 = unsafe { &mut *(p3_ptr(page, self.recursive_index)) };
        let p3_entry = &p3[page.p3_index()];
        p3_entry.frame().map_err(|err| match err {
            FrameError::FrameNotPresent => UnmapError::PageNotMapped,
            FrameError::HugeFrame => UnmapError::EntryWithInvalidFlagsPresent(p3_entry.flags()),
        })?;

        let p2 = unsafe { &mut *(p2_ptr(page, self.recursive_index)) };
        let p2_entry = &mut p2[page.p2_index()];
        let flags = p2_entry.flags();

        if !flags.contains(PageTableFlags::PRESENT) {
            return Err(UnmapError::PageNotMapped);
        }
        if !flags.contains(PageTableFlags::HUGE_PAGE) {
            return Err(UnmapError::EntryWithInvalidFlagsPresent(p2_entry.flags()));
        }

        let frame = PhysFrame::from_start_address(p2_entry.addr())
            .map_err(|()| UnmapError::InvalidFrameAddressInPageTable)?;
        allocator(frame);
        p2_entry.set_unused();
        Ok(MapperFlush::new(page))
    }

    fn update_flags(
        &mut self,
        page: Page<Size2MB>,
        flags: PageTableFlags,
    ) -> Result<MapperFlush<Size2MB>, FlagUpdateError> {
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

    fn translate(&self, page: Page<Size2MB>) -> Option<PhysFrame<Size2MB>> {
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

impl<'a> Mapper<Size4KB> for RecursivePageTable<'a> {
    fn map_to<A>(
        &mut self,
        page: Page<Size4KB>,
        frame: PhysFrame<Size4KB>,
        flags: PageTableFlags,
        allocator: &mut A,
    ) -> Result<MapperFlush<Size4KB>, MapToError>
    where
        A: FnMut() -> Option<PhysFrame>,
    {
        let p4 = &mut self.p4;

        let p3_created = Self::create_next_table(&mut p4[page.p4_index()], allocator)?;
        let p3 = unsafe { &mut *(p3_ptr(page, self.recursive_index)) };
        if p3_created {
            p3.zero()
        }

        let p2_created = Self::create_next_table(&mut p3[page.p3_index()], allocator)?;
        let p2 = unsafe { &mut *(p2_ptr(page, self.recursive_index)) };
        if p2_created {
            p2.zero()
        }

        let p1_created = Self::create_next_table(&mut p2[page.p2_index()], allocator)?;
        let p1 = unsafe { &mut *(p1_ptr(page, self.recursive_index)) };
        if p1_created {
            p1.zero()
        }

        if !p1[page.p1_index()].is_unused() {
            return Err(MapToError::PageAlreadyInUse);
        }
        p1[page.p1_index()].set_frame(frame, flags);

        Ok(MapperFlush::new(page))
    }

    fn unmap<A>(
        &mut self,
        page: Page<Size4KB>,
        allocator: &mut A,
    ) -> Result<MapperFlush<Size4KB>, UnmapError>
    where
        A: FnMut(PhysFrame<Size4KB>),
    {
        let p4 = &mut self.p4;
        let p4_entry = &p4[page.p4_index()];
        p4_entry.frame().map_err(|err| match err {
            FrameError::FrameNotPresent => UnmapError::PageNotMapped,
            FrameError::HugeFrame => UnmapError::EntryWithInvalidFlagsPresent(p4_entry.flags()),
        })?;

        let p3 = unsafe { &mut *(p3_ptr(page, self.recursive_index)) };
        let p3_entry = &p3[page.p3_index()];
        p3_entry.frame().map_err(|err| match err {
            FrameError::FrameNotPresent => UnmapError::PageNotMapped,
            FrameError::HugeFrame => UnmapError::EntryWithInvalidFlagsPresent(p3_entry.flags()),
        })?;

        let p2 = unsafe { &mut *(p2_ptr(page, self.recursive_index)) };
        let p2_entry = &p2[page.p2_index()];
        p2_entry.frame().map_err(|err| match err {
            FrameError::FrameNotPresent => UnmapError::PageNotMapped,
            FrameError::HugeFrame => UnmapError::EntryWithInvalidFlagsPresent(p3_entry.flags()),
        })?;

        let p1 = unsafe { &mut *(p1_ptr(page, self.recursive_index)) };
        let p1_entry = &mut p1[page.p1_index()];

        let frame = p1_entry.frame().map_err(|err| match err {
            FrameError::FrameNotPresent => UnmapError::PageNotMapped,
            FrameError::HugeFrame => UnmapError::EntryWithInvalidFlagsPresent(p3_entry.flags()),
        })?;
        allocator(frame);
        p1_entry.set_unused();
        Ok(MapperFlush::new(page))
    }

    fn update_flags(
        &mut self,
        page: Page<Size4KB>,
        flags: PageTableFlags,
    ) -> Result<MapperFlush<Size4KB>, FlagUpdateError> {
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

    fn translate(&self, page: Page<Size4KB>) -> Option<PhysFrame<Size4KB>> {
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
    Page::from_page_table_indices(
        recursive_index,
        recursive_index,
        recursive_index,
        page.p4_index(),
    ).start_address()
        .as_mut_ptr()
}

fn p2_ptr<S: NotGiantPageSize>(page: Page<S>, recursive_index: u9) -> *mut PageTable {
    Page::from_page_table_indices(
        recursive_index,
        recursive_index,
        page.p4_index(),
        page.p3_index(),
    ).start_address()
        .as_mut_ptr()
}

fn p1_ptr(page: Page<Size4KB>, recursive_index: u9) -> *mut PageTable {
    Page::from_page_table_indices(
        recursive_index,
        page.p4_index(),
        page.p3_index(),
        page.p2_index(),
    ).start_address()
        .as_mut_ptr()
}
