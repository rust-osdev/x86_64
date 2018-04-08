//! Abstractions for page tables and other paging related structures.

pub use self::page_table::*;
pub use self::recursive::*;

use addr::{PhysAddr, VirtAddr};
use core::ops::{Add, AddAssign, Sub, SubAssign};
use ux::*;

mod page_table;
mod recursive;

/// The default page size on x86_64.
pub const PAGE_SIZE: u16 = 4096;

/// A virtual 4kB page.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Page {
    number: u64,
}

impl Page {
    /// Returns the page that contains the given virtual address.
    pub fn containing_address(address: VirtAddr) -> Page {
        Page {
            number: address.as_u64() / u64::from(PAGE_SIZE),
        }
    }

    pub fn from_page_table_indices(p4_index: u9, p3_index: u9, p2_index: u9, p1_index: u9) -> Page {
        use bit_field::BitField;

        let mut addr = 0;
        addr.set_bits(39..48, u64::from(p4_index));
        addr.set_bits(30..39, u64::from(p3_index));
        addr.set_bits(21..30, u64::from(p2_index));
        addr.set_bits(12..21, u64::from(p1_index));
        Page::containing_address(VirtAddr::new(addr))
    }

    /// Returns the start address of the page.
    pub fn start_address(&self) -> VirtAddr {
        VirtAddr::new(self.number * u64::from(PAGE_SIZE))
    }

    /// Returns the level 4 page table index of this page.
    pub fn p4_index(&self) -> u9 {
        self.start_address().p4_index()
    }

    /// Returns the level 3 page table index of this page.
    pub fn p3_index(&self) -> u9 {
        self.start_address().p3_index()
    }

    /// Returns the level 2 page table index of this page.
    pub fn p2_index(&self) -> u9 {
        self.start_address().p2_index()
    }

    /// Returns the level 1 page table index of this page.
    pub fn p1_index(&self) -> u9 {
        self.start_address().p1_index()
    }

    pub fn range(start: Page, end: Page) -> PageRange {
        PageRange { start, end }
    }

    pub fn range_inclusive(start: Page, end: Page) -> PageRangeInclusive {
        PageRangeInclusive { start, end }
    }
}

impl Add<u64> for Page {
    type Output = Self;
    fn add(self, rhs: u64) -> Self::Output {
        Page::containing_address(self.start_address() + rhs * u64::from(PAGE_SIZE))
    }
}

impl AddAssign<u64> for Page {
    fn add_assign(&mut self, rhs: u64) {
        *self = self.clone() + rhs;
    }
}

impl Sub<u64> for Page {
    type Output = Self;
    fn sub(self, rhs: u64) -> Self::Output {
        Page::containing_address(self.start_address() - rhs * u64::from(PAGE_SIZE))
    }
}

impl SubAssign<u64> for Page {
    fn sub_assign(&mut self, rhs: u64) {
        *self = self.clone() - rhs;
    }
}

impl Sub<Page> for Page {
    type Output = u64;
    fn sub(self, rhs: Page) -> Self::Output {
        self.number.checked_sub(rhs.number).unwrap()
    }
}

pub struct PageRange {
    pub start: Page,
    pub end: Page,
}

impl Iterator for PageRange {
    type Item = Page;

    fn next(&mut self) -> Option<Page> {
        if self.start < self.end {
            let page = self.start.clone();
            self.start += 1;
            Some(page)
        } else {
            None
        }
    }
}

pub struct PageRangeInclusive {
    pub start: Page,
    pub end: Page,
}

impl Iterator for PageRangeInclusive {
    type Item = Page;

    fn next(&mut self) -> Option<Page> {
        if self.start <= self.end {
            let page = self.start.clone();
            self.start += 1;
            Some(page)
        } else {
            None
        }
    }
}

/// A physical 4kB frame.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct PhysFrame {
    number: u64,
}

impl PhysFrame {
    /// Returns the frame that contains the given physical address.
    pub fn containing_address(address: PhysAddr) -> PhysFrame {
        PhysFrame {
            number: address.as_u64() / u64::from(PAGE_SIZE),
        }
    }

    /// Returns the start address of the page.
    pub fn start_address(&self) -> PhysAddr {
        PhysAddr::new(self.number * u64::from(PAGE_SIZE))
    }

    pub fn range(start: PhysFrame, end: PhysFrame) -> PhysFrameRange {
        PhysFrameRange { start, end }
    }

    pub fn range_inclusive(start: PhysFrame, end: PhysFrame) -> PhysFrameRangeInclusive {
        PhysFrameRangeInclusive { start, end }
    }
}

impl Add<u64> for PhysFrame {
    type Output = Self;
    fn add(self, rhs: u64) -> Self::Output {
        PhysFrame::containing_address(self.start_address() + rhs * u64::from(PAGE_SIZE))
    }
}

impl AddAssign<u64> for PhysFrame {
    fn add_assign(&mut self, rhs: u64) {
        *self = self.clone() + rhs;
    }
}

impl Sub<u64> for PhysFrame {
    type Output = Self;
    fn sub(self, rhs: u64) -> Self::Output {
        PhysFrame::containing_address(self.start_address() - rhs * u64::from(PAGE_SIZE))
    }
}

impl SubAssign<u64> for PhysFrame {
    fn sub_assign(&mut self, rhs: u64) {
        *self = self.clone() - rhs;
    }
}

impl Sub<PhysFrame> for PhysFrame {
    type Output = u64;
    fn sub(self, rhs: PhysFrame) -> Self::Output {
        self.number.checked_sub(rhs.number).unwrap()
    }
}

pub struct PhysFrameRange {
    pub start: PhysFrame,
    pub end: PhysFrame,
}

impl Iterator for PhysFrameRange {
    type Item = PhysFrame;

    fn next(&mut self) -> Option<PhysFrame> {
        if self.start < self.end {
            let frame = self.start.clone();
            self.start += 1;
            Some(frame)
        } else {
            None
        }
    }
}

pub struct PhysFrameRangeInclusive {
    pub start: PhysFrame,
    pub end: PhysFrame,
}

impl Iterator for PhysFrameRangeInclusive {
    type Item = PhysFrame;

    fn next(&mut self) -> Option<PhysFrame> {
        if self.start <= self.end {
            let frame = self.start.clone();
            self.start += 1;
            Some(frame)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn test_page_ranges() {
        let page_size = u64::from(PAGE_SIZE);
        let number = 1000;

        let start_addr = VirtAddr::new(0xdeadbeaf);
        let start = Page::containing_address(start_addr);
        let end = start.clone() + number;

        let mut range = Page::range(start.clone(), end.clone());
        for i in 0..number {
            assert_eq!(
                range.next(),
                Some(Page::containing_address(
                    start_addr + u64::from(PAGE_SIZE) * i
                ))
            );
        }
        assert_eq!(range.next(), None);

        let mut range_inclusive = Page::range_inclusive(start, end);
        for i in 0..=number {
            assert_eq!(
                range_inclusive.next(),
                Some(Page::containing_address(
                    start_addr + u64::from(PAGE_SIZE) * i
                ))
            );
        }
        assert_eq!(range_inclusive.next(), None);
    }
}
