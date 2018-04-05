use structures::paging::{Page, VirtAddr, PhysFrame};
use structures::paging::page_table::{PageTable, PageTableEntry, PageTableFlags};
use registers::control::Cr3;
use ux::u9;

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
    EntryWithInvalidFlagsPresent(Page, PageTableEntry),
    PageNotMapped(Page),
}

impl<'a> RecursivePageTable<'a> {
    pub fn new(table: &'a mut PageTable) -> Result<Self, NotRecursivelyMapped> {
        let page = Page::containing_address(VirtAddr::new(table as *const _ as u64));
        let recursive_index = page.p4_index();

        if page.p3_index() != recursive_index || page.p2_index() != recursive_index
            || page.p1_index() != recursive_index
        {
            return Err(NotRecursivelyMapped);
        }
        if Some(Cr3::read().0) != table[recursive_index].frame() {
            return Err(NotRecursivelyMapped);
        }

        Ok(RecursivePageTable { p4: table, recursive_index })
    }

    pub fn map_to<A>(&mut self, page: Page, frame: PhysFrame, flags: PageTableFlags,
            allocator: &mut A) -> Result<(), MapToError>
        where A: FnMut() -> Option<PhysFrame>
    {
        use structures::paging::PageTableFlags as Flags;
        let p4 = &mut self.p4;

        let mut create_next_table = |entry: &mut PageTableEntry| {
            if entry.is_unused() {
                if let Some(frame) = allocator() {
                    entry.set(frame, Flags::PRESENT | Flags::WRITABLE);
                    return Ok(true);
                } else {
                    return Err(MapToError::FrameAllocationFailed);
                }
            }
            if entry.flags().contains(Flags::HUGE_PAGE) {
                return Err(MapToError::EntryWithInvalidFlagsPresent);
            }
            Ok(false)
        };

        let p3_created = create_next_table(&mut p4[page.p4_index()])?;
        let p3 = unsafe { &mut *(p3_ptr(&page, self.recursive_index)) };
        if p3_created { p3.zero() }

        let p2_created = create_next_table(&mut p3[page.p3_index()])?;
        let p2 = unsafe { &mut *(p2_ptr(&page, self.recursive_index)) };
        if p2_created { p2.zero() }

        let p1_created = create_next_table(&mut p2[page.p2_index()])?;
        let p1 = unsafe { &mut *(p1_ptr(&page, self.recursive_index)) };
        if p1_created { p1.zero() }

        if !p1[page.p1_index()].is_unused() {
            return Err(MapToError::PageAlreadyInUse);
        }
        p1[page.p1_index()].set(frame, flags);

        Ok(())
    }

    pub fn identity_map<A>(&mut self, frame: PhysFrame, flags: PageTableFlags,
            allocator: &mut A) -> Result<(), MapToError>
        where A: FnMut() -> Option<PhysFrame>
    {
        let page = Page::containing_address(VirtAddr::new(frame.start_address().as_u64()));
        self.map_to(page, frame, flags, allocator)
    }

    pub fn unmap<A>(&mut self, page: Page, allocator: &mut A) -> Result<(), UnmapError>
        where A: FnMut(PhysFrame)
    {
        use structures::paging::PageTableFlags as Flags;

        let p4 = &mut self.p4;
        let p4_entry = &p4[page.p4_index()];
        if p4_entry.is_unused() { return Err(UnmapError::PageNotMapped(page)); }
        if p4_entry.flags().contains(Flags::HUGE_PAGE) {
            return Err(UnmapError::EntryWithInvalidFlagsPresent(page, p4_entry.clone()));
        }

        let p3 = unsafe { &mut *(p3_ptr(&page, self.recursive_index)) };
        let p3_entry = &p3[page.p3_index()];
        if p3_entry.is_unused() { return Err(UnmapError::PageNotMapped(page)); }
        if p3_entry.flags().contains(Flags::HUGE_PAGE) {
            return Err(UnmapError::EntryWithInvalidFlagsPresent(page, p3_entry.clone()));
        }

        let p2 = unsafe { &mut *(p2_ptr(&page, self.recursive_index)) };
        let p2_entry = &mut p2[page.p2_index()];
        if p2_entry.is_unused() { return Err(UnmapError::PageNotMapped(page)); }
        if p2_entry.flags().contains(Flags::PRESENT | Flags::HUGE_PAGE) {
            let frame = match p2_entry.frame() {
                Some(frame) => frame,
                None => return Err(UnmapError::EntryWithInvalidFlagsPresent(page, p2_entry.clone())),
            };
            allocator(frame);
            p2_entry.set_unused();
            return Ok(());
        }

        let p1 = unsafe { &mut *(p1_ptr(&page, self.recursive_index)) };
        let p1_entry = &mut p1[page.p1_index()];
        if p1_entry.is_unused() { return Err(UnmapError::PageNotMapped(page)); }
        if p1_entry.flags().contains(Flags::PRESENT) {
            let frame = match p1_entry.frame() {
                Some(frame) => frame,
                None => return Err(UnmapError::EntryWithInvalidFlagsPresent(page, p1_entry.clone())),
            };
            allocator(frame);
            p1_entry.set_unused();
            return Ok(());
        }
        Err(UnmapError::EntryWithInvalidFlagsPresent(page, p1_entry.clone()))
    }
}

fn p3_ptr(page: &Page, recursive_index: u9) -> *mut PageTable {
    Page::from_page_table_indices(
        recursive_index, recursive_index, recursive_index, page.p4_index()
    ).start_address().as_mut_ptr()
}

fn p2_ptr(page: &Page, recursive_index: u9) -> *mut PageTable {
    Page::from_page_table_indices(
        recursive_index, recursive_index, page.p4_index(), page.p3_index()
    ).start_address().as_mut_ptr()
}

fn p1_ptr(page: &Page, recursive_index: u9) -> *mut PageTable {
    Page::from_page_table_indices(
        recursive_index, page.p4_index(), page.p3_index(), page.p2_index()
    ).start_address().as_mut_ptr()
}
