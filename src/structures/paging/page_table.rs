use core::fmt;
use core::ops::{Index, IndexMut};

use super::{PageSize, PhysFrame, Size4KiB};
use addr::PhysAddr;

use usize_conversions::usize_from;
use ux::*;

/// The error returned by the `PageTableEntry::frame` method.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FrameError {
    /// The entry does not have the `PRESENT` flag set, so it isn't currently mapped to a frame.
    FrameNotPresent,
    /// The entry does have the `HUGE_PAGE` flag set. The `frame` method has a standard 4KiB frame
    /// as return type, so a huge frame can't be returned.
    HugeFrame,
}

/// A 64-bit page table entry.
#[derive(Clone)]
#[repr(transparent)]
pub struct PageTableEntry {
    entry: u64,
}

impl PageTableEntry {
    /// Returns whether this entry is zero.
    pub fn is_unused(&self) -> bool {
        self.entry == 0
    }

    /// Sets this entry to zero.
    pub fn set_unused(&mut self) {
        self.entry = 0;
    }

    /// Returns the flags of this entry.
    pub fn flags(&self) -> PageTableFlags {
        PageTableFlags::from_bits_truncate(self.entry)
    }

    /// Returns the physical address mapped by this entry, might be zero.
    pub fn addr(&self) -> PhysAddr {
        PhysAddr::new(self.entry & 0x000fffff_fffff000)
    }

    /// Returns the physical frame mapped by this entry.
    ///
    /// Returns the following errors:
    ///
    /// - `FrameError::FrameNotPresent` if the entry doesn't have the `PRESENT` flag set.
    /// - `FrameError::HugeFrame` if the entry has the `HUGE_PAGE` flag set (for huge pages the
    ///    `addr` function must be used)
    pub fn frame(&self) -> Result<PhysFrame, FrameError> {
        if !self.flags().contains(PageTableFlags::PRESENT) {
            Err(FrameError::FrameNotPresent)
        } else if self.flags().contains(PageTableFlags::HUGE_PAGE) {
            Err(FrameError::HugeFrame)
        } else {
            Ok(PhysFrame::containing_address(self.addr()))
        }
    }

    /// Map the entry to the specified physical address with the specified flags.
    pub fn set_addr(&mut self, addr: PhysAddr, flags: PageTableFlags) {
        assert!(addr.is_aligned(Size4KiB::SIZE));
        self.entry = (addr.as_u64()) | flags.bits();
    }

    /// Map the entry to the specified physical frame with the specified flags.
    pub fn set_frame(&mut self, frame: PhysFrame, flags: PageTableFlags) {
        assert!(!flags.contains(PageTableFlags::HUGE_PAGE));
        self.set_addr(frame.start_address(), flags)
    }

    /// Sets the flags of this entry.
    pub fn set_flags(&mut self, flags: PageTableFlags) {
        self.entry = self.addr().as_u64() | flags.bits();
    }
}

impl fmt::Debug for PageTableEntry {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut f = f.debug_struct("PageTableEntry");
        f.field("addr", &self.addr());
        f.field("flags", &self.flags());
        f.finish()
    }
}

bitflags! {
    /// Possible flags for a page table entry.
    pub struct PageTableFlags: u64 {
        /// Specifies whether the mapped frame or page table is loaded in memory.
        const PRESENT =         1 << 0;
        /// Controls whether writes to the mapped frames are allowed.
        ///
        /// If this bit is unset in a level 1 page table entry, the mapped frame is read-only.
        /// If this bit is unset in a higher level page table entry the complete range of mapped
        /// pages is read-only.
        const WRITABLE =        1 << 1;
        /// Controls whether accesses from userspace (i.e. ring 3) are permitted.
        const USER_ACCESSIBLE = 1 << 2;
        /// If this bit is set, a “write-through” policy is used for the cache, else a “write-back”
        /// policy is used.
        const WRITE_THROUGH =   1 << 3;
        /// Disables caching for the pointed entry is cacheable.
        const NO_CACHE =        1 << 4;
        /// Set by the CPU when the mapped frame or page table is accessed.
        const ACCESSED =        1 << 5;
        /// Set by the CPU on a write to the mapped frame.
        const DIRTY =           1 << 6;
        /// Specifies that the entry maps a huge frame instead of a page table. Only allowed in
        /// P2 or P3 tables.
        const HUGE_PAGE =       1 << 7;
        /// Indicates that the mapping is present in all address spaces, so it isn't flushed from
        /// the TLB on an address space switch.
        const GLOBAL =          1 << 8;
        /// Available to the OS, can be used to store additional data, e.g. custom flags.
        const BIT_9 =           1 << 9;
        /// Available to the OS, can be used to store additional data, e.g. custom flags.
        const BIT_10 =          1 << 10;
        /// Available to the OS, can be used to store additional data, e.g. custom flags.
        const BIT_11 =          1 << 11;
        /// Available to the OS, can be used to store additional data, e.g. custom flags.
        const BIT_52 =          1 << 52;
        /// Available to the OS, can be used to store additional data, e.g. custom flags.
        const BIT_53 =          1 << 53;
        /// Available to the OS, can be used to store additional data, e.g. custom flags.
        const BIT_54 =          1 << 54;
        /// Available to the OS, can be used to store additional data, e.g. custom flags.
        const BIT_55 =          1 << 55;
        /// Available to the OS, can be used to store additional data, e.g. custom flags.
        const BIT_56 =          1 << 56;
        /// Available to the OS, can be used to store additional data, e.g. custom flags.
        const BIT_57 =          1 << 57;
        /// Available to the OS, can be used to store additional data, e.g. custom flags.
        const BIT_58 =          1 << 58;
        /// Available to the OS, can be used to store additional data, e.g. custom flags.
        const BIT_59 =          1 << 59;
        /// Available to the OS, can be used to store additional data, e.g. custom flags.
        const BIT_60 =          1 << 60;
        /// Available to the OS, can be used to store additional data, e.g. custom flags.
        const BIT_61 =          1 << 61;
        /// Available to the OS, can be used to store additional data, e.g. custom flags.
        const BIT_62 =          1 << 62;
        /// Forbid code execution from the mapped frames.
        ///
        /// Can be only used when the no-execute page protection feature is enabled in the EFER
        /// register.
        const NO_EXECUTE =      1 << 63;
    }
}

/// The number of entries in a page table.
const ENTRY_COUNT: usize = 512;

/// Represents a page table.
///
/// Always page-sized.
///
/// This struct implements the `Index` and `IndexMut` traits, so the entries can be accessed
/// through index operations. For example, `page_table[15]` returns the 15th page table entry.
#[repr(transparent)]
pub struct PageTable {
    entries: [PageTableEntry; ENTRY_COUNT],
}

impl PageTable {
    /// Clears all entries.
    pub fn zero(&mut self) {
        for entry in self.entries.iter_mut() {
            entry.set_unused();
        }
    }
}

impl Index<usize> for PageTable {
    type Output = PageTableEntry;

    fn index(&self, index: usize) -> &Self::Output {
        &self.entries[index]
    }
}

impl IndexMut<usize> for PageTable {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.entries[index]
    }
}

impl Index<u9> for PageTable {
    type Output = PageTableEntry;

    fn index(&self, index: u9) -> &Self::Output {
        &self.entries[usize_from(u16::from(index))]
    }
}

impl IndexMut<u9> for PageTable {
    fn index_mut(&mut self, index: u9) -> &mut Self::Output {
        &mut self.entries[usize_from(u16::from(index))]
    }
}

impl fmt::Debug for PageTable {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.entries[..].fmt(f)
    }
}
