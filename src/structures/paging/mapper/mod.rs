//! Abstractions for reading and modifying the mapping of pages.

pub use self::mapped_page_table::{
    Display as MappedPageTableDisplay, MappedPageTable, PageTableFrameMapping,
};
#[cfg(target_pointer_width = "64")]
pub use self::mapped_page_table::{OffsetPageTable, PhysOffset};
#[cfg(all(feature = "virt_addr_rt", target_arch = "x86_64"))]
pub use self::recursive_page_table::{InvalidPageTable, RecursivePageTable};

use crate::PhysAddr;
use crate::addr::{FixedValidity, VirtAddrGeneric, VirtAddrValidity};
use crate::structures::paging::{
    Page, PageSize, PhysFrame, Size1GiB, Size2MiB, Size4KiB,
    frame_alloc::{FrameAllocator, FrameDeallocator},
    page::PageRangeInclusive,
    page_table::{PageTableEntry, PageTableFlags},
};

#[cfg(any(test, all(feature = "virt_addr_rt", target_arch = "x86_64")))]
use crate::structures::paging::page_table::PageTableLevel;

mod mapped_page_table;
#[cfg(all(feature = "virt_addr_rt", target_arch = "x86_64"))]
mod recursive_page_table;

/// An empty convenience trait that requires the `Mapper` trait for all page sizes.
pub trait MapperAllSizes<V: VirtAddrValidity = FixedValidity<48>>:
    Mapper<Size4KiB, V> + Mapper<Size2MiB, V> + Mapper<Size1GiB, V>
{
}

impl<T, V: VirtAddrValidity> MapperAllSizes<V> for T where
    T: Mapper<Size4KiB, V> + Mapper<Size2MiB, V> + Mapper<Size1GiB, V>
{
}

/// Provides methods for translating virtual addresses.
pub trait Translate<V: VirtAddrValidity = FixedValidity<48>> {
    /// Return the frame that the given virtual address is mapped to and the offset within that
    /// frame.
    ///
    /// If the given address has a valid mapping, the mapped frame and the offset within that
    /// frame is returned. Otherwise an error value is returned.
    ///
    /// This function works with huge pages of all sizes.
    fn translate(&self, addr: VirtAddrGeneric<V>) -> TranslateResult;

    /// Translates the given virtual address to the physical address that it maps to.
    ///
    /// Returns `None` if there is no valid mapping for the given address.
    ///
    /// This is a convenience method. For more information about a mapping see the
    /// [`translate`](Translate::translate) method.
    #[inline]
    fn translate_addr(&self, addr: VirtAddrGeneric<V>) -> Option<PhysAddr> {
        match self.translate(addr) {
            TranslateResult::NotMapped
            | TranslateResult::AddressNotValid
            | TranslateResult::InvalidFrameAddress(_) => None,
            TranslateResult::Mapped { frame, offset, .. } => Some(frame.start_address() + offset),
        }
    }
}

/// The return value of the [`Translate::translate`] function.
///
/// If the given address has a valid mapping, a `Frame4KiB`, `Frame2MiB`, or `Frame1GiB` variant
/// is returned, depending on the size of the mapped page. The remaining variants indicate errors.
#[derive(Debug)]
pub enum TranslateResult {
    /// The virtual address is mapped to a physical frame.
    Mapped {
        /// The mapped frame.
        frame: MappedFrame,
        /// The offset within the mapped frame.
        offset: u64,
        /// The entry flags in the lowest-level page table.
        ///
        /// Flags of higher-level page table entries are not included here, but they can still
        /// affect the effective flags for an address, for example when the WRITABLE flag is not
        /// set for a level 3 entry.
        flags: PageTableFlags,
    },
    /// The given virtual address is not mapped to a physical frame.
    NotMapped,
    /// The address is valid under `V` but cannot be represented by this root level.
    AddressNotValid,
    /// The page table entry for the given virtual address points to an invalid physical address.
    InvalidFrameAddress(PhysAddr),
}

/// Represents a physical frame mapped in a page table.
#[derive(Debug)]
pub enum MappedFrame {
    /// The virtual address is mapped to a 4KiB frame.
    Size4KiB(PhysFrame<Size4KiB>),
    /// The virtual address is mapped to a "large" 2MiB frame.
    Size2MiB(PhysFrame<Size2MiB>),
    /// The virtual address is mapped to a "huge" 1GiB frame.
    Size1GiB(PhysFrame<Size1GiB>),
}

impl MappedFrame {
    /// Returns the start address of the frame.
    pub const fn start_address(&self) -> PhysAddr {
        match self {
            MappedFrame::Size4KiB(frame) => frame.start_address,
            MappedFrame::Size2MiB(frame) => frame.start_address,
            MappedFrame::Size1GiB(frame) => frame.start_address,
        }
    }

    /// Returns the size the frame (4KB, 2MB or 1GB).
    pub const fn size(&self) -> u64 {
        match self {
            MappedFrame::Size4KiB(_) => Size4KiB::SIZE,
            MappedFrame::Size2MiB(_) => Size2MiB::SIZE,
            MappedFrame::Size1GiB(_) => Size1GiB::SIZE,
        }
    }
}

/// The result of [`Mapper::unmap`].
pub type MapperUnmapResult<S, V> = (PhysFrame<S>, PageTableFlags, MapperFlush<S, V>);

/// A trait for common page table operations on pages of size `S`.
pub trait Mapper<S: PageSize, V: VirtAddrValidity = FixedValidity<48>> {
    /// Creates a new mapping in the page table.
    ///
    /// This function might need additional physical frames to create new page tables. These
    /// frames are allocated from the `allocator` argument. At most four frames are required for
    /// a five-level root (three for a four-level root).
    ///
    /// Parent page table entries are automatically updated with `PRESENT | WRITABLE | USER_ACCESSIBLE`
    /// if present in the `PageTableFlags`. Depending on the used mapper implementation
    /// the `PRESENT` and `WRITABLE` flags might be set for parent tables,
    /// even if they are not set in `PageTableFlags`.
    ///
    /// The `map_to_with_table_flags` method gives explicit control over the parent page table flags.
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
    /// - Creating uninitialized or invalid values: Rust requires that all values
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
    ///
    /// # Examples
    ///
    /// Create a USER_ACCESSIBLE mapping:
    ///
    /// ```
    /// # #[cfg(all(feature = "instructions", target_arch = "x86_64"))]
    /// # use x86_64::structures::paging::{
    /// #    Mapper, Page, PhysFrame, FrameAllocator,
    /// #    Size4KiB, OffsetPageTable, page_table::PageTableFlags
    /// # };
    /// # #[cfg(all(feature = "instructions", target_arch = "x86_64"))]
    /// # unsafe fn test(mapper: &mut OffsetPageTable, frame_allocator: &mut impl FrameAllocator<Size4KiB>,
    /// #         page: Page<Size4KiB>, frame: PhysFrame) {
    ///         mapper
    ///           .map_to(
    ///               page,
    ///               frame,
    ///              PageTableFlags::PRESENT
    ///                   | PageTableFlags::WRITABLE
    ///                   | PageTableFlags::USER_ACCESSIBLE,
    ///               frame_allocator,
    ///           )
    ///           .unwrap()
    ///           .flush();
    /// # }
    /// ```
    #[inline]
    unsafe fn map_to<A>(
        &mut self,
        page: Page<S, V>,
        frame: PhysFrame<S>,
        flags: PageTableFlags,
        frame_allocator: &mut A,
    ) -> Result<MapperFlush<S, V>, MapToError<S>>
    where
        Self: Sized,
        A: FrameAllocator<Size4KiB> + ?Sized,
    {
        let parent_table_flags = flags
            & (PageTableFlags::PRESENT
                | PageTableFlags::WRITABLE
                | PageTableFlags::USER_ACCESSIBLE);

        unsafe {
            self.map_to_with_table_flags(page, frame, flags, parent_table_flags, frame_allocator)
        }
    }

    /// Creates a new mapping in the page table.
    ///
    /// This function might need additional physical frames to create new page tables. These
    /// frames are allocated from the `allocator` argument. At most four frames are required for
    /// a five-level root (three for a four-level root).
    ///
    /// The flags of the parent table(s) can be explicitly specified. Those flags are used for
    /// newly created table entries, and for existing entries the flags are added.
    ///
    /// Depending on the used mapper implementation, the `PRESENT` and `WRITABLE` flags might
    /// be set for parent tables, even if they are not specified in `parent_table_flags`.
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
    /// - Creating uninitialized or invalid values: Rust requires that all values
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
    ///
    /// # Examples
    ///
    /// Create USER_ACCESSIBLE | NO_EXECUTE | NO_CACHE mapping and update
    /// the top hierarchy only with USER_ACCESSIBLE:
    ///
    /// ```
    /// # #[cfg(all(feature = "instructions", target_arch = "x86_64"))]
    /// # use x86_64::structures::paging::{
    /// #    Mapper, PhysFrame, Page, FrameAllocator,
    /// #    Size4KiB, OffsetPageTable, page_table::PageTableFlags
    /// # };
    /// # #[cfg(all(feature = "instructions", target_arch = "x86_64"))]
    /// # unsafe fn test(mapper: &mut OffsetPageTable, frame_allocator: &mut impl FrameAllocator<Size4KiB>,
    /// #         page: Page<Size4KiB>, frame: PhysFrame) {
    ///         mapper
    ///           .map_to_with_table_flags(
    ///               page,
    ///               frame,
    ///              PageTableFlags::PRESENT
    ///                   | PageTableFlags::WRITABLE
    ///                   | PageTableFlags::USER_ACCESSIBLE
    ///                   | PageTableFlags::NO_EXECUTE
    ///                   | PageTableFlags::NO_CACHE,
    ///              PageTableFlags::USER_ACCESSIBLE,
    ///               frame_allocator,
    ///           )
    ///           .unwrap()
    ///           .flush();
    /// # }
    /// ```
    unsafe fn map_to_with_table_flags<A>(
        &mut self,
        page: Page<S, V>,
        frame: PhysFrame<S>,
        flags: PageTableFlags,
        parent_table_flags: PageTableFlags,
        frame_allocator: &mut A,
    ) -> Result<MapperFlush<S, V>, MapToError<S>>
    where
        Self: Sized,
        A: FrameAllocator<Size4KiB> + ?Sized;

    /// Removes a mapping from the page table and returns the frame that used to be mapped.
    ///
    /// Note that no page tables or pages are deallocated.
    fn unmap(&mut self, page: Page<S, V>) -> Result<MapperUnmapResult<S, V>, UnmapError>;

    /// Clears a mapping from the page table and returns the frame that used to be mapped.
    ///
    /// Unlike [`Mapper::unmap`] this will ignore the present flag of the page and will successfully
    /// clear the table entry for any valid page.
    ///
    /// Note that no page tables or pages are deallocated.
    fn clear(&mut self, page: Page<S, V>) -> Result<UnmappedFrame<S, V>, UnmapError>;

    /// Updates the flags of an existing mapping.
    ///
    /// To read the current flags of a mapped page, use the [`Translate::translate`] method.
    ///
    /// ## Safety
    ///
    /// This method is unsafe because changing the flags of a mapping
    /// might result in undefined behavior. For example, setting the
    /// `GLOBAL` and `WRITABLE` flags for a page might result in the corruption
    /// of values stored in that page from processes running in other address
    /// spaces.
    unsafe fn update_flags(
        &mut self,
        page: Page<S, V>,
        flags: PageTableFlags,
    ) -> Result<MapperFlush<S, V>, FlagUpdateError>;

    /// Set the flags of the root level-5 entry. On a level-4 mapper this
    /// returns [`FlagUpdateError::PageTableLevelNotPresent`].
    ///
    /// # Safety
    ///
    /// As with [Self::update_flags], the caller must ensure that changing the effective
    /// permissions of every mapping below this entry cannot violate memory safety.
    unsafe fn set_flags_p5_entry(
        &mut self,
        page: Page<S, V>,
        flags: PageTableFlags,
    ) -> Result<MapperFlushAll, FlagUpdateError> {
        let _ = (page, flags);
        Err(FlagUpdateError::PageTableLevelNotPresent)
    }

    /// Set the flags of an existing page level 4 table entry
    ///
    /// ## Safety
    ///
    /// This method is unsafe because changing the flags of a mapping
    /// might result in undefined behavior. For example, setting the
    /// `GLOBAL` and `WRITABLE` flags for a page might result in the corruption
    /// of values stored in that page from processes running in other address
    /// spaces.
    unsafe fn set_flags_p4_entry(
        &mut self,
        page: Page<S, V>,
        flags: PageTableFlags,
    ) -> Result<MapperFlushAll, FlagUpdateError>;

    /// Set the flags of an existing page table level 3 entry
    ///
    /// ## Safety
    ///
    /// This method is unsafe because changing the flags of a mapping
    /// might result in undefined behavior. For example, setting the
    /// `GLOBAL` and `WRITABLE` flags for a page might result in the corruption
    /// of values stored in that page from processes running in other address
    /// spaces.
    unsafe fn set_flags_p3_entry(
        &mut self,
        page: Page<S, V>,
        flags: PageTableFlags,
    ) -> Result<MapperFlushAll, FlagUpdateError>;

    /// Set the flags of an existing page table level 2 entry
    ///
    /// ## Safety
    ///
    /// This method is unsafe because changing the flags of a mapping
    /// might result in undefined behavior. For example, setting the
    /// `GLOBAL` and `WRITABLE` flags for a page might result in the corruption
    /// of values stored in that page from processes running in other address
    /// spaces.
    unsafe fn set_flags_p2_entry(
        &mut self,
        page: Page<S, V>,
        flags: PageTableFlags,
    ) -> Result<MapperFlushAll, FlagUpdateError>;

    /// Return the frame that the specified page is mapped to.
    ///
    /// This function assumes that the page is mapped to a frame of size `S` and returns an
    /// error otherwise.
    fn translate_page(&self, page: Page<S, V>) -> Result<PhysFrame<S>, TranslateError>;

    /// Maps the given frame to the virtual page with the same address.
    ///
    /// ## Safety
    ///
    /// This is a convenience function that invokes [`Mapper::map_to`] internally, so
    /// all safety requirements of it also apply for this function.
    #[inline]
    unsafe fn identity_map<A>(
        &mut self,
        frame: PhysFrame<S>,
        flags: PageTableFlags,
        frame_allocator: &mut A,
    ) -> Result<MapperFlush<S, V>, MapToError<S>>
    where
        Self: Sized,
        A: FrameAllocator<Size4KiB> + ?Sized,
        S: PageSize,
        Self: Mapper<S, V>,
    {
        let address = VirtAddrGeneric::<V>::try_new_with_validity(frame.start_address().as_u64())
            .map_err(|_| MapToError::AddressNotValid)?;
        let page = Page::containing_address(address);
        unsafe { self.map_to(page, frame, flags, frame_allocator) }
    }
}

/// The result of [`Mapper::clear`], representing either
/// the unmapped frame or the entry data if the frame is not marked as present.
#[derive(Debug)]
#[must_use = "Page table changes must be flushed or ignored if the page is present."]
pub enum UnmappedFrame<S: PageSize, V: VirtAddrValidity = FixedValidity<48>> {
    /// The frame was present before the [`Mapper::clear`] call
    Present {
        /// The physical frame that was unmapped
        frame: PhysFrame<S>,
        /// The flags of the frame that was unmapped
        flags: PageTableFlags,
        /// The changed page, to flush the TLB
        flush: MapperFlush<S, V>,
    },
    /// The frame was not present before the [`Mapper::clear`] call
    NotPresent {
        /// The page table entry
        entry: PageTableEntry,
    },
}

/// This type represents a page whose mapping has changed in the page table.
///
/// The old mapping might be still cached in the translation lookaside buffer (TLB), so it needs
/// to be flushed from the TLB before it's accessed. This type is returned from a function that
/// changed the mapping of a page to ensure that the TLB flush is not forgotten.
#[derive(Debug)]
#[must_use = "Page Table changes must be flushed or ignored."]
#[cfg_attr(
    not(all(feature = "instructions", target_arch = "x86_64")),
    allow(dead_code)
)] // FIXME
pub struct MapperFlush<S: PageSize, V: VirtAddrValidity = FixedValidity<48>>(Page<S, V>);

impl<S: PageSize, V: VirtAddrValidity> MapperFlush<S, V> {
    /// Create a new flush promise
    ///
    /// Note that this method is intended for implementing the [`Mapper`] trait and no other uses
    /// are expected.
    #[inline]
    pub fn new(page: Page<S, V>) -> Self {
        MapperFlush(page)
    }

    /// Flush the page from the TLB to ensure that the newest mapping is used.
    #[cfg(all(feature = "instructions", target_arch = "x86_64"))]
    #[inline]
    pub fn flush(self) {
        crate::instructions::tlb::flush(self.0.start_address());
    }

    /// Don't flush the TLB and silence the “must be used” warning.
    #[inline]
    pub fn ignore(self) {}

    /// Returns the page to be flushed.
    #[inline]
    pub fn page(&self) -> Page<S, V> {
        self.0
    }
}

/// This type represents a change of a page table requiring a complete TLB flush
///
/// The old mapping might be still cached in the translation lookaside buffer (TLB), so it needs
/// to be flushed from the TLB before it's accessed. This type is returned from a function that
/// made the change to ensure that the TLB flush is not forgotten.
#[derive(Debug, Default)]
#[must_use = "Page Table changes must be flushed or ignored."]
pub struct MapperFlushAll(());

impl MapperFlushAll {
    /// Create a new flush promise
    ///
    /// Note that this method is intended for implementing the [`Mapper`] trait and no other uses
    /// are expected.
    #[inline]
    pub fn new() -> Self {
        MapperFlushAll(())
    }

    /// Flush all pages from the TLB to ensure that the newest mapping is used.
    #[cfg(all(feature = "instructions", target_arch = "x86_64"))]
    #[inline]
    pub fn flush_all(self) {
        crate::instructions::tlb::flush_all()
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
    /// The address is not usable by the mapper's root level.
    AddressNotValid,
    /// An upper level page table entry has the `HUGE_PAGE` flag set, which means that the
    /// given page is part of an already mapped huge page.
    ParentEntryHugePage,
    /// The given page is already mapped to a physical frame.
    PageAlreadyMapped(PhysFrame<S>),
}

/// An error indicating that an `unmap` call failed.
#[derive(Debug)]
pub enum UnmapError {
    /// The address is not usable by the mapper's root level.
    AddressNotValid,
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
    /// The address is not usable by the mapper's root level.
    AddressNotValid,
    /// The requested page-table level does not exist for this mapper's root.
    PageTableLevelNotPresent,
    /// The given page is not mapped to a physical frame.
    PageNotMapped,
    /// An upper level page table entry has the `HUGE_PAGE` flag set, which means that the
    /// given page is part of a huge page and can't be freed individually.
    ParentEntryHugePage,
}

/// An error indicating that an `translate` call failed.
#[derive(Debug)]
pub enum TranslateError {
    /// The address is not usable by the mapper's root level.
    AddressNotValid,
    /// The given page is not mapped to a physical frame.
    PageNotMapped,
    /// An upper level page table entry has the `HUGE_PAGE` flag set, which means that the
    /// given page is part of a huge page and can't be freed individually.
    ParentEntryHugePage,
    /// The page table entry for the given page points to an invalid physical address.
    InvalidFrameAddress(PhysAddr),
}

static _ASSERT_OBJECT_SAFE: Option<&(dyn Translate + Sync)> = None;

/// Provides methods for cleaning up unused entries.
pub trait CleanUp {
    /// Removes empty child page tables below the root (P1-P3, and P4 for a five-level root).
    ///
    /// ## Safety
    ///
    /// The caller has to guarantee that it's safe to free page table frames:
    /// All page table frames must only be used once and only in this page table
    /// (e.g. no reference counted page tables or reusing the same page tables for different virtual addresses ranges in the same page table).
    unsafe fn clean_up<D>(&mut self, frame_deallocator: &mut D)
    where
        D: FrameDeallocator<Size4KiB>;

    /// Removes empty child page tables in a certain range.
    /// ```
    /// # use core::ops::RangeInclusive;
    /// # use x86_64::{VirtAddr, structures::paging::{
    /// #    FrameDeallocator, Mapper, Size4KiB, mapper::CleanUp, page::Page,
    /// # }};
    /// # unsafe fn test(page_table: &mut (impl CleanUp + Mapper<Size4KiB>), frame_deallocator: &mut impl FrameDeallocator<Size4KiB>) {
    /// // clean up all page tables in the lower half of the address space
    /// let lower_half = Page::range_inclusive(
    ///     Page::containing_address(VirtAddr::new(0)),
    ///     Page::containing_address(VirtAddr::new(0x0000_7fff_ffff_ffff)),
    /// );
    /// page_table.clean_up_addr_range(lower_half, frame_deallocator);
    /// # }
    /// ```
    ///
    /// ## Safety
    ///
    /// The caller has to guarantee that it's safe to free page table frames:
    /// All page table frames must only be used once and only in this page table
    /// (e.g. no reference counted page tables or reusing the same page tables for different virtual addresses ranges in the same page table).
    ///
    /// The mapper must support the range policy through Mapper<Size4KiB, V>.
    /// Both endpoints must be valid under their address policy and representable by the
    /// mapper's root. A range may span both canonical halves; cleanup skips the root's
    /// canonical hole. For active recursive mappings, the root level, runtime cache and
    /// active paging mode must also continue to satisfy the mapper's construction contract.
    unsafe fn clean_up_addr_range<D, V: VirtAddrValidity>(
        &mut self,
        range: PageRangeInclusive<Size4KiB, V>,
        frame_deallocator: &mut D,
    ) where
        D: FrameDeallocator<Size4KiB>,
        Self: Mapper<Size4KiB, V>;
}

#[inline]
#[cfg(test)]
pub(crate) fn cleanup_table_address<V: VirtAddrValidity>(
    root_level: PageTableLevel,
    level: PageTableLevel,
    start: VirtAddrGeneric<V>,
) -> VirtAddrGeneric<V> {
    if level == root_level {
        VirtAddrGeneric::zero()
    } else {
        // SAFETY: level is alway not root level, so the alignment is always
        // less than the half of the address space.
        unsafe { start.align_down_u64(level.table_address_space_alignment()) }
    }
}

#[inline]
#[cfg(any(test, all(feature = "virt_addr_rt", target_arch = "x86_64")))]
pub(crate) fn cleanup_entry_address<V: VirtAddrValidity>(
    table_address: VirtAddrGeneric<V>,
    offset: u64,
) -> Option<VirtAddrGeneric<V>> {
    VirtAddrGeneric::<V>::forward_checked_u64(table_address, offset)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::addr::VirtAddr;

    #[test]
    fn cleanup_root_addresses_use_zero_base_and_canonical_forwarding() {
        let table_addr = cleanup_table_address(
            PageTableLevel::Four,
            PageTableLevel::Four,
            VirtAddr::new(0xffff_8000_0000_0000),
        );
        assert_eq!(table_addr.as_u64(), 0);

        let entry_addr = cleanup_entry_address(table_addr, 256 * (1 << 39)).unwrap();
        assert_eq!(entry_addr.as_u64(), 0xffff_8000_0000_0000);
    }

    #[cfg(feature = "virt_addr_57")]
    #[test]
    fn cleanup_l5_root_addresses_canonicalize_high_half() {
        type V57 = crate::addr::FixedValidity<57>;

        let table_addr = cleanup_table_address(
            PageTableLevel::Five,
            PageTableLevel::Five,
            VirtAddrGeneric::<V57>::new(0xff00_0000_0000_0000),
        );
        assert_eq!(table_addr.as_u64(), 0);

        let entry_addr = cleanup_entry_address(table_addr, 256 * (1 << 48)).unwrap();
        assert_eq!(entry_addr.as_u64(), 0xff00_0000_0000_0000);
    }
}
