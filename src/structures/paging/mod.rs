//! Abstractions for page tables and other paging related structures.

pub use self::page_table::*;

use addr::{VirtAddr, PhysAddr};
use core::ops::{Add, AddAssign, Sub, SubAssign};
use ux::*;

mod page_table;

/// The default page size on x86_64.
pub const PAGE_SIZE: u16 = 4096;

/// A virtual 4kB page.
#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct Page {
   number: u64,
}

impl Page {
    /// Returns the page that contains the given virtual address.
    pub fn containing_address(address: VirtAddr) -> Page {
        Page { number: address.as_u64() / u64::from(PAGE_SIZE) }
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

/// A physical 4kB frame.
#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct PhysFrame {
   number: u64,
}

impl PhysFrame {
    /// Returns the frame that contains the given physical address.
    pub fn containing_address(address: PhysAddr) -> PhysFrame {
        PhysFrame { number: address.as_u64() / u64::from(PAGE_SIZE) }
    }

    /// Returns the start address of the page.
    pub fn start_address(&self) -> PhysAddr {
        PhysAddr::new(self.number * u64::from(PAGE_SIZE))
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
