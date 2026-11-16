#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![reexport_test_harness_main = "test_main"]
#![test_runner(testing::test_runner)]

use bootloader::bootinfo::{BootInfo, MemoryRegionType};
use core::panic::PanicInfo;
use spin::{Mutex, Once};
use testing::{serial_print, serial_println};
use x86_64::structures::paging::mapper::{
    FlagUpdateError, MapToError, MappedFrame, TranslateError, TranslateResult, UnmapError,
    UnmappedFrame,
};
use x86_64::structures::paging::{
    FrameAllocator, Mapper, Page, PageSize, PageTable, PageTableFlags, PageTableIndex, PhysFrame,
    RecursivePageTable, Size1GiB, Size2MiB, Size4KiB, Translate,
};
use x86_64::{PhysAddr, VirtAddr};

const FLAGS: PageTableFlags = PageTableFlags::PRESENT.union(PageTableFlags::WRITABLE);

static BOOT_INFO: Once<&'static BootInfo> = Once::new();
static FRAME_ALLOCATOR: Mutex<BootInfoFrameAllocator> =
    Mutex::new(BootInfoFrameAllocator { next: 0 });

#[no_mangle] // don't mangle the name of this function
pub extern "C" fn _start(boot_info: &'static BootInfo) -> ! {
    BOOT_INFO.call_once(|| boot_info);

    test_main();

    loop {}
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    testing::test_panic_handler(info)
}

/// Returns the usable physical memory regions as `(start, end)` addresses.
fn usable_regions() -> impl Iterator<Item = (u64, u64)> {
    let boot_info = BOOT_INFO.r#try().unwrap();
    boot_info
        .memory_map
        .iter()
        .filter(|region| region.region_type == MemoryRegionType::Usable)
        .map(|region| (region.range.start_addr(), region.range.end_addr()))
}

/// Returns an unused 2MiB frame that is backed by memory.
///
/// The frame is taken from the end of the usable memory, so that it doesn't overlap with the
/// frames returned by the [`BootInfoFrameAllocator`], which are taken from the start.
fn usable_2mib_frame() -> PhysFrame<Size2MiB> {
    let (_, end) = usable_regions()
        .filter(|(start, end)| {
            let frame_end = end & !(Size2MiB::SIZE - 1);
            frame_end >= start + 2 * Size2MiB::SIZE
        })
        .last()
        .expect("no usable 2MiB frame");
    let frame_end = end & !(Size2MiB::SIZE - 1);
    PhysFrame::from_start_address(PhysAddr::new(frame_end - Size2MiB::SIZE)).unwrap()
}

/// Allocates 4KiB frames for page tables from the start of the usable memory.
struct BootInfoFrameAllocator {
    next: usize,
}

unsafe impl FrameAllocator<Size4KiB> for BootInfoFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame> {
        let frame = usable_regions()
            .flat_map(|(start, end)| (start..end).step_by(Size4KiB::SIZE as usize))
            .nth(self.next)?;
        self.next += 1;
        Some(PhysFrame::containing_address(PhysAddr::new(frame)))
    }
}

/// Returns a mapper for the active page table.
fn mapper() -> RecursivePageTable<'static> {
    let boot_info = BOOT_INFO.r#try().unwrap();
    let level_4_table = boot_info.recursive_page_table_addr as *mut PageTable;
    RecursivePageTable::new(unsafe { &mut *level_4_table }).unwrap()
}

/// Returns a page in an unused part of the address space.
///
/// Each test uses its own level 4 entry, so that the tests don't affect each other.
fn test_page<S: PageSize>(mapper: &RecursivePageTable<'_>, p4_index: u16) -> Page<S> {
    assert!(mapper.level_4_table()[PageTableIndex::new(p4_index)].is_unused());
    Page::from_start_address(VirtAddr::new(u64::from(p4_index) << 39)).unwrap()
}

fn translated_flags(mapper: &RecursivePageTable<'_>, addr: VirtAddr) -> PageTableFlags {
    match mapper.translate(addr) {
        TranslateResult::Mapped { flags, .. } => flags,
        other => panic!("unexpected translate result: {:?}", other),
    }
}

/// Maps a 2MiB page with the PAT bit set and accesses it.
///
/// This checks that the CPU accepts the created entry, i.e. that the PAT bit doesn't end up
/// in the address of the entry.
#[test_case]
fn huge_page_with_pat_2mib() {
    serial_print!("huge_page_with_pat_2mib... ");
    let mut mapper = mapper();
    let page = test_page::<Size2MiB>(&mapper, 100);
    let frame = usable_2mib_frame();
    let with_pat = FLAGS | PageTableFlags::PAT_HUGE_PAGE;

    unsafe { mapper.map_to(page, frame, with_pat, &mut *FRAME_ALLOCATOR.lock()) }
        .unwrap()
        .flush();

    // Access the first and the last 4KiB of the page.
    let first = page.start_address().as_mut_ptr::<u64>();
    let last = (page.start_address() + (Size2MiB::SIZE - 8)).as_mut_ptr::<u64>();
    unsafe {
        first.write_volatile(0x1234_5678_9abc_def0);
        last.write_volatile(0x0fed_cba9_8765_4321);
        assert_eq!(first.read_volatile(), 0x1234_5678_9abc_def0);
        assert_eq!(last.read_volatile(), 0x0fed_cba9_8765_4321);
    }

    // The mapping can be read back, including the PAT bit.
    assert_eq!(mapper.translate_page(page).unwrap(), frame);
    match mapper.translate(page.start_address() + 0x1234u64) {
        TranslateResult::Mapped {
            frame: MappedFrame::Size2MiB(mapped_frame),
            offset,
            flags,
        } => {
            assert_eq!(mapped_frame, frame);
            assert_eq!(offset, 0x1234);
            assert!(flags.contains(with_pat | PageTableFlags::HUGE_PAGE));
        }
        other => panic!("unexpected translate result: {:?}", other),
    }

    // The PAT bit can be cleared and set again without affecting the address.
    unsafe { mapper.update_flags(page, FLAGS) }.unwrap().flush();
    assert!(
        !translated_flags(&mapper, page.start_address()).contains(PageTableFlags::PAT_HUGE_PAGE)
    );
    assert_eq!(mapper.translate_page(page).unwrap(), frame);
    assert_eq!(unsafe { first.read_volatile() }, 0x1234_5678_9abc_def0);

    unsafe { mapper.update_flags(page, with_pat) }
        .unwrap()
        .flush();
    assert!(translated_flags(&mapper, page.start_address()).contains(PageTableFlags::PAT_HUGE_PAGE));
    assert_eq!(unsafe { last.read_volatile() }, 0x0fed_cba9_8765_4321);

    // Unmapping returns the frame and the flags.
    let (unmapped_frame, flags, flush) = mapper.unmap(page).unwrap();
    flush.flush();
    assert_eq!(unmapped_frame, frame);
    assert!(flags.contains(with_pat | PageTableFlags::HUGE_PAGE));
    assert!(matches!(
        mapper.translate_page(page),
        Err(TranslateError::PageNotMapped)
    ));

    serial_println!("[ok]");
}

/// Maps a 1GiB page with the PAT bit set.
///
/// The page is not accessed because 1GiB pages are not supported by all (emulated) CPUs.
#[test_case]
fn huge_page_with_pat_1gib() {
    serial_print!("huge_page_with_pat_1gib... ");
    let mut mapper = mapper();
    let page = test_page::<Size1GiB>(&mapper, 101);
    let frame = PhysFrame::<Size1GiB>::from_start_address(PhysAddr::new(0x4000_0000)).unwrap();
    let with_pat = FLAGS | PageTableFlags::PAT_HUGE_PAGE;

    unsafe { mapper.map_to(page, frame, with_pat, &mut *FRAME_ALLOCATOR.lock()) }
        .unwrap()
        .ignore();

    assert_eq!(mapper.translate_page(page).unwrap(), frame);
    assert_eq!(
        translated_flags(&mapper, page.start_address()),
        with_pat | PageTableFlags::HUGE_PAGE
    );

    unsafe { mapper.update_flags(page, FLAGS) }
        .unwrap()
        .ignore();
    assert_eq!(
        translated_flags(&mapper, page.start_address()),
        FLAGS | PageTableFlags::HUGE_PAGE
    );
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
            assert_eq!(flags, with_pat | PageTableFlags::HUGE_PAGE);
        }
        UnmappedFrame::NotPresent { .. } => panic!("page should be present"),
    }

    serial_println!("[ok]");
}

#[test_case]
fn map_to_rejects_invalid_flags() {
    serial_print!("map_to_rejects_invalid_flags... ");
    let mut mapper = mapper();
    let page = test_page::<Size4KiB>(&mapper, 102);
    let frame = PhysFrame::<Size4KiB>::containing_address(PhysAddr::new(0x4000_0000));
    let frames_before = FRAME_ALLOCATOR.lock().next;

    // The PAT bit of huge pages is part of the address of 4KiB pages.
    let pat = PageTableFlags::PAT_HUGE_PAGE;
    let result = unsafe { mapper.map_to(page, frame, FLAGS | pat, &mut *FRAME_ALLOCATOR.lock()) };
    assert!(matches!(result, Err(MapToError::InvalidFlags(flags)) if flags == pat));

    // Parent tables must not be turned into huge pages.
    let huge = PageTableFlags::HUGE_PAGE;
    let result = unsafe {
        mapper.map_to_with_table_flags(
            page,
            frame,
            FLAGS,
            FLAGS | huge,
            &mut *FRAME_ALLOCATOR.lock(),
        )
    };
    assert!(matches!(result, Err(MapToError::InvalidFlags(flags)) if flags == huge));

    let page_2mib = Page::<Size2MiB>::containing_address(page.start_address());
    let result = unsafe {
        mapper.map_to_with_table_flags(
            page_2mib,
            usable_2mib_frame(),
            FLAGS,
            FLAGS | pat,
            &mut *FRAME_ALLOCATOR.lock(),
        )
    };
    assert!(matches!(result, Err(MapToError::InvalidFlags(flags)) if flags == pat));

    // Nothing was allocated or mapped.
    assert_eq!(FRAME_ALLOCATOR.lock().next, frames_before);
    assert!(mapper.level_4_table()[page.p4_index()].is_unused());

    serial_println!("[ok]");
}

#[test_case]
fn update_flags_rejects_invalid_flags() {
    serial_print!("update_flags_rejects_invalid_flags... ");
    let mut mapper = mapper();
    let page = test_page::<Size4KiB>(&mapper, 103);
    let frame = PhysFrame::<Size4KiB>::containing_address(PhysAddr::new(0x4000_0000));

    unsafe { mapper.map_to(page, frame, FLAGS, &mut *FRAME_ALLOCATOR.lock()) }
        .unwrap()
        .ignore();

    let pat = PageTableFlags::PAT_HUGE_PAGE;
    let huge = PageTableFlags::HUGE_PAGE;
    assert!(matches!(
        unsafe { mapper.update_flags(page, FLAGS | pat) },
        Err(FlagUpdateError::InvalidFlags(flags)) if flags == pat
    ));
    assert!(matches!(
        unsafe { mapper.set_flags_p4_entry(page, FLAGS | huge) },
        Err(FlagUpdateError::InvalidFlags(flags)) if flags == huge
    ));
    assert!(matches!(
        unsafe { mapper.set_flags_p3_entry(page, FLAGS | huge) },
        Err(FlagUpdateError::InvalidFlags(flags)) if flags == huge
    ));
    assert!(matches!(
        unsafe { mapper.set_flags_p2_entry(page, FLAGS | pat) },
        Err(FlagUpdateError::InvalidFlags(flags)) if flags == pat
    ));

    // The PAT bit of 4KiB pages is still accepted.
    let pat_4kib = FLAGS | PageTableFlags::PAT_4KIB_PAGE;
    unsafe { mapper.update_flags(page, pat_4kib) }
        .unwrap()
        .ignore();

    // The mapping is otherwise unchanged.
    let (unmapped_frame, flags, flush) = mapper.unmap(page).unwrap();
    flush.ignore();
    assert_eq!(unmapped_frame, frame);
    assert_eq!(flags, pat_4kib);

    serial_println!("[ok]");
}

/// A 4KiB mapping creates page table pointers in the level 3 and level 2 tables. The huge
/// page methods must not treat these entries as huge pages.
#[test_case]
fn huge_page_methods_reject_page_table_pointers() {
    serial_print!("huge_page_methods_reject_page_table_pointers... ");
    let mut mapper = mapper();
    let page = test_page::<Size4KiB>(&mapper, 104);
    let page_2mib = Page::<Size2MiB>::containing_address(page.start_address());
    let page_1gib = Page::<Size1GiB>::containing_address(page.start_address());
    let frame = PhysFrame::<Size4KiB>::containing_address(PhysAddr::new(0x4000_0000));

    unsafe { mapper.map_to(page, frame, FLAGS, &mut *FRAME_ALLOCATOR.lock()) }
        .unwrap()
        .ignore();

    assert!(matches!(
        unsafe { mapper.update_flags(page_2mib, FLAGS) },
        Err(FlagUpdateError::ParentEntryHugePage)
    ));
    assert!(matches!(
        unsafe { mapper.update_flags(page_1gib, FLAGS) },
        Err(FlagUpdateError::ParentEntryHugePage)
    ));
    assert!(matches!(
        mapper.translate_page(page_2mib),
        Err(TranslateError::ParentEntryHugePage)
    ));
    assert!(matches!(
        mapper.translate_page(page_1gib),
        Err(TranslateError::ParentEntryHugePage)
    ));
    assert!(matches!(
        mapper.unmap(page_2mib),
        Err(UnmapError::ParentEntryHugePage)
    ));

    // The 4KiB mapping is still intact.
    assert_eq!(mapper.translate_page(page).unwrap(), frame);

    serial_println!("[ok]");
}
