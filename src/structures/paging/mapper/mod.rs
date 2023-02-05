//! Abstractions for reading and modifying the mapping of pages.

pub use self::mapped_page_table::{MappedPageTable, PageTableFrameMapping};
#[cfg(target_pointer_width = "64")]
pub use self::offset_page_table::OffsetPageTable;
#[cfg(feature = "instructions")]
pub use self::recursive_page_table::{InvalidPageTable, RecursivePageTable};

use core::convert::Infallible;

#[cfg(feature = "experimental")]
use crate::structures::paging::{frame::PhysFrameRange, page::PageRange};
use crate::structures::paging::{
    frame_alloc::{FrameAllocator, FrameDeallocator},
    page::PageRangeInclusive,
    page_table::PageTableFlags,
    Page, PageSize, PhysFrame, Size1GiB, Size2MiB, Size4KiB,
};
use crate::{PhysAddr, VirtAddr};

use super::page_table::FrameError;

mod mapped_page_table;
mod offset_page_table;
#[cfg(feature = "instructions")]
mod recursive_page_table;

/// An empty convencience trait that requires the `Mapper` trait for all page sizes.
pub trait MapperAllSizes: Mapper<Size4KiB> + Mapper<Size2MiB> + Mapper<Size1GiB> {}

impl<T> MapperAllSizes for T where T: Mapper<Size4KiB> + Mapper<Size2MiB> + Mapper<Size1GiB> {}

/// Provides methods for translating virtual addresses.
pub trait Translate {
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
    /// [`translate`](Translate::translate) method.
    #[inline]
    fn translate_addr(&self, addr: VirtAddr) -> Option<PhysAddr> {
        match self.translate(addr) {
            TranslateResult::NotMapped | TranslateResult::InvalidFrameAddress(_) => None,
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
        /// The offset whithin the mapped frame.
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

/// A trait for common page table operations on pages of size `S`.
pub trait Mapper<S: PageSize> {
    /// Creates a new mapping in the page table.
    ///
    /// This function might need additional physical frames to create new page tables. These
    /// frames are allocated from the `allocator` argument. At most three frames are required.
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
    ///
    /// # Examples
    ///
    /// Create a USER_ACCESSIBLE mapping:
    ///
    /// ```
    /// # #[cfg(feature = "instructions")]
    /// # use x86_64::structures::paging::{
    /// #    Mapper, Page, PhysFrame, FrameAllocator,
    /// #    Size4KiB, OffsetPageTable, page_table::PageTableFlags
    /// # };
    /// # #[cfg(feature = "instructions")]
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
        page: Page<S>,
        frame: PhysFrame<S>,
        flags: PageTableFlags,
        frame_allocator: &mut A,
    ) -> Result<MapperFlush<S>, MapToError<S>>
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

    #[cfg(feature = "experimental")]
    /// Maps the given range of frames to the range of virtual pages.
    ///
    /// ## Safety
    ///
    /// This is a convencience function that invokes [`Mapper::map_to`] internally, so
    /// all safety requirements of it also apply for this function.
    ///
    /// ## Panics
    ///
    /// This function panics if the amount of pages does not equal the amount of frames.
    ///
    /// ## Errors
    ///
    /// If an error occurs half-way through a [`MapperFlushRange<S>`] is returned that contains the frames that were successfully mapped.
    #[inline]
    unsafe fn map_to_range<A>(
        &mut self,
        pages: PageRange<S>,
        frames: PhysFrameRange<S>,
        flags: PageTableFlags,
        frame_allocator: &mut A,
    ) -> Result<MapperFlushRange<S>, (MapToError<S>, MapperFlushRange<S>)>
    where
        Self: Sized,
        A: FrameAllocator<Size4KiB> + ?Sized,
    {
        let parent_table_flags = flags
            & (PageTableFlags::PRESENT
                | PageTableFlags::WRITABLE
                | PageTableFlags::USER_ACCESSIBLE);

        unsafe {
            self.map_to_range_with_table_flags(
                pages,
                frames,
                flags,
                parent_table_flags,
                frame_allocator,
            )
        }
    }

    /// Creates a new mapping in the page table.
    ///
    /// This function might need additional physical frames to create new page tables. These
    /// frames are allocated from the `allocator` argument. At most three frames are required.
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
    ///
    /// # Examples
    ///
    /// Create USER_ACCESSIBLE | NO_EXECUTE | NO_CACHE mapping and update
    /// the top hierarchy only with USER_ACCESSIBLE:
    ///
    /// ```
    /// # #[cfg(feature = "instructions")]
    /// # use x86_64::structures::paging::{
    /// #    Mapper, PhysFrame, Page, FrameAllocator,
    /// #    Size4KiB, OffsetPageTable, page_table::PageTableFlags
    /// # };
    /// # #[cfg(feature = "instructions")]
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
        page: Page<S>,
        frame: PhysFrame<S>,
        flags: PageTableFlags,
        parent_table_flags: PageTableFlags,
        frame_allocator: &mut A,
    ) -> Result<MapperFlush<S>, MapToError<S>>
    where
        Self: Sized,
        A: FrameAllocator<Size4KiB> + ?Sized;

    #[cfg(feature = "experimental")]
    /// Maps the given range of frames to the range of virtual pages.
    ///
    /// ## Safety
    ///
    /// This is a convencience function that invokes [`Mapper::map_to_with_table_flags`] internally, so
    /// all safety requirements of it also apply for this function.
    ///
    /// ## Panics
    ///
    /// This function panics if the amount of pages does not equal the amount of frames.
    ///
    /// ## Errors
    ///
    /// If an error occurs half-way through a [`MapperFlushRange<S>`] is returned that contains the frames that were successfully mapped.
    unsafe fn map_to_range_with_table_flags<A>(
        &mut self,
        pages: PageRange<S>,
        frames: PhysFrameRange<S>,
        flags: PageTableFlags,
        parent_table_flags: PageTableFlags,
        frame_allocator: &mut A,
    ) -> Result<MapperFlushRange<S>, (MapToError<S>, MapperFlushRange<S>)>
    where
        Self: Sized,
        A: FrameAllocator<Size4KiB> + ?Sized,
    {
        assert_eq!(pages.count(), frames.count());

        pages
            .zip(frames)
            .try_for_each(|(page, frame)| {
                unsafe {
                    self.map_to_with_table_flags(
                        page,
                        frame,
                        flags,
                        parent_table_flags,
                        frame_allocator,
                    )
                }
                .map(|_| ())
                .map_err(|e| {
                    (
                        e,
                        MapperFlushRange::new(PageRange {
                            start: pages.start,
                            end: page,
                        }),
                    )
                })
            })
            .map(|_| MapperFlushRange::new(pages))
    }

    #[cfg(feature = "experimental")]
    /// Maps frames from the allocator to the given range of virtual pages.
    ///
    /// ## Safety
    ///
    /// This function invokes [`Mapper::map_to_with_table_flags`] internally, so
    /// all safety requirements of it also apply for this function.
    ///
    /// ## Errors
    ///
    /// If an error occurs half-way through a [`MapperFlushRange<S>`] is returned that contains the frames that were successfully mapped.
    unsafe fn map_range_with_table_flags<A>(
        &mut self,
        mut pages: PageRange<S>,
        flags: PageTableFlags,
        parent_table_flags: PageTableFlags,
        frame_allocator: &mut A,
    ) -> Result<MapperFlushRange<S>, (MapToError<S>, MapperFlushRange<S>)>
    where
        Self: Sized,
        A: FrameAllocator<Size4KiB> + FrameAllocator<S> + ?Sized,
    {
        pages
            .try_for_each(|page| {
                let frame = frame_allocator
                    .allocate_frame()
                    .ok_or((MapToError::FrameAllocationFailed, page))?;

                unsafe {
                    // SAFETY: frame was just freshly allocated and therefore can't cause aliasing issues
                    self.map_to_with_table_flags(
                        page,
                        frame,
                        flags,
                        parent_table_flags,
                        frame_allocator,
                    )
                }
                .map(|_| ())
                .map_err(|e| (e, page))
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

    #[cfg(feature = "experimental")]
    /// Maps frames from the allocator to the given range of virtual pages.
    ///
    /// ## Safety
    ///
    /// This function invokes [`Mapper::map_to_with_table_flags`] internally, so
    /// all safety requirements of it also apply for this function.
    ///
    /// ## Errors
    ///
    /// If an error occurs half-way through a [`MapperFlushRange<S>`] is returned that contains the frames that were successfully mapped.
    #[inline]
    unsafe fn map_range<A>(
        &mut self,
        pages: PageRange<S>,
        flags: PageTableFlags,
        frame_allocator: &mut A,
    ) -> Result<MapperFlushRange<S>, (MapToError<S>, MapperFlushRange<S>)>
    where
        Self: Sized,
        A: FrameAllocator<Size4KiB> + FrameAllocator<S> + ?Sized,
    {
        let parent_table_flags = flags
            & (PageTableFlags::PRESENT
                | PageTableFlags::WRITABLE
                | PageTableFlags::USER_ACCESSIBLE);

        unsafe {
            self.map_range_with_table_flags(pages, flags, parent_table_flags, frame_allocator)
        }
    }

    /// Removes a mapping from the page table and returns the frame that used to be mapped.
    ///
    /// Note that no page tables or pages are deallocated.
    fn unmap(&mut self, page: Page<S>) -> Result<(PhysFrame<S>, MapperFlush<S>), UnmapError>;

    #[cfg(feature = "experimental")]
    /// Removes a range of mapping from the page table and deallocate the frames that used to be mapped.
    ///
    /// Note that no page tables or pages are deallocated.
    ///
    /// ## Errors
    /// If an error occurs half-way through a [`MapperFlushRange<S>`] is returned that contains the pages that were successfully unmapped.
    ///
    /// ## Safety
    /// The caller has to guarantee that it's safe to deallocate frames:
    /// All unmapped frames must only be in this page table.
    unsafe fn unmap_range<D>(
        &mut self,
        pages: PageRange<S>,
        deallocator: &mut D,
    ) -> Result<MapperFlushRange<S>, (UnmapError, MapperFlushRange<S>)>
    where
        Self: Sized,
        D: FrameDeallocator<S> + ?Sized,
    {
        pages
            .clone()
            .try_for_each(|page| {
                let (frame, _) = self.unmap(page).map_err(|e| {
                    (
                        e,
                        MapperFlushRange::new(PageRange {
                            start: pages.start,
                            end: page,
                        }),
                    )
                })?;
                unsafe {
                    // SAFETY: the page has been unmapped so the frame is unused
                    deallocator.deallocate_frame(frame);
                }
                Ok(())
            })
            .map(|_| MapperFlushRange::new(pages))
    }

    /// Updates the flags of an existing mapping.
    ///
    /// ## Safety
    ///
    /// This method is unsafe because changing the flags of a mapping
    /// might result in undefined behavior. For example, setting the
    /// `GLOBAL` and `MUTABLE` flags for a page might result in the corruption
    /// of values stored in that page from processes running in other address
    /// spaces.
    unsafe fn update_flags(
        &mut self,
        page: Page<S>,
        flags: PageTableFlags,
    ) -> Result<MapperFlush<S>, FlagUpdateError>;

    #[cfg(feature = "experimental")]
    /// Updates the flags of a range of existing mappings.
    ///
    /// ## Safety
    ///
    /// This method is unsafe because changing the flags of a mapping
    /// might result in undefined behavior. For example, setting the
    /// `GLOBAL` and `WRITABLE` flags for a page might result in the corruption
    /// of values stored in that page from processes running in other address
    /// spaces.
    ///
    /// ## Errors
    /// If an error occurs half-way through a [`MapperFlushRange<S>`] is returned that contains the pages that were successfully updated.
    unsafe fn update_flags_range(
        &mut self,
        pages: PageRange<S>,
        flags: PageTableFlags,
    ) -> Result<MapperFlushRange<S>, (FlagUpdateError, MapperFlushRange<S>)> {
        pages
            .clone()
            .try_for_each(|page| {
                unsafe { self.update_flags(page, flags) }
                    .map(|_| ())
                    .map_err(|e| {
                        (
                            e,
                            MapperFlushRange::new(PageRange {
                                start: pages.start,
                                end: page,
                            }),
                        )
                    })
            })
            .map(|_| MapperFlushRange::new(pages))
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
        page: Page<S>,
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
        page: Page<S>,
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
        page: Page<S>,
        flags: PageTableFlags,
    ) -> Result<MapperFlushAll, FlagUpdateError>;

    /// Return the frame that the specified page is mapped to.
    ///
    /// This function assumes that the page is mapped to a frame of size `S` and returns an
    /// error otherwise.
    fn translate_page(&self, page: Page<S>) -> Result<PhysFrame<S>, TranslateError>;

    /// Maps the given frame to the virtual page with the same address.
    ///
    /// ## Safety
    ///
    /// This is a convencience function that invokes [`Mapper::map_to`] internally, so
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
        A: FrameAllocator<Size4KiB> + ?Sized,
        S: PageSize,
        Self: Mapper<S>,
    {
        let page = Page::containing_address(VirtAddr::new(frame.start_address().as_u64()));
        unsafe { self.map_to(page, frame, flags, frame_allocator) }
    }

    #[cfg(feature = "experimental")]
    /// Maps the given range of frames to the range of virtual pages with the same address.
    ///
    /// ## Safety
    ///
    /// This is a convencience function that invokes [`Mapper::map_to_range`] internally, so
    /// all safety requirements of it also apply for this function.
    #[inline]
    unsafe fn identity_map_range<A>(
        &mut self,
        frames: PhysFrameRange<S>,
        flags: PageTableFlags,
        frame_allocator: &mut A,
    ) -> Result<MapperFlushRange<S>, (MapToError<S>, MapperFlushRange<S>)>
    where
        Self: Sized,
        A: FrameAllocator<Size4KiB> + ?Sized,
        S: PageSize,
        Self: Mapper<S>,
    {
        let start = Page::containing_address(VirtAddr::new(frames.start.start_address().as_u64()));
        let end = Page::containing_address(VirtAddr::new(frames.end.start_address().as_u64()));
        let pages = PageRange { start, end };
        unsafe { self.map_to_range(pages, frames, flags, frame_allocator) }
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
    ///
    /// Note that this method is intended for implementing the [`Mapper`] trait and no other uses
    /// are expected.
    #[inline]
    pub fn new(page: Page<S>) -> Self {
        MapperFlush(page)
    }

    /// Flush the page from the TLB to ensure that the newest mapping is used.
    #[cfg(feature = "instructions")]
    #[inline]
    pub fn flush(self) {
        crate::instructions::tlb::flush(self.0.start_address());
    }

    /// Don't flush the TLB and silence the “must be used” warning.
    #[inline]
    pub fn ignore(self) {}
}

#[cfg(feature = "experimental")]
/// This type represents a range of pages whose mappings have changed in the page table.
///
/// The old mappings might be still cached in the translation lookaside buffer (TLB), so they need
/// to be flushed from the TLB before they're accessed. This type is returned from a function that
/// changed the mappings of a range of pages to ensure that the TLB flush is not forgotten.
#[derive(Debug)]
#[must_use = "Page Table changes must be flushed or ignored."]
pub struct MapperFlushRange<S: PageSize>(PageRange<S>);

#[cfg(feature = "experimental")]
impl<S: PageSize> MapperFlushRange<S> {
    /// Create a new flush promise
    #[inline]
    fn new(pages: PageRange<S>) -> Self {
        MapperFlushRange(pages)
    }

    /// Create a new empty flush promise
    #[inline]
    fn empty() -> Self {
        MapperFlushRange::new(PageRange {
            start: Page::containing_address(VirtAddr::zero()),
            end: Page::containing_address(VirtAddr::zero()),
        })
    }

    /// Flush the pages from the TLB to ensure that the newest mapping is used.
    #[cfg(feature = "instructions")]
    #[inline]
    pub fn flush_range(self) {
        for page in self.0 {
            crate::instructions::tlb::flush(page.start_address())
        }
    }

    /// Flush all pages from the TLB to ensure that the newest mapping is used.
    #[cfg(feature = "instructions")]
    #[inline]
    pub fn flush_all(self) {
        crate::instructions::tlb::flush_all()
    }

    /// Don't flush the TLB and silence the “must be used” warning.
    #[inline]
    pub fn ignore(self) {}

    /// Get the range of changed pages.
    pub fn pages(&self) -> PageRange<S> {
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
    #[cfg(feature = "instructions")]
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

impl From<FrameError> for UnmapError {
    fn from(e: FrameError) -> Self {
        match e {
            FrameError::FrameNotPresent => Self::PageNotMapped,
            FrameError::HugeFrame => Self::ParentEntryHugePage,
        }
    }
}

impl From<Infallible> for UnmapError {
    fn from(i: Infallible) -> Self {
        match i {}
    }
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

impl From<FrameError> for FlagUpdateError {
    fn from(e: FrameError) -> Self {
        match e {
            FrameError::FrameNotPresent => Self::PageNotMapped,
            FrameError::HugeFrame => Self::ParentEntryHugePage,
        }
    }
}

impl From<Infallible> for FlagUpdateError {
    fn from(i: Infallible) -> Self {
        match i {}
    }
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

static _ASSERT_OBJECT_SAFE: Option<&(dyn Translate + Sync)> = None;

/// Provides methods for cleaning up unused entries.
pub trait CleanUp {
    /// Remove all empty P1-P3 tables
    ///
    /// ## Safety
    ///
    /// The caller has to guarantee that it's safe to free page table frames:
    /// All page table frames must only be used once and only in this page table
    /// (e.g. no reference counted page tables or reusing the same page tables for different virtual addresses ranges in the same page table).
    unsafe fn clean_up<D>(&mut self, frame_deallocator: &mut D)
    where
        D: FrameDeallocator<Size4KiB>;

    /// Remove all empty P1-P3 tables in a certain range
    /// ```
    /// # use core::ops::RangeInclusive;
    /// # use x86_64::{VirtAddr, structures::paging::{
    /// #    FrameDeallocator, Size4KiB, mapper::CleanUp, page::Page,
    /// # }};
    /// # unsafe fn test(page_table: &mut impl CleanUp, frame_deallocator: &mut impl FrameDeallocator<Size4KiB>) {
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
    unsafe fn clean_up_addr_range<D>(
        &mut self,
        range: PageRangeInclusive,
        frame_deallocator: &mut D,
    ) where
        D: FrameDeallocator<Size4KiB>;
}
