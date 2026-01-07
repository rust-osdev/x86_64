//! Iterator over [`MappedPageTable`]s.
//!
//! The main type of this module is [`MappedPageTableIter`] returning [`MappedPageItem`]s.

use core::ops::Add;

use super::{MappedPageTable, PageTableFrameMapping, PageTableWalkError, PageTableWalker};
use crate::structures::paging::{
    Page, PageSize, PageTable, PageTableFlags, PageTableIndex, PhysFrame, Size1GiB, Size2MiB,
    Size4KiB,
};

/// A mapped page.
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct MappedPage<S: PageSize = Size4KiB> {
    /// The page of this mapping.
    pub page: Page<S>,

    /// The frame of this mapping.
    pub frame: PhysFrame<S>,

    /// The page table flags of this mapping.
    pub flags: PageTableFlags,
}

impl<S: PageSize> Add<u64> for MappedPage<S> {
    type Output = Self;

    fn add(self, rhs: u64) -> Self::Output {
        Self {
            page: self.page + rhs,
            frame: self.frame + rhs,
            flags: self.flags,
        }
    }
}

/// A [`MappedPage`] of any size.
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub enum MappedPageItem {
    /// The [`MappedPage`] has a size of 4KiB.
    Size4KiB(MappedPage<Size4KiB>),

    /// The [`MappedPage`] has a size of 2MiB.
    Size2MiB(MappedPage<Size2MiB>),

    /// The [`MappedPage`] has a size of 1GiB.
    Size1GiB(MappedPage<Size1GiB>),
}

impl Add<u64> for MappedPageItem {
    type Output = Self;

    fn add(self, rhs: u64) -> Self::Output {
        match self {
            Self::Size4KiB(mapped_page) => Self::Size4KiB(mapped_page + rhs),
            Self::Size2MiB(mapped_page) => Self::Size2MiB(mapped_page + rhs),
            Self::Size1GiB(mapped_page) => Self::Size1GiB(mapped_page + rhs),
        }
    }
}

/// An iterator over a [`MappedPageTable`].
///
/// This iterator returns every mapped page individually as a [`MappedPageItem`].
///
/// This struct is created by [`MappedPageTable::iter`].
///
/// # Current implementation
///
/// Performs a depth-first search for the next [`MappedPageItem`].
pub struct MappedPageTableIter<'a, P: PageTableFrameMapping> {
    page_table_walker: PageTableWalker<P>,
    level_4_table: &'a PageTable,
    p4_index: u16,
    p3_index: u16,
    p2_index: u16,
    p1_index: u16,
}

impl<P: PageTableFrameMapping> MappedPageTable<'_, P> {
    /// Returns an iterator over the page table's [`MappedPageItem`]s.
    // When making this public, add an `IntoIterator` impl for `&MappedPageTable<'_, P>`
    pub(super) fn iter(&self) -> MappedPageTableIter<'_, &P> {
        let page_table_walker = unsafe { PageTableWalker::new(self.page_table_frame_mapping()) };
        MappedPageTableIter {
            page_table_walker,
            level_4_table: self.level_4_table(),
            p4_index: 0,
            p3_index: 0,
            p2_index: 0,
            p1_index: 0,
        }
    }
}

impl<P: PageTableFrameMapping> MappedPageTableIter<'_, P> {
    /// Returns the current P4 index.
    ///
    /// When at then end, this returns [`None`].
    fn p4_index(&self) -> Option<PageTableIndex> {
        if self.p4_index == 512 {
            return None;
        }

        Some(PageTableIndex::new(self.p4_index))
    }

    /// Returns the current P3 index.
    ///
    /// When at then end, this returns [`None`].
    fn p3_index(&self) -> Option<PageTableIndex> {
        if self.p3_index == 512 {
            return None;
        }

        Some(PageTableIndex::new(self.p3_index))
    }

    /// Returns the current P2 index.
    ///
    /// When at then end, this returns [`None`].
    fn p2_index(&self) -> Option<PageTableIndex> {
        if self.p2_index == 512 {
            return None;
        }

        Some(PageTableIndex::new(self.p2_index))
    }

    /// Returns the current P1 index.
    ///
    /// When at then end, this returns [`None`].
    fn p1_index(&self) -> Option<PageTableIndex> {
        if self.p1_index == 512 {
            return None;
        }

        Some(PageTableIndex::new(self.p1_index))
    }

    /// Increments the current P4 index.
    ///
    /// This sets the lower indixes to zero.
    /// When reaching the end, this returns [`None`] .
    fn increment_p4_index(&mut self) -> Option<()> {
        self.p4_index += 1;
        self.p3_index = 0;
        self.p2_index = 0;
        self.p1_index = 0;

        if self.p4_index == 512 {
            // There is no higher index to increment.
            return None;
        }

        Some(())
    }

    /// Increments the current P3 index.
    ///
    /// This sets the lower indixes to zero.
    /// When reaching the end, this increments the next-higher index and returns [`None`].
    fn increment_p3_index(&mut self) -> Option<()> {
        self.p3_index += 1;
        self.p2_index = 0;
        self.p1_index = 0;

        if self.p3_index == 512 {
            self.increment_p4_index()?;
            return None;
        }

        Some(())
    }

    /// Increments the current P2 index.
    ///
    /// This sets the lower indixes to zero.
    /// When reaching the end, this increments the next-higher index and returns [`None`].
    fn increment_p2_index(&mut self) -> Option<()> {
        self.p2_index += 1;
        self.p1_index = 0;

        if self.p2_index == 512 {
            self.increment_p3_index()?;
            return None;
        }

        Some(())
    }

    /// Increments the current P1 index.
    ///
    /// When reaching the end, this increments the next-higher index and returns [`None`].
    fn increment_p1_index(&mut self) -> Option<()> {
        self.p1_index += 1;
        // There is no lower index to zero.

        if self.p1_index == 512 {
            self.increment_p2_index()?;
            return None;
        }

        Some(())
    }

    /// Searches for the next [`MappedPageItem`] without backtracking.
    ///
    /// This method explores the page table along the next depth-first search branch.
    /// It does not perform any backtracking and returns `None` when reaching a dead end.
    fn next_forward(&mut self) -> Option<MappedPageItem> {
        let p4 = self.level_4_table;

        // Open the current P3 table.
        let p3 = loop {
            match self.page_table_walker.next_table(&p4[self.p4_index()?]) {
                Ok(page_table) => break page_table,
                Err(PageTableWalkError::NotMapped) => {
                    // This slot is empty. Try again with the next one.
                    self.increment_p4_index()?;
                }
                Err(PageTableWalkError::MappedToHugePage) => {
                    // We cannot return a 512GiB page.
                    // Ignore the error and try again with the next slot.
                    self.increment_p4_index()?;
                }
            }
        };

        // Open the current P2 table.
        let p2 = loop {
            match self.page_table_walker.next_table(&p3[self.p3_index()?]) {
                Ok(page_table) => break page_table,
                Err(PageTableWalkError::NotMapped) => {
                    // This slot is empty. Try again with the next one.
                    self.increment_p3_index()?;
                }
                Err(PageTableWalkError::MappedToHugePage) => {
                    // We have found a 1GiB page.
                    let page =
                        Page::from_page_table_indices_1gib(self.p4_index()?, self.p3_index()?);
                    let entry = &p3[self.p3_index()?];
                    let frame = PhysFrame::containing_address(entry.addr());
                    let flags = entry.flags();
                    let mapped_page = MappedPageItem::Size1GiB(MappedPage { page, frame, flags });

                    // Make sure we don't land here next time.
                    self.increment_p3_index();
                    return Some(mapped_page);
                }
            }
        };

        // Open the current P1 table.
        let p1 = loop {
            match self.page_table_walker.next_table(&p2[self.p2_index()?]) {
                Ok(page_table) => break page_table,
                Err(PageTableWalkError::NotMapped) => {
                    // This slot is empty. Try again with the next one.
                    self.increment_p2_index()?;
                }
                Err(PageTableWalkError::MappedToHugePage) => {
                    // We have found a 2MiB page.
                    let page = Page::from_page_table_indices_2mib(
                        self.p4_index()?,
                        self.p3_index()?,
                        self.p2_index()?,
                    );
                    let entry = &p2[self.p2_index()?];
                    let frame = PhysFrame::containing_address(entry.addr());
                    let flags = entry.flags();
                    let mapped_page = MappedPageItem::Size2MiB(MappedPage { page, frame, flags });

                    // Make sure we don't land here next time.
                    self.increment_p2_index();
                    return Some(mapped_page);
                }
            }
        };

        while !p1[self.p1_index()?]
            .flags()
            .contains(PageTableFlags::PRESENT)
        {
            self.increment_p1_index()?;
        }

        // We have found a 4KiB page.
        let page = Page::from_page_table_indices(
            self.p4_index()?,
            self.p3_index()?,
            self.p2_index()?,
            self.p1_index()?,
        );
        let entry = &p1[self.p1_index()?];
        let frame = PhysFrame::containing_address(entry.addr());
        let flags = entry.flags();
        let mapped_page = MappedPageItem::Size4KiB(MappedPage { page, frame, flags });

        // Make sure we don't land here next time.
        self.increment_p1_index();
        Some(mapped_page)
    }
}

impl<P: PageTableFrameMapping> Iterator for MappedPageTableIter<'_, P> {
    type Item = MappedPageItem;

    fn next(&mut self) -> Option<Self::Item> {
        // Call `next_forward` until we have explored all P4 indexes.
        while self.p4_index().is_some() {
            if let Some(item) = self.next_forward() {
                return Some(item);
            }
        }

        None
    }
}
