//! Range iterator over [`MappedPageTable`]s.
//!
//! The main type of this module is [`MappedPageTableRangeInclusiveIter`] returning [`MappedPageRangeInclusiveItem`]s.

use core::convert::TryFrom;
use core::fmt;
use core::ops::RangeInclusive;

use super::iter::{MappedPage, MappedPageItem, MappedPageTableIter};
use super::{MappedPageTable, PageTableFrameMapping};
use crate::structures::paging::frame::PhysFrameRangeInclusive;
use crate::structures::paging::page::PageRangeInclusive;
use crate::structures::paging::{
    PageSize, PageTableFlags, PhysFrame, Size1GiB, Size2MiB, Size4KiB,
};

/// A contiguous range of [`MappedPage`]s.
pub struct MappedPageRangeInclusive<S: PageSize = Size4KiB> {
    page_range: PageRangeInclusive<S>,
    frame_start: PhysFrame<S>,
    flags: PageTableFlags,
}

impl<S: PageSize> MappedPageRangeInclusive<S> {
    /// Returns the page range.
    pub fn page_range(&self) -> PageRangeInclusive<S> {
        self.page_range.clone()
    }

    /// Returns the frame range.
    pub fn frame_range(&self) -> PhysFrameRangeInclusive<S> {
        let start = self.frame_start;
        let end = start + self.page_range.len() - 1;
        PhysFrameRangeInclusive { start, end }
    }

    /// Returns the page table flags.
    pub fn flags(&self) -> PageTableFlags {
        self.flags
    }

    /// Returns the number of pages in the range.
    pub fn len(&self) -> u64 {
        self.page_range.len()
    }

    /// Returns whether this is an identity mapping.
    pub fn is_identity_mapped(&self) -> bool {
        self.page_range.start.start_address().as_u64() == self.frame_start.start_address().as_u64()
    }
}

impl<S: PageSize> TryFrom<RangeInclusive<MappedPage<S>>> for MappedPageRangeInclusive<S> {
    /// The type returned in the event of a conversion error.
    type Error = TryFromMappedPageError;

    /// Tries to create a mapped page range from a range of mapped pages.
    ///
    /// This returns an error if the number of pages is not equal to the number of frames.
    /// This also returns an error if the page table flags are not equal.
    fn try_from(value: RangeInclusive<MappedPage<S>>) -> Result<Self, Self::Error> {
        let page_range = PageRangeInclusive {
            start: value.start().page,
            end: value.end().page,
        };

        let frame_start = value.start().frame;
        let frame_range = PhysFrameRangeInclusive {
            start: frame_start,
            end: value.end().frame,
        };
        if page_range.len() != frame_range.len() {
            return Err(TryFromMappedPageError);
        }

        let flags = value.start().flags;
        if flags != value.end().flags {
            return Err(TryFromMappedPageError);
        }

        Ok(Self {
            page_range,
            frame_start,
            flags,
        })
    }
}

/// A [`MappedPageRangeInclusive`] of any size.
pub enum MappedPageRangeInclusiveItem {
    /// The [`MappedPageRangeInclusive`] has a size of 4KiB.
    Size4KiB(MappedPageRangeInclusive<Size4KiB>),

    /// The [`MappedPageRangeInclusive`] has a size of 2MiB.
    Size2MiB(MappedPageRangeInclusive<Size2MiB>),

    /// The [`MappedPageRangeInclusive`] has a size of 1GiB.
    Size1GiB(MappedPageRangeInclusive<Size1GiB>),
}

impl TryFrom<RangeInclusive<MappedPageItem>> for MappedPageRangeInclusiveItem {
    /// The type returned in the event of a conversion error.
    type Error = TryFromMappedPageError;

    /// Tries to create a mapped page range from a range of mapped pages.
    ///
    /// This returns an error if the number of pages is not equal to the number of frames
    /// or when the page sizes are not equal.
    /// This also returns an error if the page table flags are not equal.
    fn try_from(value: RangeInclusive<MappedPageItem>) -> Result<Self, Self::Error> {
        match (*value.start(), *value.end()) {
            (MappedPageItem::Size4KiB(start), MappedPageItem::Size4KiB(end)) => {
                let range = MappedPageRangeInclusive::try_from(start..=end)?;
                Ok(Self::Size4KiB(range))
            }
            (MappedPageItem::Size2MiB(start), MappedPageItem::Size2MiB(end)) => {
                let range = MappedPageRangeInclusive::try_from(start..=end)?;
                Ok(Self::Size2MiB(range))
            }
            (MappedPageItem::Size1GiB(start), MappedPageItem::Size1GiB(end)) => {
                let range = MappedPageRangeInclusive::try_from(start..=end)?;
                Ok(Self::Size1GiB(range))
            }
            (_, _) => Err(TryFromMappedPageError),
        }
    }
}

/// The error type returned when a conversion from a range of mapped pages to mapped page range fails.
#[derive(PartialEq, Eq, Clone, Debug)]
pub struct TryFromMappedPageError;

impl fmt::Display for TryFromMappedPageError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("provided mapped pages were not compatible")
    }
}

/// A range iterator over a [`MappedPageTable`].
///
/// This iterator returns every contiguous range of page mappings as a [`MappedPageRangeInclusiveItem`].
///
/// This struct is created by [`MappedPageTable::range_iter`].
///
/// # Current implementation
///
/// Performs a depth-fist search for the next contiguous range of [`MappedPageItem`]s and returns it as a [`MappedPageRangeInclusiveItem`].
pub struct MappedPageTableRangeInclusiveIter<'a, P: PageTableFrameMapping> {
    iter: MappedPageTableIter<'a, P>,
    next_start: Option<MappedPageItem>,
}

impl<P: PageTableFrameMapping> MappedPageTable<'_, P> {
    /// Returns an iterator over the page table's [`MappedPageRangeInclusiveItem`]s.
    pub(super) fn range_iter(&self) -> MappedPageTableRangeInclusiveIter<'_, &P> {
        MappedPageTableRangeInclusiveIter {
            iter: self.iter(),
            next_start: None,
        }
    }
}

impl<P: PageTableFrameMapping> Iterator for MappedPageTableRangeInclusiveIter<'_, P> {
    type Item = MappedPageRangeInclusiveItem;

    fn next(&mut self) -> Option<Self::Item> {
        // Take the start item from last iteration or get a new one.
        let start = self.next_start.take().or_else(|| self.iter.next())?;

        // Find the end of the current contiguous range.
        let mut end = start;
        for mapped_page in &mut self.iter {
            if mapped_page != end + 1 {
                // The current item is no longer contiguous to the current range,
                // so save it for next time.
                self.next_start = Some(mapped_page);
                break;
            }

            end = mapped_page;
        }

        let range = MappedPageRangeInclusiveItem::try_from(start..=end).unwrap();
        Some(range)
    }
}
