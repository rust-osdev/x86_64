use core::fmt;
use core::ops::{Index, IndexMut};

use super::PhysFrame;
use addr::PhysAddr;

use usize_conversions::usize_from;
use ux::*;

#[derive(Clone)]
#[repr(transparent)]
pub struct PageTableEntry(u64);

impl PageTableEntry {
    pub fn is_unused(&self) -> bool {
        self.0 == 0
    }

    pub fn set_unused(&mut self) {
        self.0 = 0;
    }

    pub fn flags(&self) -> PageTableFlags {
        PageTableFlags::from_bits_truncate(self.0)
    }

    pub fn frame(&self) -> Option<PhysFrame> {
        if self.flags().contains(PageTableFlags::PRESENT) {
            let addr = PhysAddr::new(self.0 & 0x000fffff_fffff000);
            Some(PhysFrame::containing_address(addr))
        } else {
            None
        }
    }

    pub fn set(&mut self, frame: PhysFrame, flags: PageTableFlags) {
        self.0 = (frame.start_address().as_u64()) | flags.bits();
    }
}

impl fmt::Debug for PageTableEntry {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut f = f.debug_struct("PageTableEntry");
        f.field("frame", &self.frame());
        f.field("flags", &self.flags());
        f.finish()
    }
}

bitflags! {
    pub struct PageTableFlags: u64 {
        const PRESENT =         1 << 0;
        const WRITABLE =        1 << 1;
        const USER_ACCESSIBLE = 1 << 2;
        const WRITE_THROUGH =   1 << 3;
        const NO_CACHE =        1 << 4;
        const ACCESSED =        1 << 5;
        const DIRTY =           1 << 6;
        const HUGE_PAGE =       1 << 7;
        const GLOBAL =          1 << 8;
        const BIT_9 =           1 << 9;
        const BIT_10 =          1 << 10;
        const BIT_11 =          1 << 11;
        const BIT_52 =          1 << 52;
        const BIT_53 =          1 << 53;
        const BIT_54 =          1 << 54;
        const BIT_55 =          1 << 55;
        const BIT_56 =          1 << 56;
        const BIT_57 =          1 << 57;
        const BIT_58 =          1 << 58;
        const BIT_59 =          1 << 59;
        const BIT_60 =          1 << 60;
        const BIT_61 =          1 << 61;
        const BIT_62 =          1 << 62;
        const NO_EXECUTE =      1 << 63;
    }
}

const ENTRY_COUNT: usize = 512;

#[repr(transparent)]
pub struct PageTable {
    entries: [PageTableEntry; ENTRY_COUNT],
}

impl PageTable {
    pub fn zero(&mut self) {
        for entry in self.entries.iter_mut() {
            entry.set_unused();
        }
    }
}

impl Index<usize> for PageTable {
    type Output = PageTableEntry;

    fn index(&self, index: usize) -> &PageTableEntry {
        &self.entries[index]
    }
}

impl IndexMut<usize> for PageTable {
    fn index_mut(&mut self, index: usize) -> &mut PageTableEntry {
        &mut self.entries[index]
    }
}

impl Index<u9> for PageTable {
    type Output = PageTableEntry;

    fn index(&self, index: u9) -> &PageTableEntry {
        &self.entries[usize_from(u16::from(index))]
    }
}

impl IndexMut<u9> for PageTable {
    fn index_mut(&mut self, index: u9) -> &mut PageTableEntry {
        &mut self.entries[usize_from(u16::from(index))]
    }
}

impl fmt::Debug for PageTable {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.entries[..].fmt(f)
    }
}
