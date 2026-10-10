//! Iterator over [`MappedPageTable`]s.
//!
//! The main type of this module is [`MappedPageTableIter`] returning [`MappedPageItem`]s.

use super::{MappedPageTable, PageTableFrameMapping, PageTableWalkError, PageTableWalker};
use crate::addr::{FixedValidity, VirtAddrValidity};
use crate::structures::paging::page_table::{PageTableIndices, PageTableLevel};
use crate::structures::paging::{
    Page, PageSize, PageTable, PageTableFlags, PageTableIndex, PhysFrame, Size1GiB, Size2MiB,
    Size4KiB,
};

/// A mapped page.
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct MappedPage<S: PageSize = Size4KiB, V: VirtAddrValidity = FixedValidity<48>> {
    /// The page of this mapping.
    pub page: Page<S, V>,

    /// The frame of this mapping.
    pub frame: PhysFrame<S>,

    /// The page table flags of this mapping.
    pub flags: PageTableFlags,
}

/// A [`MappedPage`] of any size.
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub enum MappedPageItem<V: VirtAddrValidity = FixedValidity<48>> {
    /// The [`MappedPage`] has a size of 4KiB.
    Size4KiB(MappedPage<Size4KiB, V>),

    /// The [`MappedPage`] has a size of 2MiB.
    Size2MiB(MappedPage<Size2MiB, V>),

    /// The [`MappedPage`] has a size of 1GiB.
    Size1GiB(MappedPage<Size1GiB, V>),
}

impl<V: VirtAddrValidity> MappedPageItem<V> {
    /// Whether this mapping immediately follows the other mapping in both address spaces.
    pub(super) fn follows(&self, previous: &Self) -> bool {
        fn adjacent<S: PageSize, V: VirtAddrValidity>(
            next: &MappedPage<S, V>,
            prev: &MappedPage<S, V>,
        ) -> bool {
            prev.flags == next.flags
                && prev.page.start_address().as_u64().checked_add(S::SIZE)
                    == Some(next.page.start_address().as_u64())
                && prev.frame.start_address().as_u64().checked_add(S::SIZE)
                    == Some(next.frame.start_address().as_u64())
        }
        match (self, previous) {
            (Self::Size4KiB(next), Self::Size4KiB(prev)) => adjacent(next, prev),
            (Self::Size2MiB(next), Self::Size2MiB(prev)) => adjacent(next, prev),
            (Self::Size1GiB(next), Self::Size1GiB(prev)) => adjacent(next, prev),
            _ => false,
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
pub struct MappedPageTableIter<'a, P: PageTableFrameMapping, const BITS: usize = 48>
where
    FixedValidity<BITS>: VirtAddrValidity,
{
    page_table_walker: PageTableWalker<P>,
    root_table: &'a PageTable,
    /// Next position to inspect, or `None` once the entire tree is exhausted.
    position: Option<PageTableIndices>,
}

impl<P: PageTableFrameMapping, const BITS: usize> MappedPageTable<'_, P, BITS>
where
    FixedValidity<BITS>: VirtAddrValidity,
{
    /// Returns an iterator over the page table's [`MappedPageItem`]s.
    // When making this public, add an `IntoIterator` impl for `&MappedPageTable<'_, P>`
    pub(super) fn iter(&self) -> MappedPageTableIter<'_, &P, BITS> {
        let page_table_walker = unsafe { PageTableWalker::new(self.page_table_frame_mapping()) };
        MappedPageTableIter {
            page_table_walker,
            root_table: self.root_table(),
            position: Some(PageTableIndices::from_address(0)),
        }
    }
}

impl<P: PageTableFrameMapping, const BITS: usize> MappedPageTableIter<'_, P, BITS>
where
    FixedValidity<BITS>: VirtAddrValidity,
{
    /// Advances past the entry at `level`, clearing all lower indices.
    ///
    /// Returns the highest level changed by carrying, or `None` when the root is exhausted.
    /// If the returned level is above `level`, callers must reopen tables from the root
    /// before continuing: their current child-table references belong to the previous branch.
    fn advance(&mut self, mut level: PageTableLevel) -> Option<PageTableLevel> {
        let root_level = if BITS == 57 {
            PageTableLevel::Five
        } else {
            PageTableLevel::Four
        };
        debug_assert!(level <= root_level);
        let position = self.position.as_mut()?;
        position.zero_below(level);
        loop {
            let index = u16::from(position.index(level));
            let next = PageTableIndex::new_truncate(index + 1);
            position.set_index(level, next);
            if u16::from(next) != 0 {
                return Some(level);
            }
            if level == root_level {
                self.position = None;
                return None;
            }
            level = level.next_higher_level().expect("level is below the root");
        }
    }

    /// Searches for the next [`MappedPageItem`] without backtracking.
    ///
    /// This method explores the page table along the next depth-first search branch.
    /// It does not perform any backtracking and returns `None` when reaching a dead end.
    fn next_forward(&mut self) -> Option<MappedPageItem<FixedValidity<BITS>>> {
        let p4 = if BITS == 57 {
            loop {
                match self
                    .page_table_walker
                    .next_table(&self.root_table[self.position?.index(PageTableLevel::Five)])
                {
                    Ok(table) => break table,
                    Err(_) => {
                        self.advance(PageTableLevel::Five)?;
                    }
                }
            }
        } else {
            self.root_table
        };

        // Open the current P3 table.
        let p3 = loop {
            match self
                .page_table_walker
                .next_table(&p4[self.position?.index(PageTableLevel::Four)])
            {
                Ok(page_table) => break page_table,
                Err(PageTableWalkError::NotMapped) => {
                    // This slot is empty. Try again with the next one.
                    if self.advance(PageTableLevel::Four)? != PageTableLevel::Four {
                        return None;
                    }
                }
                Err(PageTableWalkError::MappedToHugePage) => {
                    // We cannot return a 512GiB page.
                    // Ignore the error and try again with the next slot.
                    if self.advance(PageTableLevel::Four)? != PageTableLevel::Four {
                        return None;
                    }
                }
            }
        };

        // Open the current P2 table.
        let p2 = loop {
            match self
                .page_table_walker
                .next_table(&p3[self.position?.index(PageTableLevel::Three)])
            {
                Ok(page_table) => break page_table,
                Err(PageTableWalkError::NotMapped) => {
                    // This slot is empty. Try again with the next one.
                    if self.advance(PageTableLevel::Three)? != PageTableLevel::Three {
                        return None;
                    }
                }
                Err(PageTableWalkError::MappedToHugePage) => {
                    // We have found a 1GiB page.
                    let page = if BITS == 57 {
                        Page::<Size1GiB, FixedValidity<BITS>>::from_page_table_indices_1gib_l5(
                            self.position?.index(PageTableLevel::Five),
                            self.position?.index(PageTableLevel::Four),
                            self.position?.index(PageTableLevel::Three),
                        )
                    } else {
                        Page::<Size1GiB, FixedValidity<BITS>>::from_page_table_indices_1gib(
                            self.position?.index(PageTableLevel::Four),
                            self.position?.index(PageTableLevel::Three),
                        )
                    };
                    let entry = &p3[self.position?.index(PageTableLevel::Three)];
                    let frame = PhysFrame::containing_address(entry.addr());
                    let flags = entry.flags();
                    let mapped_page = MappedPageItem::Size1GiB(MappedPage { page, frame, flags });

                    // Make sure we don't land here next time.
                    self.advance(PageTableLevel::Three);
                    return Some(mapped_page);
                }
            }
        };

        // Open the current P1 table.
        let p1 = loop {
            match self
                .page_table_walker
                .next_table(&p2[self.position?.index(PageTableLevel::Two)])
            {
                Ok(page_table) => break page_table,
                Err(PageTableWalkError::NotMapped) => {
                    // This slot is empty. Try again with the next one.
                    if self.advance(PageTableLevel::Two)? != PageTableLevel::Two {
                        return None;
                    }
                }
                Err(PageTableWalkError::MappedToHugePage) => {
                    // We have found a 2MiB page.
                    let page = if BITS == 57 {
                        Page::<Size2MiB, FixedValidity<BITS>>::from_page_table_indices_2mib_l5(
                            self.position?.index(PageTableLevel::Five),
                            self.position?.index(PageTableLevel::Four),
                            self.position?.index(PageTableLevel::Three),
                            self.position?.index(PageTableLevel::Two),
                        )
                    } else {
                        Page::<Size2MiB, FixedValidity<BITS>>::from_page_table_indices_2mib(
                            self.position?.index(PageTableLevel::Four),
                            self.position?.index(PageTableLevel::Three),
                            self.position?.index(PageTableLevel::Two),
                        )
                    };
                    let entry = &p2[self.position?.index(PageTableLevel::Two)];
                    let frame = PhysFrame::containing_address(entry.addr());
                    let flags = entry.flags();
                    let mapped_page = MappedPageItem::Size2MiB(MappedPage { page, frame, flags });

                    // Make sure we don't land here next time.
                    self.advance(PageTableLevel::Two);
                    return Some(mapped_page);
                }
            }
        };

        while !p1[self.position?.index(PageTableLevel::One)]
            .flags()
            .contains(PageTableFlags::PRESENT)
        {
            if self.advance(PageTableLevel::One)? != PageTableLevel::One {
                return None;
            }
        }

        // We have found a 4KiB page.
        let page = if BITS == 57 {
            Page::<Size4KiB, FixedValidity<BITS>>::from_page_table_indices_l5(
                self.position?.index(PageTableLevel::Five),
                self.position?.index(PageTableLevel::Four),
                self.position?.index(PageTableLevel::Three),
                self.position?.index(PageTableLevel::Two),
                self.position?.index(PageTableLevel::One),
            )
        } else {
            Page::<Size4KiB, FixedValidity<BITS>>::from_page_table_indices(
                self.position?.index(PageTableLevel::Four),
                self.position?.index(PageTableLevel::Three),
                self.position?.index(PageTableLevel::Two),
                self.position?.index(PageTableLevel::One),
            )
        };
        let entry = &p1[self.position?.index(PageTableLevel::One)];
        let frame = PhysFrame::containing_address(entry.addr());
        let flags = entry.flags();
        let mapped_page = MappedPageItem::Size4KiB(MappedPage { page, frame, flags });

        // Make sure we don't land here next time.
        self.advance(PageTableLevel::One);
        Some(mapped_page)
    }
}

impl<P: PageTableFrameMapping, const BITS: usize> Iterator for MappedPageTableIter<'_, P, BITS>
where
    FixedValidity<BITS>: VirtAddrValidity,
{
    type Item = MappedPageItem<FixedValidity<BITS>>;

    fn next(&mut self) -> Option<Self::Item> {
        // Restart from the root after crossing a table boundary, until the tree is exhausted.
        while self.position.is_some() {
            if let Some(item) = self.next_forward() {
                return Some(item);
            }
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct IdentityMapping;

    unsafe impl PageTableFrameMapping for IdentityMapping {
        fn frame_to_pointer(&self, frame: PhysFrame) -> *mut PageTable {
            frame.start_address().as_u64() as *mut PageTable
        }
    }

    fn assert_advance<const BITS: usize>(
        start: [u16; 5],
        level: PageTableLevel,
        expected: Option<(PageTableLevel, [u16; 5])>,
    ) where
        FixedValidity<BITS>: VirtAddrValidity,
    {
        let levels = [
            PageTableLevel::One,
            PageTableLevel::Two,
            PageTableLevel::Three,
            PageTableLevel::Four,
            PageTableLevel::Five,
        ];
        let mut root = PageTable::new();
        let mapper = unsafe { MappedPageTable::<_, BITS>::new(&mut root, IdentityMapping) };
        let mut iter = mapper.iter();
        let position = iter.position.as_mut().unwrap();
        for (level, index) in levels.into_iter().zip(start) {
            position.set_index(level, PageTableIndex::new(index));
        }
        match expected {
            Some((changed_level, indices)) => {
                assert_eq!(iter.advance(level), Some(changed_level));
                let position = iter.position.unwrap();
                assert_eq!(
                    levels.map(|level| u16::from(position.index(level))),
                    indices
                );
            }
            None => {
                assert_eq!(iter.advance(level), None);
                assert!(iter.position.is_none());
                assert_eq!(iter.advance(PageTableLevel::One), None);
                assert!(iter.next().is_none());
            }
        }
    }

    #[test]
    fn advance_clears_lower_indices_and_preserves_higher_indices() {
        for (level, expected) in [
            (PageTableLevel::One, [2, 2, 3, 4, 0]),
            (PageTableLevel::Two, [0, 3, 3, 4, 0]),
            (PageTableLevel::Three, [0, 0, 4, 4, 0]),
            (PageTableLevel::Four, [0, 0, 0, 5, 0]),
        ] {
            assert_advance::<48>([1, 2, 3, 4, 0], level, Some((level, expected)));
        }
        #[cfg(feature = "virt_addr_57")]
        assert_advance::<57>(
            [1, 2, 3, 4, 5],
            PageTableLevel::Five,
            Some((PageTableLevel::Five, [0, 0, 0, 0, 6])),
        );
    }

    #[test]
    fn advance_carries_to_the_correct_parent_level() {
        for (start, level, changed_level, expected) in [
            (
                [511, 1, 2, 3, 0],
                PageTableLevel::One,
                PageTableLevel::Two,
                [0, 2, 2, 3, 0],
            ),
            (
                [7, 511, 2, 3, 0],
                PageTableLevel::Two,
                PageTableLevel::Three,
                [0, 0, 3, 3, 0],
            ),
            (
                [7, 8, 511, 3, 0],
                PageTableLevel::Three,
                PageTableLevel::Four,
                [0, 0, 0, 4, 0],
            ),
            (
                [511, 511, 511, 3, 0],
                PageTableLevel::One,
                PageTableLevel::Four,
                [0, 0, 0, 4, 0],
            ),
        ] {
            assert_advance::<48>(start, level, Some((changed_level, expected)));
        }
        #[cfg(feature = "virt_addr_57")]
        assert_advance::<57>(
            [7, 8, 9, 511, 4],
            PageTableLevel::Four,
            Some((PageTableLevel::Five, [0, 0, 0, 0, 5])),
        );
    }

    #[test]
    fn advance_past_the_actual_root_exhausts_the_iterator() {
        assert_advance::<48>([7, 8, 9, 511, 0], PageTableLevel::Four, None);
        assert_advance::<48>([511, 511, 511, 511, 0], PageTableLevel::One, None);
        #[cfg(feature = "virt_addr_57")]
        {
            assert_advance::<57>([7, 8, 9, 10, 511], PageTableLevel::Five, None);
            assert_advance::<57>([511, 511, 511, 511, 511], PageTableLevel::One, None);
        }
    }

    /// Builds an independent subtree ending at `leaf_level` and keeps every table alive.
    fn subtree(
        tables: &mut std::vec::Vec<std::boxed::Box<PageTable>>,
        level: PageTableLevel,
        leaf_level: PageTableLevel,
        index: usize,
        mapped: bool,
    ) -> crate::PhysAddr {
        let mut table = std::boxed::Box::new(PageTable::new());
        let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;
        if level > leaf_level {
            let child = subtree(
                tables,
                level.next_lower_level().unwrap(),
                leaf_level,
                index,
                mapped,
            );
            table[index].set_addr(child, flags);
        } else if mapped {
            let flags = if leaf_level == PageTableLevel::One {
                flags
            } else {
                flags | PageTableFlags::HUGE_PAGE
            };
            table[index].set_addr(crate::PhysAddr::new(0x4000_0000), flags);
        }
        let address = crate::PhysAddr::new(&mut *table as *mut PageTable as u64);
        tables.push(table);
        address
    }

    fn check_parent_boundaries<const BITS: usize>()
    where
        FixedValidity<BITS>: VirtAddrValidity,
    {
        let root_level = if BITS == 57 {
            PageTableLevel::Five
        } else {
            PageTableLevel::Four
        };
        for (child_level, boundary) in [
            (PageTableLevel::One, 0x20_0000),
            (PageTableLevel::Two, 0x4000_0000),
            (PageTableLevel::Three, 0x80_0000_0000),
            (PageTableLevel::Four, 0x1_0000_0000_0000),
        ] {
            if child_level >= root_level {
                continue;
            }
            for (leaf_level, page_size) in [
                (PageTableLevel::One, 0x1000),
                (PageTableLevel::Two, 0x20_0000),
                (PageTableLevel::Three, 0x4000_0000),
            ] {
                if leaf_level > child_level {
                    continue;
                }
                for mapped in [false, true] {
                    let mut tables = std::vec::Vec::new();
                    let mut parent = std::boxed::Box::new(PageTable::new());
                    let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;
                    parent[0].set_addr(
                        subtree(&mut tables, child_level, leaf_level, 511, mapped),
                        flags,
                    );
                    parent[1].set_addr(
                        subtree(&mut tables, child_level, leaf_level, 0, true),
                        flags,
                    );
                    let mut level = child_level.next_higher_level().unwrap();
                    while level < root_level {
                        let mut higher = std::boxed::Box::new(PageTable::new());
                        higher[0].set_addr(
                            crate::PhysAddr::new(&mut *parent as *mut PageTable as u64),
                            flags,
                        );
                        tables.push(parent);
                        parent = higher;
                        level = level.next_higher_level().unwrap();
                    }
                    let mapper =
                        unsafe { MappedPageTable::<_, BITS>::new(&mut parent, IdentityMapping) };
                    let mut iter = mapper.iter();
                    // Start at the final entry of the old subtree. It may be absent or mapped.
                    iter.position = Some(PageTableIndices::from_address(boundary - page_size));
                    let mut expected = std::vec::Vec::new();
                    if mapped {
                        expected.push(boundary - page_size);
                    }
                    expected.push(boundary);
                    for address in expected {
                        let (va, pa, size, actual_flags) =
                            match iter.next().expect("next subtree must not be skipped") {
                                MappedPageItem::Size4KiB(m) => (
                                    m.page.start_address().as_u64(),
                                    m.frame.start_address().as_u64(),
                                    Size4KiB::SIZE,
                                    m.flags,
                                ),
                                MappedPageItem::Size2MiB(m) => (
                                    m.page.start_address().as_u64(),
                                    m.frame.start_address().as_u64(),
                                    Size2MiB::SIZE,
                                    m.flags,
                                ),
                                MappedPageItem::Size1GiB(m) => (
                                    m.page.start_address().as_u64(),
                                    m.frame.start_address().as_u64(),
                                    Size1GiB::SIZE,
                                    m.flags,
                                ),
                            };
                        assert_eq!((va, pa, size), (address, 0x4000_0000, page_size));
                        assert_eq!(
                            actual_flags,
                            if leaf_level == PageTableLevel::One {
                                flags
                            } else {
                                flags | PageTableFlags::HUGE_PAGE
                            }
                        );
                    }
                    assert!(iter.next().is_none());
                    assert!(iter.next().is_none());
                }
            }
        }
    }

    #[test]
    fn l4_traversal_reopens_parents_after_empty_and_mapped_entries() {
        check_parent_boundaries::<48>();
    }

    #[cfg(feature = "virt_addr_57")]
    #[test]
    fn l5_traversal_reopens_parents_after_empty_and_mapped_entries() {
        check_parent_boundaries::<57>();
    }

    #[test]
    fn empty_root_traversal_preserves_exhaustion() {
        fn check<const BITS: usize>()
        where
            FixedValidity<BITS>: VirtAddrValidity,
        {
            let mut root = PageTable::new();
            let mapper = unsafe { MappedPageTable::<_, BITS>::new(&mut root, IdentityMapping) };
            let mut iter = mapper.iter();
            // One bounded traversal exposes the bug without hanging inside Iterator::next.
            assert!(iter.next_forward().is_none());
            assert!(
                iter.position.is_none(),
                "root traversal must remain exhausted"
            );
            assert!(iter.next().is_none());
            assert!(iter.next().is_none());
            assert_eq!(std::format!("{}", mapper.display()), "");
        }
        check::<48>();
        #[cfg(feature = "virt_addr_57")]
        check::<57>();
    }

    #[cfg(feature = "virt_addr_57")]
    #[test]
    fn l5_traversal_reopens_p4_after_advancing_p5() {
        let mut root = PageTable::new();
        let mut first_p4 = PageTable::new();
        let mut second_p4 = PageTable::new();
        let mut p3 = PageTable::new();
        let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;
        root[0].set_addr(
            crate::PhysAddr::new(&mut first_p4 as *mut PageTable as u64),
            flags,
        );
        root[1].set_addr(
            crate::PhysAddr::new(&mut second_p4 as *mut PageTable as u64),
            flags,
        );
        second_p4[0].set_addr(
            crate::PhysAddr::new(&mut p3 as *mut PageTable as u64),
            flags,
        );
        p3[0].set_addr(
            crate::PhysAddr::new(Size1GiB::SIZE),
            flags | PageTableFlags::HUGE_PAGE,
        );
        let mapper = unsafe { MappedPageTable::<_, 57>::new(&mut root, IdentityMapping) };
        let mut iter = mapper.iter();
        iter.position
            .as_mut()
            .unwrap()
            .set_index(PageTableLevel::Four, PageTableIndex::new(511));
        // Crossing the parent boundary must return to the root before reading another P4.
        assert!(iter.next_forward().is_none());
        let position = iter.position.unwrap();
        assert_eq!(u16::from(position.index(PageTableLevel::Five)), 1);
        assert_eq!(u16::from(position.index(PageTableLevel::Four)), 0);
        let Some(MappedPageItem::Size1GiB(mapping)) = iter.next() else {
            panic!("mapping in the next P5 entry was skipped");
        };
        assert_eq!(
            mapping.page.start_address().as_u64(),
            crate::structures::paging::page_table::PageTableLevel::Five
                .entry_address_space_alignment()
        );
        assert_eq!(mapping.frame.start_address().as_u64(), Size1GiB::SIZE);
        assert!(iter.next().is_none());
        assert!(iter.next().is_none());
    }
}
