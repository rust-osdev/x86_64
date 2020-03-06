//! Abstractions for reading and modifying the mapping of pages.

pub use self::mapped_page_table::{MappedPageTable, PhysToVirt};
#[cfg(target_arch = "x86_64")]
pub use self::{offset_page_table::OffsetPageTable, recursive_page_table::RecursivePageTable};

use crate::structures::paging::{
    frame_alloc::FrameAllocator, page_table::PageTableFlags, Page, PageSize, PhysFrame, Size1GiB,
    Size2MiB, Size4KiB,
};
use crate::{PhysAddr, VirtAddr};

mod mapped_page_table;
mod offset_page_table;
mod recursive_page_table;

/// This trait defines page table operations that work for all page sizes of the x86_64
/// architecture.
pub trait MapperAllSizes: Mapper<Size4KiB> + Mapper<Size2MiB> + Mapper<Size1GiB> {
    /// Return the frame that the given virtual address is mapped to and the offset within that
    /// frame.
    ///
    /// If the given address has a valid mapping, the mapped frame and the offset within that
    /// frame is returned. Otherwise an error value is returned.
    ///
    /// This function works with huge pages of all sizes.
    fn translate(&self, addr: VirtAddr) -> TranslateResult;

    /// Translates the given virtual address to the physical address that it maps to.
    ///
    /// Returns `None` if there is no valid mapping for the given address.
    ///
    /// This is a convenience method. For more information about a mapping see the
    /// [`translate`](MapperAllSizes::translate) method.
    #[inline]
    fn translate_addr(&self, addr: VirtAddr) -> Option<PhysAddr> {
        match self.translate(addr) {
            TranslateResult::PageNotMapped | TranslateResult::InvalidFrameAddress(_) => None,
            TranslateResult::Frame4KiB { frame, offset } => Some(frame.start_address() + offset),
            TranslateResult::Frame2MiB { frame, offset } => Some(frame.start_address() + offset),
            TranslateResult::Frame1GiB { frame, offset } => Some(frame.start_address() + offset),
        }
    }
}

/// The return value of the [`MapperAllSizes::translate`] function.
///
/// If the given address has a valid mapping, a `Frame4KiB`, `Frame2MiB`, or `Frame1GiB` variant
/// is returned, depending on the size of the mapped page. The remaining variants indicate errors.
#[derive(Debug)]
pub enum TranslateResult {
    /// The page is mapped to a physical frame of size 4KiB.
    Frame4KiB {
        /// The mapped frame.
        frame: PhysFrame<Size4KiB>,
        /// The offset whithin the mapped frame.
        offset: u64,
    },
    /// The page is mapped to a physical frame of size 2MiB.
    Frame2MiB {
        /// The mapped frame.
        frame: PhysFrame<Size2MiB>,
        /// The offset whithin the mapped frame.
        offset: u64,
    },
    /// The page is mapped to a physical frame of size 2MiB.
    Frame1GiB {
        /// The mapped frame.
        frame: PhysFrame<Size1GiB>,
        /// The offset whithin the mapped frame.
        offset: u64,
    },
    /// The given page is not mapped to a physical frame.
    PageNotMapped,
    /// The page table entry for the given page points to an invalid physical address.
    InvalidFrameAddress(PhysAddr),
}

/// A trait for common page table operations on pages of size `S`.
pub trait Mapper<S: PageSize> {
    /// Creates a new mapping in the page table.
    ///
    /// This function might need additional physical frames to create new page tables. These
    /// frames are allocated from the `allocator` argument. At most three frames are required.
    ///
    /// ## Safety
    ///
    /// Creating page table mappings is a fundamentally unsafe operation because
    /// there are various ways to break memory safety through it. For example,
    /// re-mapping an in-use page to a different frame changes and invalidates
    /// all values stored in that page, resulting in undefined behavior on the
    /// next use.
    ///
    /// The caller must ensure that no undefined behavior or memory safety
    /// violations can occur through the new mapping. Among other things, the
    /// caller must prevent the following:
    ///
    /// - Aliasing of `&mut` references, i.e. two `&mut` references that point to
    ///   the same physical address. This is undefined behavior in Rust.
    ///     - This can be ensured by mapping each page to an individual physical
    ///       frame that is not mapped anywhere else.
    /// - Creating uninitalized or invalid values: Rust requires that all values
    ///   have a correct memory layout. For example, a `bool` must be either a 0
    ///   or a 1 in memory, but not a 3 or 4. An exception is the `MaybeUninit`
    ///   wrapper type, which abstracts over possibly uninitialized memory.
    ///     - This is only a problem when re-mapping pages to different physical
    ///       frames. Mapping a page that is not in use yet is fine.
    ///
    /// Special care must be taken when sharing pages with other address spaces,
    /// e.g. by setting the `GLOBAL` flag. For example, a global mapping must be
    /// the same in all address spaces, otherwise undefined behavior can occur
    /// because of TLB races. It's worth noting that all the above requirements
    /// also apply to shared mappings, including the aliasing requirements.
    unsafe fn map_to<A>(
        &mut self,
        page: Page<S>,
        frame: PhysFrame<S>,
        flags: PageTableFlags,
        frame_allocator: &mut A,
    ) -> Result<MapperFlush<S>, MapToError<S>>
    where
        Self: Sized,
        A: FrameAllocator<Size4KiB>;

    /// Removes a mapping from the page table and returns the frame that used to be mapped.
    ///
    /// Note that no page tables or pages are deallocated.
    fn unmap(&mut self, page: Page<S>) -> Result<(PhysFrame<S>, MapperFlush<S>), UnmapError>;

    /// Updates the flags of an existing mapping.
    fn update_flags(
        &mut self,
        page: Page<S>,
        flags: PageTableFlags,
    ) -> Result<MapperFlush<S>, FlagUpdateError>;

    /// Return the frame that the specified page is mapped to.
    ///
    /// This function assumes that the page is mapped to a frame of size `S` and returns an
    /// error otherwise.
    fn translate_page(&self, page: Page<S>) -> Result<PhysFrame<S>, TranslateError>;

    /// Maps the given frame to the virtual page with the same address.
    ///
    /// ## Safety
    ///
    /// This is a convencience function that invokes [`map_to`] internally, so
    /// all safety requirements of it also apply for this function.
    #[inline]
    unsafe fn identity_map<A>(
        &mut self,
        frame: PhysFrame<S>,
        flags: PageTableFlags,
        frame_allocator: &mut A,
    ) -> Result<MapperFlush<S>, MapToError<S>>
    where
        Self: Sized,
        A: FrameAllocator<Size4KiB>,
        S: PageSize,
        Self: Mapper<S>,
    {
        let page = Page::containing_address(VirtAddr::new(frame.start_address().as_u64()));
        self.map_to(page, frame, flags, frame_allocator)
    }
}

/// This type represents a page whose mapping has changed in the page table.
///
/// The old mapping might be still cached in the translation lookaside buffer (TLB), so it needs
/// to be flushed from the TLB before it's accessed. This type is returned from function that
/// change the mapping of a page to ensure that the TLB flush is not forgotten.
#[derive(Debug)]
#[must_use = "Page Table changes must be flushed or ignored."]
pub struct MapperFlush<S: PageSize>(Page<S>);

impl<S: PageSize> MapperFlush<S> {
    /// Create a new flush promise
    fn new(page: Page<S>) -> Self {
        MapperFlush(page)
    }

    /// Flush the page from the TLB to ensure that the newest mapping is used.
    #[cfg(target_arch = "x86_64")]
    #[inline]
    pub fn flush(self) {
        crate::instructions::tlb::flush(self.0.start_address());
    }

    /// Don't flush the TLB and silence the “must be used” warning.
    #[inline]
    pub fn ignore(self) {}
}

/// This error is returned from `map_to` and similar methods.
#[derive(Debug)]
pub enum MapToError<S: PageSize> {
    /// An additional frame was needed for the mapping process, but the frame allocator
    /// returned `None`.
    FrameAllocationFailed,
    /// An upper level page table entry has the `HUGE_PAGE` flag set, which means that the
    /// given page is part of an already mapped huge page.
    ParentEntryHugePage,
    /// The given page is already mapped to a physical frame.
    PageAlreadyMapped(PhysFrame<S>),
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
    /// An upper level page table entry has the `HUGE_PAGE` flag set, which means that the
    /// given page is part of a huge page and can't be freed individually.
    ParentEntryHugePage,
}

/// An error indicating that an `translate` call failed.
#[derive(Debug)]
pub enum TranslateError {
    /// The given page is not mapped to a physical frame.
    PageNotMapped,
    /// An upper level page table entry has the `HUGE_PAGE` flag set, which means that the
    /// given page is part of a huge page and can't be freed individually.
    ParentEntryHugePage,
    /// The page table entry for the given page points to an invalid physical address.
    InvalidFrameAddress(PhysAddr),
}

static _ASSERT_OBJECT_SAFE: Option<&(dyn MapperAllSizes + Sync)> = None;
