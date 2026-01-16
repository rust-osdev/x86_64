//! Display adapters for [`MappedPageTable`].

use core::fmt::{self, Write};

use super::range_iter::{MappedPageRangeInclusive, MappedPageRangeInclusiveItem};
use super::{MappedPageTable, PageTableFrameMapping};
use crate::structures::paging::frame::PhysFrameRangeInclusive;
use crate::structures::paging::page::PageRangeInclusive;
use crate::structures::paging::{PageSize, PageTableFlags};

impl<P: PageTableFrameMapping> MappedPageTable<'_, P> {
    /// Display the page table mappings as a human-readable table.
    ///
    /// This method returns an object that implements [`fmt::Display`].
    /// For details, see [`MappedPageTableDisplay`].
    ///
    /// # Examples
    ///
    /// ```ignore-i686
    /// use x86_64::structures::paging::MappedPageTable;
    ///
    /// # let level_4_table = &mut x86_64::structures::paging::page_table::PageTable::new();
    /// # let phys_offset = x86_64::VirtAddr::zero();
    /// let page_table = unsafe { MappedPageTable::from_phys_offset(level_4_table, phys_offset) };
    ///
    /// println!("{}", page_table.display());
    /// ```
    ///
    /// [`MappedPageTableDisplay`]: Display
    pub fn display(&self) -> Display<'_, P> {
        Display { page_table: self }
    }
}

/// [`Display`] adapter for [`MappedPageTable`].
///
/// This struct formats as a human-readable version of the page table mappings when used with [`format_args!`] and `{}`.
/// It is created using [`MappedPageTable::display`].
///
/// This struct also supports formatting with the alternate (`#`) flag for aligned columns with table headers.
///
/// Note that the [`PRESENT`] flag is not listed explicitly, since only present mappings are formatted.
///
/// # Examples
///
/// ```ignore-i686
/// use x86_64::structures::paging::MappedPageTable;
///
/// # let level_4_table = &mut x86_64::structures::paging::page_table::PageTable::new();
/// # let phys_offset = x86_64::VirtAddr::zero();
/// let page_table = unsafe { MappedPageTable::from_phys_offset(level_4_table, phys_offset) };
///
/// println!("{}", page_table.display());
/// ```
///
/// This is how a formatted table looks like:
///
/// ```text
/// 100000-101000 100000-101000 WRITABLE | ACCESSED | DIRTY
/// 101000-103000 101000-103000 WRITABLE | ACCESSED
/// 103000-105000 103000-105000 WRITABLE
/// 105000-106000 105000-106000 WRITABLE | ACCESSED
/// 106000-107000 106000-107000 WRITABLE
/// 107000-10d000 107000-10d000 WRITABLE | ACCESSED
/// 10d000-111000 10d000-111000 WRITABLE
/// 111000-112000 111000-112000 WRITABLE | ACCESSED
/// 112000-114000 112000-114000 WRITABLE
/// 114000-118000 114000-118000 WRITABLE | ACCESSED
/// 118000-119000 118000-119000 WRITABLE
/// 119000-11a000 119000-11a000 WRITABLE | ACCESSED
/// 11a000-11b000 11a000-11b000 WRITABLE
/// 11b000-11c000 11b000-11c000 WRITABLE | ACCESSED | DIRTY
/// 11c000-120000 11c000-120000 WRITABLE | ACCESSED
/// 120000-121000 120000-121000 WRITABLE
/// 121000-122000 121000-122000 WRITABLE | ACCESSED | DIRTY
/// 122000-123000 122000-123000 WRITABLE
/// 123000-124000 123000-124000 WRITABLE | ACCESSED | DIRTY
/// 124000-125000 124000-125000 WRITABLE
/// ffffff8000000000-ffffff8000001000 11f000-120000 WRITABLE | ACCESSED
/// ffffff8000001000-ffffff8000002000 120000-121000 WRITABLE
/// ffffffffc0000000-ffffffffc0001000 11e000-11f000 WRITABLE | ACCESSED
/// ffffffffffe00000-ffffffffffe01000 11d000-11e000 WRITABLE | ACCESSED
/// fffffffffffff000-                 11c000-11d000 WRITABLE
/// ```
///
/// This is how a table formatted with the alternate (`#`) flag looks like:
///
/// ```text
/// size   len                   virtual address                  physical address flags
/// 4KiB     1           100000-          101000                   identity-mapped WRITABLE | ACCESSED | DIRTY
/// 4KiB     2           101000-          103000                   identity-mapped WRITABLE | ACCESSED
/// 4KiB     2           103000-          105000                   identity-mapped WRITABLE
/// 4KiB     1           105000-          106000                   identity-mapped WRITABLE | ACCESSED
/// 4KiB     1           106000-          107000                   identity-mapped WRITABLE
/// 4KiB     7           107000-          10e000                   identity-mapped WRITABLE | ACCESSED
/// 4KiB     3           10e000-          111000                   identity-mapped WRITABLE
/// 4KiB     1           111000-          112000                   identity-mapped WRITABLE | ACCESSED
/// 4KiB     2           112000-          114000                   identity-mapped WRITABLE
/// 4KiB     4           114000-          118000                   identity-mapped WRITABLE | ACCESSED
/// 4KiB     1           118000-          119000                   identity-mapped WRITABLE
/// 4KiB     1           119000-          11a000                   identity-mapped WRITABLE | ACCESSED
/// 4KiB     1           11a000-          11b000                   identity-mapped WRITABLE
/// 4KiB     1           11b000-          11c000                   identity-mapped WRITABLE | ACCESSED | DIRTY
/// 4KiB     5           11c000-          121000                   identity-mapped WRITABLE | ACCESSED
/// 4KiB     1           121000-          122000                   identity-mapped WRITABLE | ACCESSED | DIRTY
/// 4KiB     1           122000-          123000                   identity-mapped WRITABLE
/// 4KiB     1           123000-          124000                   identity-mapped WRITABLE | ACCESSED | DIRTY
/// 4KiB     1           124000-          125000                   identity-mapped WRITABLE
/// 4KiB     1 ffffff8000000000-ffffff8000001000           11f000-          120000 WRITABLE | ACCESSED
/// 4KiB     1 ffffff8000001000-ffffff8000002000           120000-          121000 WRITABLE
/// 4KiB     1 ffffffffc0000000-ffffffffc0001000           11e000-          11f000 WRITABLE | ACCESSED
/// 4KiB     1 ffffffffffe00000-ffffffffffe01000           11d000-          11e000 WRITABLE | ACCESSED
/// 4KiB     1 fffffffffffff000-                           11c000-          11d000 WRITABLE
/// ```
///
/// [`Display`]: fmt::Display
/// [`PRESENT`]: PageTableFlags::PRESENT
pub struct Display<'a, P: PageTableFrameMapping> {
    page_table: &'a MappedPageTable<'a, P>,
}

impl<P: PageTableFrameMapping + fmt::Debug> fmt::Debug for Display<'_, P> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self.page_table, f)
    }
}

impl<P: PageTableFrameMapping> fmt::Display for Display<'_, P> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut has_fields = false;

        if f.alternate() {
            write!(
                f,
                "size {:>5} {:>33} {:>33} flags",
                "len", "virtual address", "physical address"
            )?;
            has_fields = true;
        }

        for mapped_page_range in self.page_table.range_iter() {
            if has_fields {
                f.write_char('\n')?;
            }
            fmt::Display::fmt(&mapped_page_range.display(), f)?;

            has_fields = true;
        }

        Ok(())
    }
}

/// A helper struct for formatting a [`MappedPageRangeInclusiveItem`] as a table row.
struct MappedPageRangeInclusiveItemDisplay<'a> {
    item: &'a MappedPageRangeInclusiveItem,
}

impl MappedPageRangeInclusiveItem {
    fn display(&self) -> MappedPageRangeInclusiveItemDisplay<'_> {
        MappedPageRangeInclusiveItemDisplay { item: self }
    }
}

impl fmt::Display for MappedPageRangeInclusiveItemDisplay<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.item {
            MappedPageRangeInclusiveItem::Size4KiB(range) => fmt::Display::fmt(&range.display(), f),
            MappedPageRangeInclusiveItem::Size2MiB(range) => fmt::Display::fmt(&range.display(), f),
            MappedPageRangeInclusiveItem::Size1GiB(range) => fmt::Display::fmt(&range.display(), f),
        }
    }
}

/// A helper struct for formatting a [`MappedPageRangeInclusive`] as a table row.
struct MappedPageRangeInclusiveDisplay<'a, S: PageSize> {
    range: &'a MappedPageRangeInclusive<S>,
}

impl<S: PageSize> MappedPageRangeInclusive<S> {
    fn display(&self) -> MappedPageRangeInclusiveDisplay<'_, S> {
        MappedPageRangeInclusiveDisplay { range: self }
    }
}

impl<S: PageSize> fmt::Display for MappedPageRangeInclusiveDisplay<'_, S> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if f.alternate() {
            let size = S::DEBUG_STR;
            write!(f, "{size} ")?;

            let len = self.range.len();
            write!(f, "{len:5} ")?;
        }

        let page_range = self.range.page_range();
        // Forward the formatter's options such as the alternate (`#`) flag.
        fmt::Pointer::fmt(&page_range.display(), f)?;
        f.write_char(' ')?;

        if f.alternate() && self.range.is_identity_mapped() {
            write!(f, "{:>33}", "identity-mapped")?;
        } else {
            let frame_range = self.range.frame_range();
            // Forward the formatter's options such as the alternate (`#`) flag.
            fmt::Pointer::fmt(&frame_range.display(), f)?;
        }
        f.write_char(' ')?;

        // Every entry is present, don't print it explicitly.
        let flags = self.range.flags() - PageTableFlags::PRESENT;
        // Format the flags as `A | B` instead of `Flags(A | B)`.
        bitflags::parser::to_writer(&flags, &mut *f)?;

        Ok(())
    }
}

/// A helper type for formatting an address range as [`fmt::Pointer`].
struct AddressRangeDisplay<T> {
    start: T,
    end: Option<T>,
}

impl<S: PageSize> PageRangeInclusive<S> {
    fn display(&self) -> AddressRangeDisplay<u64> {
        let start = self.start.start_address().as_u64();
        let end = self.end.start_address().as_u64().checked_add(S::SIZE);
        AddressRangeDisplay { start, end }
    }
}

impl<S: PageSize> PhysFrameRangeInclusive<S> {
    fn display(&self) -> AddressRangeDisplay<u64> {
        let start = self.start.start_address().as_u64();
        let end = self.end.start_address().as_u64().checked_add(S::SIZE);
        AddressRangeDisplay { start, end }
    }
}

impl<T: fmt::LowerHex> fmt::Pointer for AddressRangeDisplay<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { start, end } = self;
        match (end, f.alternate()) {
            (Some(end), false) => write!(f, "{start:x}-{end:x}"),
            (Some(end), true) => write!(f, "{start:16x}-{end:16x}"),
            (None, false) => write!(f, "{start:x}-{:16}", ""),
            (None, true) => write!(f, "{start:16x}-{:16}", ""),
        }
    }
}
