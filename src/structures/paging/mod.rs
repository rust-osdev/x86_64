//! Abstractions for page tables and other paging related structures.
//!
//! Page tables translate virtual memory “pages” to physical memory “frames”.

pub use self::frame_alloc::*;
pub use self::page_table::*;
#[cfg(target_pointer_width = "64")]
pub use self::recursive::*;

use core::{
    fmt,
    iter::Step,
    marker::PhantomData,
    mem,
    ops::{Add, AddAssign, Range, Sub, SubAssign},
};
use os_bootinfo;
use ux::*;
use {PhysAddr, VirtAddr};

mod frame_alloc;
mod page_table;
mod recursive;

/// Trait for abstracting over the three possible page sizes on x86_64, 4KiB, 2MiB, 1GiB.
pub trait PageSize: Copy + Eq + PartialOrd + Ord {
    /// The page size in bytes.
    const SIZE: u64;

    /// A string representation of the page size for debug output.
    const SIZE_AS_DEBUG_STR: &'static str;
}

/// This trait is implemented for 4KiB and 2MiB pages, but not for 1GiB pages.
pub trait NotGiantPageSize: PageSize {}

/// A standard 4KiB page.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Size4KiB {}

/// A “huge” 2MiB page.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Size2MiB {}

/// A giant” 4GiB page.
///
/// (Only available on newer x86_64 CPUs.)
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Size1GiB {}

impl PageSize for Size4KiB {
    const SIZE: u64 = 4096;
    const SIZE_AS_DEBUG_STR: &'static str = "4KiB";
}

impl NotGiantPageSize for Size4KiB {}

impl PageSize for Size2MiB {
    const SIZE: u64 = Size4KiB::SIZE * 512;
    const SIZE_AS_DEBUG_STR: &'static str = "2MiB";
}

impl NotGiantPageSize for Size2MiB {}

impl PageSize for Size1GiB {
    const SIZE: u64 = Size2MiB::SIZE * 512;
    const SIZE_AS_DEBUG_STR: &'static str = "1GiB";
}

/// A virtual memory page.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(C)]
pub struct Page<S: PageSize = Size4KiB> {
    start_address: VirtAddr,
    size: PhantomData<S>,
}

impl<S: PageSize> Page<S> {
    /// Returns the page that starts at the given virtual address.
    ///
    /// Returns an error if the address is not correctly aligned (i.e. is not a valid page start).
    pub fn from_start_address(address: VirtAddr) -> Result<Self, ()> {
        if !address.is_aligned(S::SIZE) {
            return Err(());
        }
        Ok(Page::containing_address(address))
    }

    /// Returns the page that contains the given virtual address.
    pub fn containing_address(address: VirtAddr) -> Self {
        Page {
            start_address: address.align_down(S::SIZE),
            size: PhantomData,
        }
    }

    /// Returns the start address of the page.
    pub fn start_address(&self) -> VirtAddr {
        self.start_address
    }

    /// Returns the size the page (4KB, 2MB or 1GB).
    pub const fn size(&self) -> u64 {
        S::SIZE
    }

    /// Returns the level 4 page table index of this page.
    pub fn p4_index(&self) -> u9 {
        self.start_address().p4_index()
    }

    /// Returns the level 3 page table index of this page.
    pub fn p3_index(&self) -> u9 {
        self.start_address().p3_index()
    }
}

impl<S: NotGiantPageSize> Page<S> {
    /// Returns the level 2 page table index of this page.
    pub fn p2_index(&self) -> u9 {
        self.start_address().p2_index()
    }
}

impl Page<Size1GiB> {
    /// Returns the 1GiB memory page with the specified page table indices.
    pub fn from_page_table_indices_1gib(p4_index: u9, p3_index: u9) -> Self {
        use bit_field::BitField;

        let mut addr = 0;
        addr.set_bits(39..48, u64::from(p4_index));
        addr.set_bits(30..39, u64::from(p3_index));
        Page::containing_address(VirtAddr::new(addr))
    }
}

impl Page<Size2MiB> {
    /// Returns the 2MiB memory page with the specified page table indices.
    pub fn from_page_table_indices_2mib(p4_index: u9, p3_index: u9, p2_index: u9) -> Self {
        use bit_field::BitField;

        let mut addr = 0;
        addr.set_bits(39..48, u64::from(p4_index));
        addr.set_bits(30..39, u64::from(p3_index));
        addr.set_bits(21..30, u64::from(p2_index));
        Page::containing_address(VirtAddr::new(addr))
    }
}

impl Page<Size4KiB> {
    /// Returns the 4KiB memory page with the specified page table indices.
    pub fn from_page_table_indices(p4_index: u9, p3_index: u9, p2_index: u9, p1_index: u9) -> Self {
        use bit_field::BitField;

        let mut addr = 0;
        addr.set_bits(39..48, u64::from(p4_index));
        addr.set_bits(30..39, u64::from(p3_index));
        addr.set_bits(21..30, u64::from(p2_index));
        addr.set_bits(12..21, u64::from(p1_index));
        Page::containing_address(VirtAddr::new(addr))
    }

    /// Returns the level 1 page table index of this page.
    pub fn p1_index(&self) -> u9 {
        self.start_address().p1_index()
    }
}

impl<S: PageSize> fmt::Debug for Page<S> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_fmt(format_args!(
            "Page[{}]({:#x})",
            S::SIZE_AS_DEBUG_STR,
            self.start_address().as_u64()
        ))
    }
}

impl<S: PageSize> Add<u64> for Page<S> {
    type Output = Self;
    fn add(self, rhs: u64) -> Self::Output {
        Page::containing_address(self.start_address() + rhs * u64::from(S::SIZE))
    }
}

impl<S: PageSize> AddAssign<u64> for Page<S> {
    fn add_assign(&mut self, rhs: u64) {
        *self = self.clone() + rhs;
    }
}

impl<S: PageSize> Sub<u64> for Page<S> {
    type Output = Self;
    fn sub(self, rhs: u64) -> Self::Output {
        Page::containing_address(self.start_address() - rhs * u64::from(S::SIZE))
    }
}

impl<S: PageSize> SubAssign<u64> for Page<S> {
    fn sub_assign(&mut self, rhs: u64) {
        *self = self.clone() - rhs;
    }
}

impl<S: PageSize> Sub<Self> for Page<S> {
    type Output = u64;
    fn sub(self, rhs: Self) -> Self::Output {
        (self.start_address - rhs.start_address) / S::SIZE
    }
}

impl<S: PageSize> Step for Page<S> {
    fn steps_between(start: &Self, end: &Self) -> Option<usize> {
        if start.start_address < end.start_address {
            Some((*end - *start) as usize)
        } else {
            Some(0)
        }
    }

    fn replace_one(&mut self) -> Self {
        let page_one = Page {
            start_address: VirtAddr::new(S::SIZE),
            size: PhantomData,
        };

        mem::replace(self, page_one)
    }

    fn replace_zero(&mut self) -> Self {
        let page_zero = Page {
            start_address: VirtAddr::new(0),
            size: PhantomData,
        };

        mem::replace(self, page_zero)
    }

    fn add_one(&self) -> Self {
        *self + 1
    }

    fn sub_one(&self) -> Self {
        *self - 1
    }

    fn add_usize(&self, n: usize) -> Option<Self> {
        if *self < (Page::containing_address(VirtAddr::new(!0)) - n as u64) {
            // Would overflow.
            None
        } else {
            Some(*self + (n as u64))
        }
    }
}

/// Converts a range of 2MiB pages into a range of 4KiB pages. This doesn't change the range; it
/// just changes the page size.
pub fn as_4kib_page_range(Range { start, end }: Range<Page<Size2MiB>>) -> Range<Page<Size4KiB>> {
    Range {
        start: Page::from_start_address(start.start_address()).unwrap(),
        end: Page::from_start_address(end.start_address()).unwrap(),
    }
}

/// A physical memory frame.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(C)]
pub struct PhysFrame<S: PageSize = Size4KiB> {
    start_address: PhysAddr,
    size: PhantomData<S>,
}

impl<S: PageSize> PhysFrame<S> {
    /// Returns the frame that starts at the given virtual address.
    ///
    /// Returns an error if the address is not correctly aligned (i.e. is not a valid frame start).
    pub fn from_start_address(address: PhysAddr) -> Result<Self, ()> {
        if !address.is_aligned(S::SIZE) {
            return Err(());
        }
        Ok(PhysFrame::containing_address(address))
    }

    /// Returns the frame that contains the given physical address.
    pub fn containing_address(address: PhysAddr) -> Self {
        PhysFrame {
            start_address: address.align_down(S::SIZE),
            size: PhantomData,
        }
    }

    /// Returns the start address of the frame.
    pub fn start_address(&self) -> PhysAddr {
        self.start_address
    }

    /// Returns the size the frame (4KB, 2MB or 1GB).
    pub fn size(&self) -> u64 {
        S::SIZE
    }
}

impl<S: PageSize> fmt::Debug for PhysFrame<S> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_fmt(format_args!(
            "PhysFrame[{}]({:#x})",
            S::SIZE_AS_DEBUG_STR,
            self.start_address().as_u64()
        ))
    }
}

impl<S: PageSize> Add<u64> for PhysFrame<S> {
    type Output = Self;
    fn add(self, rhs: u64) -> Self::Output {
        PhysFrame::containing_address(self.start_address() + rhs * u64::from(S::SIZE))
    }
}

impl<S: PageSize> AddAssign<u64> for PhysFrame<S> {
    fn add_assign(&mut self, rhs: u64) {
        *self = self.clone() + rhs;
    }
}

impl<S: PageSize> Sub<u64> for PhysFrame<S> {
    type Output = Self;
    fn sub(self, rhs: u64) -> Self::Output {
        PhysFrame::containing_address(self.start_address() - rhs * u64::from(S::SIZE))
    }
}

impl<S: PageSize> SubAssign<u64> for PhysFrame<S> {
    fn sub_assign(&mut self, rhs: u64) {
        *self = self.clone() - rhs;
    }
}

impl<S: PageSize> Sub<PhysFrame<S>> for PhysFrame<S> {
    type Output = u64;
    fn sub(self, rhs: PhysFrame<S>) -> Self::Output {
        (self.start_address - rhs.start_address) / S::SIZE
    }
}

/// An range of physical memory frames, exclusive the upper bound.
#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(C)]
pub struct PhysFrameRange<S: PageSize = Size4KiB> {
    /// The start of the range, inclusive.
    pub start: PhysFrame<S>,
    /// The end of the range, exclusive.
    pub end: PhysFrame<S>,
}

impl<S: PageSize> PhysFrameRange<S> {
    /// Returns whether the range contains no frames.
    pub fn is_empty(&self) -> bool {
        !(self.start < self.end)
    }
}

impl<S: PageSize> Iterator for PhysFrameRange<S> {
    type Item = PhysFrame<S>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.start < self.end {
            let frame = self.start.clone();
            self.start += 1;
            Some(frame)
        } else {
            None
        }
    }
}

impl From<os_bootinfo::FrameRange> for PhysFrameRange {
    fn from(range: os_bootinfo::FrameRange) -> Self {
        PhysFrameRange {
            start: PhysFrame::from_start_address(PhysAddr::new(range.start_addr())).unwrap(),
            end: PhysFrame::from_start_address(PhysAddr::new(range.end_addr())).unwrap(),
        }
    }
}

impl Into<os_bootinfo::FrameRange> for PhysFrameRange {
    fn into(self) -> os_bootinfo::FrameRange {
        os_bootinfo::FrameRange::new(
            self.start.start_address().as_u64(),
            self.end.start_address().as_u64(),
        )
    }
}

impl<S: PageSize> fmt::Debug for PhysFrameRange<S> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("PhysFrameRange")
            .field("start", &self.start)
            .field("end", &self.end)
            .finish()
    }
}

/// An range of physical memory frames, inclusive the upper bound.
#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(C)]
pub struct PhysFrameRangeInclusive<S: PageSize = Size4KiB> {
    /// The start of the range, inclusive.
    pub start: PhysFrame<S>,
    /// The start of the range, exclusive.
    pub end: PhysFrame<S>,
}

impl<S: PageSize> PhysFrameRangeInclusive<S> {
    /// Returns whether the range contains no frames.
    pub fn is_empty(&self) -> bool {
        !(self.start <= self.end)
    }
}

impl<S: PageSize> Iterator for PhysFrameRangeInclusive<S> {
    type Item = PhysFrame<S>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.start <= self.end {
            let frame = self.start.clone();
            self.start += 1;
            Some(frame)
        } else {
            None
        }
    }
}

impl<S: PageSize> fmt::Debug for PhysFrameRangeInclusive<S> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("PhysFrameRangeInclusive")
            .field("start", &self.start)
            .field("end", &self.end)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn test_page_ranges() {
        let page_size = Size4KiB::SIZE;
        let number = 1000;

        let start_addr = VirtAddr::new(0xdeadbeaf);
        let start: Page = Page::containing_address(start_addr);
        let end = start.clone() + number;

        for (i, page) in (start..end).enumerate() {
            assert_eq!(
                page,
                Page::containing_address(start_addr + page_size * i as u64)
            );
            assert!((i as u64) < number);
        }

        for (i, page) in (start..=end).enumerate() {
            assert_eq!(
                page,
                Page::containing_address(start_addr + page_size * i as u64)
            );
            assert!((i as u64) <= number);
        }
    }
}
