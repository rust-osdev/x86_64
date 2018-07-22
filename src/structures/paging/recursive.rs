#![cfg(target_pointer_width = "64")]

use core::ops::RangeBounds;
use instructions::tlb;
use registers::control::Cr3;
use structures::paging::{
    frame_alloc::{FrameAllocator, FrameDeallocator},
    page_table::{FrameError, PageTable, PageTableEntry, PageTableFlags},
    NotGiantPageSize, Page, PageSize, PhysFrame, Size1GiB, Size2MiB, Size4KiB,
};
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

/// The mode for `reclaim_page_tables`. This determines the behavior when we come across a page
/// table entry that is used (i.e. not 0).
pub enum ReclaimPageTablesMode {
    /// Only reclaim page tables whose entries are all empty. Skip non-empty tables.
    Skip,

    /// Panic if there is a non-empty page table entry mapping a virtual address in the range.
    Panic,
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
        allocator: &mut A,
    ) -> Result<MapperFlush<S>, MapToError>
    where
        A: FrameAllocator<Size4KiB>;

    /// Removes a mapping from the page table and returns the frame that used to be mapped.
    ///
    /// Note that no page tables or other frames are deallocated.
    fn unmap(&mut self, page: Page<S>) -> Result<(PhysFrame<S>, MapperFlush<S>), UnmapError>;

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
        A: FrameAllocator<Size4KiB>,
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

/// A trait for types that walk recursive page tables. Each method can be implemented optionally to
/// do something with the parts of the page table it takes. The default implementations are noops.
pub trait RecursivePageTableVisitor {
    /// Visit this recursive page table.
    fn visit(&mut self, page_tables: &mut RecursivePageTable) {
        let p4 = &mut page_tables.p4;
        self.visit_p4(page_tables.recursive_index, p4);
    }

    /// Visit the given `p4` page table.
    fn visit_p4(&mut self, recursive_index: u9, p4: &mut PageTable) {
        let max_idx: u16 = u9::MAX.into();
        for entry_idx in 0..=max_idx {
            let entry_idx = u9::new(entry_idx);
            let entry = &mut p4[entry_idx];

            if !self.before_visit_p4_entry(entry_idx, entry) {
                continue;
            }

            // Check if the frame is present.
            match entry.frame() {
                // No entry... skip.
                Err(FrameError::FrameNotPresent) => continue,
                // Cannot have 512GiB huge pages (yet)!
                Err(FrameError::HugeFrame) => unreachable!(),
                Ok(_) => {}
            }

            let p3 = unsafe {
                &mut *Page::from_page_table_indices(
                    recursive_index,
                    recursive_index,
                    recursive_index,
                    entry_idx,
                ).start_address()
                    .as_mut_ptr()
            };
            self.visit_p3(recursive_index, entry_idx, p3);
            self.after_visit_p3(entry_idx, entry);
        }
    }

    // TODO: better doc comments
    /// If false, skip processing this p4 entry. Take the entry and index in P4 we are about to
    /// visit.
    fn before_visit_p4_entry(&mut self, _entry_idx: u9, _entry: &mut PageTableEntry) -> bool {
        true
    }

    /// Visit the given `p3` page table. `p3_pte` is a reference to the entry in `p4` that points
    /// to this `p3`.
    fn visit_p3(&mut self, recursive_index: u9, p4_idx: u9, p3: &mut PageTable) {
        let max_idx: u16 = u9::MAX.into();
        for entry_idx in 0..=max_idx {
            let entry_idx = u9::new(entry_idx);
            let entry = &mut p3[entry_idx];

            if !self.before_visit_p3_entry(p4_idx, entry_idx, entry) {
                continue;
            }

            // Check if the frame is present.
            match entry.frame() {
                // No entry... skip.
                Err(FrameError::FrameNotPresent) => continue,
                // This is a 1GiB page. Visit it; then exit.
                Err(FrameError::HugeFrame) => {
                    if !self.before_visit_1gib_page(p4_idx, entry_idx, entry) {
                        continue;
                    }

                    let frame = unsafe {
                        &mut *Page::from_page_table_indices_1gib(p4_idx, entry_idx)
                            .start_address()
                            .as_mut_ptr()
                    };
                    self.visit_1gib_page(p4_idx, entry_idx, frame);
                    self.after_visit_1gib_page(p4_idx, entry_idx, entry);
                    return;
                }
                Ok(_) => {}
            }

            let p2 = unsafe {
                &mut *Page::from_page_table_indices(
                    recursive_index,
                    recursive_index,
                    p4_idx,
                    entry_idx,
                ).start_address()
                    .as_mut_ptr()
            };

            self.visit_p2(recursive_index, p4_idx, entry_idx, p2);
            self.after_visit_p2(p4_idx, entry_idx, entry);
        }
    }

    /// Takes index and entry in P4 of the P3 we just visited
    fn after_visit_p3(&mut self, p4_idx: u9, _p4_pte: &mut PageTableEntry) {}

    /// Takes index into p3 and p4, entry in p3 of the p2/huge page we are about to visit.
    /// False if we should skip.
    fn before_visit_p3_entry(
        &mut self,
        p4_idx: u9,
        p3_idx: u9,
        entry: &mut PageTableEntry,
    ) -> bool {
        true
    }

    /// Takes index into p3 and p4, entry in p3 of the huge page we are about to visit.
    /// False if we should skip.
    fn before_visit_1gib_page(
        &mut self,
        p4_idx: u9,
        p3_idx: u9,
        entry: &mut PageTableEntry,
    ) -> bool {
        true
    }

    /// Takes index into p3 and p4 and the page itself.
    fn visit_1gib_page(&mut self, p4_idx: u9, p3_idx: u9, _page: &Page<Size1GiB>) {}

    /// Takes index into p3 and p4, and P3 entry for huge page we just visited
    fn after_visit_1gib_page(&mut self, p4_idx: u9, p3_idx: u9, entry: &mut PageTableEntry) {}

    /// Visit the given `p2` page table, pointed to by `p2_pte` in a p3 page table.
    fn visit_p2(&mut self, recursive_index: u9, p4_idx: u9, p3_idx: u9, p2: &mut PageTable) {
        let max_idx: u16 = u9::MAX.into();
        for entry_idx in 0..=max_idx {
            let entry_idx = u9::new(entry_idx);
            let entry = &mut p2[entry_idx];

            if !self.before_visit_p2_entry(p4_idx, p3_idx, entry_idx, entry) {
                continue;
            }

            // Check if the frame is present.
            match entry.frame() {
                // No entry... skip.
                Err(FrameError::FrameNotPresent) => continue,
                // This is a 2MiB page. Visit it; then exit.
                Err(FrameError::HugeFrame) => {
                    if !self.before_visit_2mib_page(p4_idx, p3_idx, entry_idx, entry) {
                        continue;
                    }

                    let frame = unsafe {
                        &mut *Page::from_page_table_indices_2mib(p4_idx, p3_idx, entry_idx)
                            .start_address()
                            .as_mut_ptr()
                    };
                    self.visit_2mib_page(p4_idx, p3_idx, entry_idx, frame);
                    self.after_visit_2mib_page(p4_idx, p3_idx, entry_idx, entry);
                    return;
                }
                Ok(_) => {}
            }

            let p1 = unsafe {
                &mut *Page::from_page_table_indices(recursive_index, p4_idx, p3_idx, entry_idx)
                    .start_address()
                    .as_mut_ptr()
            };
            self.visit_p1(recursive_index, p4_idx, p3_idx, entry_idx, p1);
            self.after_visit_p1(p4_idx, p3_idx, entry_idx, entry);
        }
    }

    /// Takes index into p3 and p4, and P3 entry for P2 we just visited
    fn after_visit_p2(&mut self, p4_idx: u9, p3_idx: u9, entry: &mut PageTableEntry) {}

    fn before_visit_p2_entry(
        &mut self,
        p4_idx: u9,
        p3_idx: u9,
        p2_idx: u9,
        entry: &mut PageTableEntry,
    ) -> bool {
        true
    }

    fn before_visit_2mib_page(
        &mut self,
        p4_idx: u9,
        p3_idx: u9,
        p2_idx: u9,
        entry: &mut PageTableEntry,
    ) -> bool {
        true
    }

    /// Visit the given 2MiB `page` pointed to by a `page_pte` in a p2 page table.
    fn visit_2mib_page(&mut self, p4_idx: u9, p3_idx: u9, p2_idx: u9, _page: &Page<Size2MiB>) {}

    fn after_visit_2mib_page(
        &mut self,
        p4_idx: u9,
        p3_idx: u9,
        p2_idx: u9,
        entry: &mut PageTableEntry,
    ) {
    }

    /// Visit the given `p1` page table, pointed to by `p1_pte` in a p2 page table.
    fn visit_p1(
        &mut self,
        _recursive_index: u9,
        p4_idx: u9,
        p3_idx: u9,
        p2_idx: u9,
        p1: &mut PageTable,
    ) {
        let max_idx: u16 = u9::MAX.into();
        for entry_idx in 0..=max_idx {
            let entry_idx = u9::new(entry_idx);
            let entry = &mut p1[entry_idx];

            if !self.before_visit_4kib_page(p4_idx, p3_idx, p2_idx, entry_idx, entry) {
                continue;
            }

            // Check if the frame is present.
            match entry.frame() {
                // No entry... skip.
                Err(FrameError::FrameNotPresent) => continue,
                // We are already at 4KiB. No huge pages here.
                Err(FrameError::HugeFrame) => unreachable!(),
                Ok(_) => {}
            }

            let page = unsafe {
                &mut *Page::from_page_table_indices(p4_idx, p3_idx, p2_idx, entry_idx)
                    .start_address()
                    .as_mut_ptr()
            };
            self.visit_4kib_page(p4_idx, p3_idx, p2_idx, entry_idx, page);
            self.after_visit_4kib_page(p4_idx, p3_idx, p2_idx, entry_idx, entry);
        }
    }

    fn before_visit_4kib_page(
        &mut self,
        p4_idx: u9,
        p3_idx: u9,
        p2_idx: u9,
        p1_idx: u9,
        entry: &mut PageTableEntry,
    ) -> bool {
        true
    }

    /// Visit the given 4KiB `page`, pointed to by `page_pte` in a p1 page table.
    fn visit_4kib_page(
        &mut self,
        p4_idx: u9,
        p3_idx: u9,
        p2_idx: u9,
        p1_idx: u9,
        _page: &Page<Size4KiB>,
    ) {
    }

    fn after_visit_4kib_page(
        &mut self,
        p4_idx: u9,
        p3_idx: u9,
        p2_idx: u9,
        p1_idx: u9,
        entry: &mut PageTableEntry,
    ) {
    }

    fn after_visit_p1(&mut self, p4_idx: u9, p3_idx: u9, p2_idx: u9, entry: &mut PageTableEntry) {}
}

/// Page table visitor that visits the page tables mapping a certain range of memory and
/// deallocates them.
pub struct DeletePageTablesVisitor<'d, R, D>
where
    D: FrameDeallocator<Size4KiB> + 'd,
    R: RangeBounds<Page<Size4KiB>>,
{
    /// The range to visit.
    range: R,
    /// The deallocator to deallocate frames to.
    deallocator: &'d mut D,
    /// The mode of operation: what to do when we encounter a non-empty page/page table?
    mode: ReclaimPageTablesMode,
}

impl<'d, R, D> DeletePageTablesVisitor<'d, R, D>
where
    D: FrameDeallocator<Size4KiB> + 'd,
    R: RangeBounds<Page<Size4KiB>>,
{
    /// Create a new `DeletePageTablesVisitor` over the given the `range`. The `deallocator` is
    /// used to free page tables. `mode` is used to determines the behavior when we come across a
    /// page table entry that is used (i.e. not 0). See the docs for `ReclaimPageTablesMode`.
    pub fn new(range: R, deallocator: &'d mut D, mode: ReclaimPageTablesMode) -> Self {
        DeletePageTablesVisitor {
            range,
            deallocator,
            mode,
        }
    }
}

impl<'d, R, D> RecursivePageTableVisitor for DeletePageTablesVisitor<'d, R, D>
where
    D: FrameDeallocator<Size4KiB> + 'd,
    R: RangeBounds<Page<Size4KiB>>,
{
    fn after_visit_p3(&mut self, p4_idx: u9, _p4_pte: &mut PageTableEntry) {
        if partially_out_of_range(p3, self.range) {
            return;
        } else {
            // TODO: if all freed, then free this P3
        }
    }

    fn before_visit_1gib_page(
        &mut self,
        p4_idx: u9,
        p3_idx: u9,
        entry: &mut PageTableEntry,
    ) -> bool {
        // TODO: skip if out of range
        false
    }

    fn visit_1gib_page(&mut self, p4_idx: u9, p3_idx: u9, _page: &Page<Size1GiB>) {
        // TODO: check mode
    }

    fn after_visit_p2(&mut self, p4_idx: u9, p3_idx: u9, entry: &mut PageTableEntry) {
        if partially_out_of_range(p2, self.range) {
            return;
        } else {
            // TODO: if all freed, then free this P2
        }
    }

    fn before_visit_2mib_page(
        &mut self,
        p4_idx: u9,
        p3_idx: u9,
        p2_idx: u9,
        entry: &mut PageTableEntry,
    ) -> bool {
        // TODO: skip if out of range
        false
    }

    fn visit_2mib_page(&mut self, p4_idx: u9, p3_idx: u9, p2_idx: u9, _page: &Page<Size2MiB>) {
        // TODO: check mode
    }

    fn after_visit_p1(&mut self, p4_idx: u9, p3_idx: u9, p2_idx: u9, entry: &mut PageTableEntry) {
        if partially_out_of_range(p1, self.range) {
            return;
        } else {
            // TODO: if all freed, then free this P1
        }
    }

    fn before_visit_4kib_page(
        &mut self,
        p4_idx: u9,
        p3_idx: u9,
        p2_idx: u9,
        p1_idx: u9,
        entry: &mut PageTableEntry,
    ) -> bool {
        // TODO: skip if out of range
        false
    }

    fn visit_4kib_page(
        &mut self,
        p4_idx: u9,
        p3_idx: u9,
        p2_idx: u9,
        p1_idx: u9,
        _page: &Page<Size4KiB>,
    ) {
        // TODO: check mode
    }
}

/*
impl<'d, R, D> RecursivePageTableVisitor for DeletePageTablesVisitor<'d, R, D>
where
    D: FrameDeallocator<Size4KiB> + 'd,
    R: RangeBounds<Page<Size4KiB>>,
{
    fn visit_p3(
        &mut self,
        recursive_index: u9,
        p4_idx: u9,
        _p3_pte: &mut PageTableEntry,
        p3: &mut PageTable,
    ) {
        let max_idx: u16 = u9::MAX.into();
        for entry_idx in 0..=max_idx {
            let entry_idx = u9::new(entry_idx);
            let entry = &mut p3[entry_idx];

            // Check if the frame is present.
            match entry.frame() {
                // No entry... this is fine.
                Err(FrameError::FrameNotPresent) => continue,
                // This is a 1GiB page. Check the mode to see what we should do.
                Err(FrameError::HugeFrame) => if entry_in_range(page(entry), self.range) {
                    // TODO
                    match self.mode {
                        ReclaimPageTablesMode::Skip => return,
                        ReclaimPageTablesMode::Panic => {
                            panic!("Attempt to reclaim a non-empty p3 page table");
                        }
                    }
                } else {
                    // skip it.
                    continue;
                },
                Ok(_) => {}
            }

            let p2 = unsafe {
                &mut *Page::from_page_table_indices(
                    recursive_index,
                    recursive_index,
                    p4_idx,
                    entry_idx,
                ).start_address()
                    .as_mut_ptr()
            };
            self.visit_p2(recursive_index, p4_idx, entry_idx, entry, p2);
        }

        if partially_out_of_range(p3, self.range) {
            return;
        } else {
            // TODO: if all freed, then free this P3
        }
    }

    fn visit_p2(
        &mut self,
        recursive_index: u9,
        p4_idx: u9,
        p3_idx: u9,
        _p2_pte: &mut PageTableEntry,
        p2: &mut PageTable,
    ) {
        let max_idx: u16 = u9::MAX.into();
        for entry_idx in 0..=max_idx {
            let entry_idx = u9::new(entry_idx);
            let entry = &mut p2[entry_idx];

            // Check if the frame is present.
            match entry.frame() {
                // No entry... skip.
                Err(FrameError::FrameNotPresent) => continue,
                // This is a 2MiB page. Visit it; then exit.
                Err(FrameError::HugeFrame) => if entry_in_range(entry, self.range) {
                    // TODO
                    match self.mode {
                        ReclaimPageTablesMode::Skip => return,
                        ReclaimPageTablesMode::Panic => {
                            panic!("Attempt to reclaim a non-empty p2 page table");
                        }
                    }
                } else {
                    // skip it.
                    continue;
                },
                Ok(_) => {}
            }

            let p1 = unsafe {
                &mut *Page::from_page_table_indices(recursive_index, p4_idx, p3_idx, entry_idx)
                    .start_address()
                    .as_mut_ptr()
            };
            self.visit_p1(recursive_index, p4_idx, p3_idx, entry_idx, entry, p1);
        }

        if partially_out_of_range(p2, self.range) {
            return;
        }

        // TODO: if all freed, then free this P2
    }

    fn visit_p1(
        &mut self,
        _recursive_index: u9,
        p4_idx: u9,
        p3_idx: u9,
        p2_idx: u9,
        _p1_pte: &mut PageTableEntry,
        p1: &mut PageTable,
    ) {
        let max_idx: u16 = u9::MAX.into();
        for entry_idx in 0..=max_idx {
            let entry_idx = u9::new(entry_idx);
            let entry = &mut p1[entry_idx];

            // Check if the frame is present.
            match entry.frame() {
                // No entry... skip.
                Err(FrameError::FrameNotPresent) => continue,
                // We are already at 4KiB. No huge pages here.
                Err(FrameError::HugeFrame) => unreachable!(),
                Ok(_) => {}
            }

            // TODO: should this be below the range check?
            match self.mode {
                ReclaimPageTablesMode::Skip => return,
                ReclaimPageTablesMode::Panic => {
                    panic!("Attempt to reclaim a non-empty p1 page table");
                }
            }
        }

        if partially_out_of_range(p1, self.range) {
            return;
        }

        // TODO: free this P1
    }
}
*/

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

    /// Internal helper function to create the page table of the next level if needed.
    ///
    /// If the passed entry is unused, a new frame is allocated from the given allocator, zeroed,
    /// and the entry is updated to that address. If the passed entry is already mapped, the next
    /// table is returned directly.
    ///
    /// The `next_page_table` page must be the page of the next page table in the hierarchy.
    ///
    /// Returns `MapToError::FrameAllocationFailed` if the entry is unused and the allocator
    /// returned `None`. Returns `MapToError::ParentEntryHugePage` if the `HUGE_PAGE` flag is set
    /// in the passed entry.
    unsafe fn create_next_table<'b, A>(
        entry: &'b mut PageTableEntry,
        next_table_page: Page,
        allocator: &mut A,
    ) -> Result<&'b mut PageTable, MapToError>
    where
        A: FrameAllocator<Size4KiB>,
    {
        /// This inner function is used to limit the scope of `unsafe`.
        ///
        /// This is a safe function, so we need to use `unsafe` blocks when we do something unsafe.
        fn inner<'b, A>(
            entry: &'b mut PageTableEntry,
            next_table_page: Page,
            allocator: &mut A,
        ) -> Result<&'b mut PageTable, MapToError>
        where
            A: FrameAllocator<Size4KiB>,
        {
            use structures::paging::PageTableFlags as Flags;

            let created;

            if entry.is_unused() {
                if let Some(frame) = allocator.alloc() {
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

        inner(entry, next_table_page, allocator)
    }

    /// Reclaim page tables for virtual addresses corresponding to the given range of pages.
    /// The page tables are returned to the given `deallocator`. We reclaim page tables according
    /// to `mode`, which describes what behvior to take when we encounter a page table entirely in
    /// the given range with a used (i.e. non-zero) page table entry. See the docs for
    /// `ReclaimPageTablesMode` for more info on modes.
    ///
    /// Note that it is the caller's responsibility to make sure the page tables are not in use.
    /// Don't forget to check shared mappings!
    ///
    /// It's also worth noting that the p4 table currently in use can never be reclaimed by this
    /// function, since it is being used. Thus, if you need to reclaim a p4 table, you must do so
    /// from another address space.
    ///
    /// We only try to reclaim page tables whose mappings would all lie entirely within the range.
    ///
    /// # Panics
    ///
    /// - If one of the range enpoints is `Unbounded`.
    /// - If the `mode` specifies that we should.
    pub unsafe fn reclaim_page_tables<D, R>(
        &mut self,
        range: R,
        deallocator: &mut D,
        mode: ReclaimPageTablesMode,
    ) where
        D: FrameDeallocator<Size4KiB>,
        R: RangeBounds<Page<Size4KiB>>,
    {
        DeletePageTablesVisitor::new(range, deallocator, mode).visit(self);

        /*
        // Reclaim tables starting from the leaves (P1) and moving towards the root (P4).
        self.reclaim_p1_page_tables(range);
        self.reclaim_p2_page_tables(range);
        self.reclaim_p3_page_tables(range);
        */
    }

    // /// Reclaims all P1 page tables whose mappings are entirely contained inside the given range of
    // /// pages.
    // unsafe fn reclaim_p1_page_tables<D, R>(
    //     &mut self,
    //     range: R,
    //     deallocator: &mut D,
    //     mode: ReclaimPageTablesMode,
    // ) where
    //     D: FrameDeallocator<Size4KiB>,
    //     R: RangeBounds<Page<Size4KiB>>,
    // {
    //     // We can only reclaim a page table if all of its possible children are in the given range,
    //     // so round `start` and `end` to exclude page tables that are only partial in range.
    //     //
    //     // That is, we round `start` up to the nearest 512 * Size4KiB = 2MiB boundary, and we round
    //     // `end` down to the nearest boundary.

    //     let p4 = &mut self.p4;

    //     for page_1gib in align_outward_inclusive_1gib(range) {
    //         let p4_entry = &mut p4[page_1gib.p4_index()];

    //         match p4_entry.frame() {
    //             // If this is a huge page or not mapped, skip it. No P1 page tables to reclaim.
    //             Err(FrameError::FrameNotPresent) | Err(FrameError::HugeFrame) => {
    //                 continue;
    //             }
    //             _ => {}
    //         };

    //         for page_2mib in align_outward_inclusive_2mib(range) {
    //             let p3 = &mut *(p3_ptr(page_2mib, self.recursive_index));
    //             let p3_entry = &mut p3[page_2mib.p3_index()];

    //             match p3_entry.frame() {
    //                 // If this is a huge page or not mapped, skip it. No P1 page tables to reclaim.
    //                 Err(FrameError::FrameNotPresent) | Err(FrameError::HugeFrame) => {
    //                     continue;
    //                 }
    //                 _ => {}
    //             };

    //             for page_4kib in align_outward_inclusive_4kib(range) {
    //                 let p2 = &mut *(p2_ptr(page_4kib, self.recursive_index));
    //                 let p2_entry = &mut p2[page_4kib.p2_index()];

    //                 let p1_frame = match p2_entry.frame() {
    //                     // If this is a huge page or not mapped, skip it. No P1 page tables to reclaim.
    //                     Err(FrameError::FrameNotPresent) | Err(FrameError::HugeFrame) => {
    //                         continue;
    //                     }
    //                     Ok(frame) => frame,
    //                 };

    //                 // Depending on mode, we behave differently
    //                 let p1 = &mut *(p1_ptr(page, self.recursive_index));
    //                 match mode {
    //                     ReclaimPageTablesMode::Skip => {
    //                         // Skip non-empty page tables.
    //                         if p1.iter().any(|pte| !pte.is_unused()) {
    //                             continue;
    //                         }
    //                     }
    //                     ReclaimPageTablesMode::Panic => {
    //                         // Don't allow non-empty page tables.
    //                         assert!(p1.iter().all(|pte| pte.is_unused()));
    //                     }
    //                 }

    //                 p2_entry.set_unused();
    //                 deallocator.dealloc(p1_frame);
    //             }
    //         }
    //     }
    // }

    // /// Reclaims all P1 page tables whose mappings are entirely contained inside the given range of
    // /// pages.
    // unsafe fn reclaim_p1_page_tables<D, R>(
    //     &mut self,
    //     range: R,
    //     deallocator: &mut D,
    //     mode: ReclaimPageTablesMode,
    // ) where
    //     D: FrameDeallocator<Size4KiB>,
    //     R: RangeBounds<Page<Size4KiB>>,
    // {
    //     // Get the page ranges.
    //     let start = match range.start_bound() {
    //         Bound::Included(endpoint) => *endpoint,
    //         Bound::Excluded(endpoint) => if endpoint == last {
    //             panic!("Starting after end of address space!");
    //         } else {
    //             *endpoint + 1
    //         },
    //         Bound::Unbounded => {
    //             // TODO: maybe allow this?
    //             panic!("Attempt to reclaim page tables for unbounded region of memory.")
    //         }
    //     };
    //     // TODO: switch everything else to handle inclusive endpoints.
    //     let end = match range.end_bound() {
    //         Bound::Included(endpoint) => *endpoint,
    //         Bound::Excluded(endpoint) => if endpoint == 0 {
    //             panic!("Ending before start of address space!");
    //         } else {
    //             *endpoint - 1
    //         },
    //         Bound::Unbounded => {
    //             // TODO: maybe allow this?
    //             panic!("Attempt to reclaim page tables for unbounded region of memory.")
    //         }
    //     };

    //     let p4 = &mut self.p4;

    //     // Starting at the leaf page tables (p1), we reclaim page tables. Then we do p2 and p3.

    //     // We can only reclaim a page table if all of its possible children are in the given range,
    //     // so round `start` and `end` to exclude page tables that are only partial in range.
    //     //
    //     // That is, we round `start` up to the nearest 512 * Size4KiB boundary, and we round `end`
    //     // down to the nearest boundary.

    //     let p1_start = if start.p1_index() == u9::new(0) {
    //         start
    //     } else {
    //         Page::from_page_table_indices(
    //             start.p4_index(),
    //             start.p3_index(),
    //             start.p2_index() + u9::new(1),
    //             u9::new(0),
    //         )
    //     };

    //     let p1_end = Page::from_page_table_indices(
    //         //TODO: make this inclusive
    //         end.p4_index(),
    //         end.p3_index(),
    //         end.p2_index(),
    //         u9::new(0),
    //     );

    //     // Free all the page tables!
    //     for page in p1_start..=p1_end {
    //         let p4_entry = &mut p4[page.p4_index()];

    //         match p4_entry.frame() {
    //             Err(FrameError::FrameNotPresent) => {
    //                 continue;
    //             }
    //             Err(FrameError::HugeFrame) => unreachable!(),
    //             _ => {}
    //         };

    //         let p3 = &mut *(p3_ptr(page, self.recursive_index));
    //         let p3_entry = &mut p3[page.p3_index()];

    //         match p3_entry.frame() {
    //             Err(FrameError::FrameNotPresent) => {
    //                 continue;
    //             }
    //             Err(FrameError::HugeFrame) => unreachable!(),
    //             _ => {}
    //         };

    //         let p2 = &mut *(p2_ptr(page, self.recursive_index));
    //         let p2_entry = &mut p2[page.p2_index()];

    //         let p1_frame = match p2_entry.frame() {
    //             Ok(frame) => frame,
    //             Err(FrameError::FrameNotPresent) => {
    //                 continue;
    //             }
    //             Err(FrameError::HugeFrame) => unreachable!(),
    //         };

    //         // Depending on mode, we behave differently
    //         let p1 = &mut *(p1_ptr(page, self.recursive_index));
    //         match mode {
    //             ReclaimPageTablesMode::Skip => {
    //                 // Skip non-empty page tables.
    //                 if p1.iter().any(|pte| !pte.is_unused()) {
    //                     continue;
    //                 }
    //             }
    //             ReclaimPageTablesMode::Panic => {
    //                 // Don't allow non-empty page tables.
    //                 assert!(p1.iter().all(|pte| pte.is_unused()));
    //             }
    //         }

    //         p2_entry.set_unused();
    //         deallocator.dealloc(p1_frame);
    //     }
    // }

    // /// Reclaims all P2 page tables whose mappings are entirely contained inside the given range of
    // /// pages.
    // unsafe fn reclaim_p2_page_tables<D, R>(
    //     &mut self,
    //     range: R,
    //     deallocator: &mut D,
    //     mode: ReclaimPageTablesMode,
    // ) where
    //     D: FrameDeallocator<Size4KiB>,
    //     R: RangeBounds<Page<Size4KiB>>,
    // {
    //     let p4 = &mut self.p4;

    //     let p2_start = if start.p2_index() == u9::new(0) {
    //         start
    //     } else {
    //         Page::from_page_table_indices(
    //             start.p4_index(),
    //             start.p3_index() + u9::new(1),
    //             u9::new(0),
    //             u9::new(0),
    //         )
    //     };

    //     let p2_end =
    //         Page::from_page_table_indices(end.p4_index(), end.p3_index(), u9::new(0), u9::new(0));

    //     // Free all the page tables!
    //     for page in p2_start..p2_end {
    //         let p4_entry = &mut p4[page.p4_index()];

    //         match p4_entry.frame() {
    //             Err(FrameError::FrameNotPresent) => {
    //                 continue;
    //             }
    //             Err(FrameError::HugeFrame) => unreachable!(),
    //             _ => {}
    //         };

    //         let p3 = &mut *(p3_ptr(page, self.recursive_index));
    //         let p3_entry = &mut p3[page.p3_index()];

    //         let p2_frame = match p3_entry.frame() {
    //             Ok(frame) => frame,
    //             Err(FrameError::FrameNotPresent) => {
    //                 continue;
    //             }
    //             Err(FrameError::HugeFrame) => unreachable!(),
    //         };

    //         // Depending on mode, we behave differently
    //         let p2 = &mut *(p2_ptr(page, self.recursive_index));
    //         match mode {
    //             ReclaimPageTablesMode::Skip => {
    //                 // Skip non-empty page tables.
    //                 if p2.iter().any(|pte| !pte.is_unused()) {
    //                     continue;
    //                 }
    //             }
    //             ReclaimPageTablesMode::Panic => {
    //                 // Don't allow non-empty page tables.
    //                 assert!(p2.iter().all(|pte| pte.is_unused()));
    //             }
    //         }

    //         p3_entry.set_unused();
    //         deallocator.dealloc(p2_frame);
    //     }
    // }

    // /// Reclaims all P3 page tables whose mappings are entirely contained inside the given range of
    // /// pages.
    // unsafe fn reclaim_p3_page_tables<D, R>(
    //     &mut self,
    //     range: R,
    //     deallocator: &mut D,
    //     mode: ReclaimPageTablesMode,
    // ) where
    //     D: FrameDeallocator<Size4KiB>,
    //     R: RangeBounds<Page<Size4KiB>>,
    // {
    //     // Get the page ranges.
    //     let start = match range.start_bound() {
    //         Bound::Included(endpoint) => *endpoint,
    //         Bound::Excluded(endpoint) => *endpoint + 1,
    //         Bound::Unbounded => {
    //             // TODO: maybe allow this?
    //             panic!("Attempt to reclaim page tables for unbounded region of memory.")
    //         }
    //     };
    //     let end = match range.end_bound() {
    //         Bound::Included(endpoint) => *endpoint + 1,
    //         Bound::Excluded(endpoint) => *endpoint,
    //         Bound::Unbounded => {
    //             // TODO: maybe allow this?
    //             panic!("Attempt to reclaim page tables for unbounded region of memory.")
    //         }
    //     };

    //     let p4 = &mut self.p4;

    //     let p3_start = if start.p3_index() == u9::new(0) {
    //         start
    //     } else {
    //         Page::from_page_table_indices(
    //             start.p4_index() + u9::new(1),
    //             u9::new(0),
    //             u9::new(0),
    //             u9::new(0),
    //         )
    //     };

    //     let p3_end =
    //         Page::from_page_table_indices(end.p4_index(), u9::new(0), u9::new(0), u9::new(0));

    //     // Free all the page tables!
    //     for page in p3_start..p3_end {
    //         let p4_entry = &mut p4[page.p4_index()];

    //         let p3_frame = match p4_entry.frame() {
    //             Ok(frame) => frame,
    //             Err(FrameError::FrameNotPresent) => {
    //                 continue;
    //             }
    //             Err(FrameError::HugeFrame) => unreachable!(),
    //         };

    //         // Depending on mode, we behave differently
    //         let p3 = &mut *(p3_ptr(page, self.recursive_index));
    //         match mode {
    //             ReclaimPageTablesMode::Skip => {
    //                 // Skip non-empty page tables.
    //                 if p3.iter().any(|pte| !pte.is_unused()) {
    //                     continue;
    //                 }
    //             }
    //             ReclaimPageTablesMode::Panic => {
    //                 // Don't allow non-empty page tables.
    //                 assert!(p3.iter().all(|pte| pte.is_unused()));
    //             }
    //         }

    //         p4_entry.set_unused();
    //         deallocator.dealloc(p3_frame);
    //     }
    // }
}

impl<'a> Mapper<Size1GiB> for RecursivePageTable<'a> {
    fn map_to<A>(
        &mut self,
        page: Page<Size1GiB>,
        frame: PhysFrame<Size1GiB>,
        flags: PageTableFlags,
        allocator: &mut A,
    ) -> Result<MapperFlush<Size1GiB>, MapToError>
    where
        A: FrameAllocator<Size4KiB>,
    {
        use structures::paging::PageTableFlags as Flags;
        let p4 = &mut self.p4;

        let p3_page = p3_page(page, self.recursive_index);
        let p3 = unsafe { Self::create_next_table(&mut p4[page.p4_index()], p3_page, allocator)? };

        if !p3[page.p3_index()].is_unused() {
            return Err(MapToError::PageAlreadyMapped);
        }
        p3[page.p3_index()].set_addr(frame.start_address(), flags | Flags::HUGE_PAGE);

        Ok(MapperFlush::new(page))
    }

    fn unmap(
        &mut self,
        page: Page<Size1GiB>,
    ) -> Result<(PhysFrame<Size1GiB>, MapperFlush<Size1GiB>), UnmapError> {
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

        p3_entry.set_unused();
        Ok((frame, MapperFlush::new(page)))
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
        allocator: &mut A,
    ) -> Result<MapperFlush<Size2MiB>, MapToError>
    where
        A: FrameAllocator<Size4KiB>,
    {
        use structures::paging::PageTableFlags as Flags;
        let p4 = &mut self.p4;

        let p3_page = p3_page(page, self.recursive_index);
        let p3 = unsafe { Self::create_next_table(&mut p4[page.p4_index()], p3_page, allocator)? };

        let p2_page = p2_page(page, self.recursive_index);
        let p2 = unsafe { Self::create_next_table(&mut p3[page.p3_index()], p2_page, allocator)? };

        if !p2[page.p2_index()].is_unused() {
            return Err(MapToError::PageAlreadyMapped);
        }
        p2[page.p2_index()].set_addr(frame.start_address(), flags | Flags::HUGE_PAGE);

        Ok(MapperFlush::new(page))
    }

    fn unmap(
        &mut self,
        page: Page<Size2MiB>,
    ) -> Result<(PhysFrame<Size2MiB>, MapperFlush<Size2MiB>), UnmapError> {
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

        p2_entry.set_unused();
        Ok((frame, MapperFlush::new(page)))
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
        allocator: &mut A,
    ) -> Result<MapperFlush<Size4KiB>, MapToError>
    where
        A: FrameAllocator<Size4KiB>,
    {
        let p4 = &mut self.p4;

        let p3_page = p3_page(page, self.recursive_index);
        let p3 = unsafe { Self::create_next_table(&mut p4[page.p4_index()], p3_page, allocator)? };

        let p2_page = p2_page(page, self.recursive_index);
        let p2 = unsafe { Self::create_next_table(&mut p3[page.p3_index()], p2_page, allocator)? };

        let p1_page = p1_page(page, self.recursive_index);
        let p1 = unsafe { Self::create_next_table(&mut p2[page.p2_index()], p1_page, allocator)? };

        if !p1[page.p1_index()].is_unused() {
            return Err(MapToError::PageAlreadyMapped);
        }
        p1[page.p1_index()].set_frame(frame, flags);

        Ok(MapperFlush::new(page))
    }

    fn unmap(
        &mut self,
        page: Page<Size4KiB>,
    ) -> Result<(PhysFrame<Size4KiB>, MapperFlush<Size4KiB>), UnmapError> {
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

        p1_entry.set_unused();
        Ok((frame, MapperFlush::new(page)))
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
