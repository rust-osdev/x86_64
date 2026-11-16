//! Tests for [`MappedPageTable`] that run on the host.
//!
//! The page tables are allocated on the heap and their addresses are used as physical
//! addresses. Only the page tables themselves are ever accessed, so the pages and frames
//! used in the mappings don't need to exist.

use std::{boxed::Box, vec::Vec};

use super::*;
use crate::PhysAddr;

const FLAGS: PageTableFlags = PageTableFlags::PRESENT.union(PageTableFlags::WRITABLE);

/// Treats the address of a heap-allocated page table as its physical address.
struct HostMapping;

unsafe impl PageTableFrameMapping for HostMapping {
    fn frame_to_pointer(&self, frame: PhysFrame) -> *mut PageTable {
        // The provenance of the page tables is exposed by the `TableAllocator`.
        core::ptr::with_exposed_provenance_mut(frame.start_address().as_u64() as usize)
    }
}

/// Allocates page tables on the heap and frees them when it is dropped.
///
/// The page tables are owned through raw pointers instead of `Box`es because the mapper
/// accesses them through pointers that are created from their address. This would violate
/// the uniqueness of a `Box`.
#[derive(Default)]
struct TableAllocator {
    tables: Vec<*mut PageTable>,
}

unsafe impl FrameAllocator<Size4KiB> for TableAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame> {
        let table = Box::into_raw(Box::new(PageTable::new()));
        self.tables.push(table);
        let addr = PhysAddr::new(table.expose_provenance() as u64);
        Some(PhysFrame::from_start_address(addr).unwrap())
    }
}

impl Drop for TableAllocator {
    fn drop(&mut self) {
        for table in self.tables.drain(..) {
            drop(unsafe { Box::from_raw(table) });
        }
    }
}

fn page<S: PageSize>() -> Page<S> {
    Page::from_start_address(VirtAddr::new(0x40_4000_0000)).unwrap()
}

fn frame<S: PageSize>() -> PhysFrame<S> {
    PhysFrame::from_start_address(PhysAddr::new(0x8000_0000)).unwrap()
}

/// Maps a page, reads the mapping back in all available ways, and unmaps it again.
///
/// `flags` are the flags used for the mapping and `read_flags` the flags that are expected
/// to be read back.
fn round_trip<S>(flags: PageTableFlags, read_flags: PageTableFlags)
where
    S: PageSize + core::fmt::Debug,
    for<'a> MappedPageTable<'a, HostMapping>: Mapper<S>,
{
    let mut level_4_table = PageTable::new();
    let mut mapper = unsafe { MappedPageTable::new(&mut level_4_table, HostMapping) };
    let mut allocator = TableAllocator::default();
    let (page, frame) = (page::<S>(), frame::<S>());

    unsafe { mapper.map_to(page, frame, flags, &mut allocator) }
        .unwrap()
        .ignore();

    assert_eq!(mapper.translate_page(page).unwrap(), frame);

    match mapper.translate(page.start_address() + 0x123u64) {
        TranslateResult::Mapped {
            frame: mapped_frame,
            offset,
            flags,
        } => {
            assert_eq!(mapped_frame.start_address(), frame.start_address());
            assert_eq!(mapped_frame.size(), S::SIZE);
            assert_eq!(offset, 0x123);
            assert_eq!(flags, read_flags);
        }
        other => panic!("unexpected translate result: {other:?}"),
    }

    let (unmapped_frame, unmapped_flags, flush) = mapper.unmap(page).unwrap();
    flush.ignore();
    assert_eq!(unmapped_frame, frame);
    assert_eq!(unmapped_flags, read_flags);

    assert!(matches!(
        mapper.translate_page(page),
        Err(TranslateError::PageNotMapped)
    ));
}

#[test]
fn round_trip_4kib() {
    round_trip::<Size4KiB>(FLAGS, FLAGS);
}

#[test]
fn round_trip_2mib() {
    round_trip::<Size2MiB>(FLAGS, FLAGS | PageTableFlags::HUGE_PAGE);
}

#[test]
fn round_trip_1gib() {
    round_trip::<Size1GiB>(FLAGS, FLAGS | PageTableFlags::HUGE_PAGE);
}

#[test]
fn round_trip_2mib_with_pat() {
    let flags = FLAGS | PageTableFlags::PAT_HUGE_PAGE;
    round_trip::<Size2MiB>(flags, flags | PageTableFlags::HUGE_PAGE);
}

#[test]
fn round_trip_1gib_with_pat() {
    let flags = FLAGS | PageTableFlags::PAT_HUGE_PAGE;
    round_trip::<Size1GiB>(flags, flags | PageTableFlags::HUGE_PAGE);
}

/// Sets and clears the PAT bit of an existing mapping.
///
/// The PAT bit is at a different position for 4KiB pages and huge pages, so the flag is
/// selected through the page.
fn update_pat<S>()
where
    S: PageSize + core::fmt::Debug,
    for<'a> MappedPageTable<'a, HostMapping>: Mapper<S>,
{
    let mut level_4_table = PageTable::new();
    let mut mapper = unsafe { MappedPageTable::new(&mut level_4_table, HostMapping) };
    let mut allocator = TableAllocator::default();
    let (page, frame) = (page::<S>(), frame::<S>());
    let read_flags = |mapper: &MappedPageTable<'_, HostMapping>| {
        let TranslateResult::Mapped { flags, .. } = mapper.translate(page.start_address()) else {
            panic!("page should be mapped");
        };
        flags
    };

    // The mapper sets the `HUGE_PAGE` flag for huge pages.
    let huge_page = if S::SIZE == Size4KiB::SIZE {
        PageTableFlags::empty()
    } else {
        PageTableFlags::HUGE_PAGE
    };
    let with_pat = FLAGS | page.pat_flag();
    // The PAT bit must be distinguishable from the `HUGE_PAGE` flag, otherwise the checks
    // below would pass without it.
    assert_ne!(with_pat | huge_page, FLAGS | huge_page);

    unsafe { mapper.map_to(page, frame, FLAGS, &mut allocator) }
        .unwrap()
        .ignore();
    assert_eq!(read_flags(&mapper), FLAGS | huge_page);

    unsafe { mapper.update_flags(page, with_pat) }
        .unwrap()
        .ignore();
    assert_eq!(read_flags(&mapper), with_pat | huge_page);
    assert_eq!(mapper.translate_page(page).unwrap(), frame);

    unsafe { mapper.update_flags(page, FLAGS) }
        .unwrap()
        .ignore();
    assert_eq!(read_flags(&mapper), FLAGS | huge_page);
    assert_eq!(mapper.translate_page(page).unwrap(), frame);

    unsafe { mapper.update_flags(page, with_pat) }
        .unwrap()
        .ignore();
    match mapper.clear(page).unwrap() {
        UnmappedFrame::Present {
            frame: cleared_frame,
            flags,
            flush,
        } => {
            flush.ignore();
            assert_eq!(cleared_frame, frame);
            assert_eq!(flags, with_pat | huge_page);
        }
        UnmappedFrame::NotPresent { .. } => panic!("page should be present"),
    }
}

#[test]
fn update_pat_4kib() {
    update_pat::<Size4KiB>();
}

#[test]
fn update_pat_2mib() {
    update_pat::<Size2MiB>();
}

#[test]
fn update_pat_1gib() {
    update_pat::<Size1GiB>();
}

/// A 4KiB mapping creates page table pointers in the level 3 and level 2 tables. The huge
/// page methods must not treat these entries as huge pages.
#[test]
fn huge_page_methods_reject_page_table_pointers() {
    let mut level_4_table = PageTable::new();
    let mut mapper = unsafe { MappedPageTable::new(&mut level_4_table, HostMapping) };
    let mut allocator = TableAllocator::default();

    unsafe { mapper.map_to(page::<Size4KiB>(), frame(), FLAGS, &mut allocator) }
        .unwrap()
        .ignore();

    assert!(matches!(
        unsafe { mapper.update_flags(page::<Size2MiB>(), FLAGS) },
        Err(FlagUpdateError::ParentEntryHugePage)
    ));
    assert!(matches!(
        unsafe { mapper.update_flags(page::<Size1GiB>(), FLAGS) },
        Err(FlagUpdateError::ParentEntryHugePage)
    ));
    assert!(matches!(
        mapper.translate_page(page::<Size2MiB>()),
        Err(TranslateError::ParentEntryHugePage)
    ));
    assert!(matches!(
        mapper.translate_page(page::<Size1GiB>()),
        Err(TranslateError::ParentEntryHugePage)
    ));

    // The 4KiB mapping is still intact.
    assert_eq!(mapper.translate_page(page::<Size4KiB>()).unwrap(), frame());
}
