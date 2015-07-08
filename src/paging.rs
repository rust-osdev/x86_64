/// The focus on this file is to describe the data-structures
/// for IA-32e paging mode.
use core::fmt;

pub type PAddr = u64;
pub type VAddr = usize;

pub const BASE_PAGE_SIZE: u64 = 4096; // 4 KiB
pub const LARGE_PAGE_SIZE: u64 = 1024*1024*2; // 2 MiB
pub const HUGE_PAGE_SIZE: u64 = 1024*1024*1024; // 1 GiB
pub const CACHE_LINE_SIZE: usize = 64; // 64 Bytes

/// MAXPHYADDR, which is at most 52; (use CPUID for finding system value).
pub const MAXPHYADDR: u64 = 52;

/// Mask to find the physical address of an entry in a page-table.
const ADDRESS_MASK: u64 = ((1 << MAXPHYADDR) - 1) & !0xfff;

/// A PML4 table.
/// In practice this has only 4 entries but it still needs to be the size of a 4K page.
pub type PML4  = [PML4Entry; 512];

/// A page directory pointer table.
pub type PDPT  = [PDPTEntry; 512];

/// A page directory.
pub type PD    = [PDEntry; 512];

/// A page table.
pub type PT    = [PTEntry; 512];

/// Given virtual address calculate corresponding entry in PML4.
pub fn pml4_index(addr: VAddr) -> usize {
    (addr >> 39) & 0b111111111
}

/// Given virtual address calculate corresponding entry in PDPT.
#[inline]
pub fn pdpt_index(addr: VAddr) -> usize {
    (addr >> 30) & 0b111111111
}

/// Given virtual address calculate corresponding entry in PD.
#[inline]
pub fn pd_index(addr: VAddr) -> usize {
    (addr >> 21) & 0b111111111
}

/// Given virtual address calculate corresponding entry in PT.
#[inline]
pub fn pt_index(addr: VAddr) -> usize {
    (addr >> 12) & 0b111111111
}

/// PML4 Entry bits description.
bitflags! {
    flags PML4Entry: u64 {
        /// Present; must be 1 to reference a page-directory-pointer table
        const PML4_P       = 0b00000001,
        /// Read/write; if 0, writes may not be allowed to the 512-GByte region
        /// controlled by this entry (see Section 4.6)
        const PML4_RW      = 0b00000010,
        /// User/supervisor; if 0, user-mode accesses are not allowed
        /// to the 512-GByte region controlled by this entry.
        const PML4_US      = 0b00000100,
        /// Page-level write-through; indirectly determines the memory type used to
        /// access the page-directory-pointer table referenced by this entry.
        const PML4_PWT     = 0b00001000,
        /// Page-level cache disable; indirectly determines the memory type used to
        /// access the page-directory-pointer table referenced by this entry.
        const PML4_PCD     = 0b00010000,
        /// Accessed; indicates whether this entry has been used for linear-address translation.
        const PML4_A       = 0b00100000,
        /// If IA32_EFER.NXE = 1, execute-disable
        /// If 1, instruction fetches are not allowed from the 512-GByte region.
        const PML4_XD      = 1 << 63,
    }
}


impl PML4Entry {

    /// Creates a new PML4Entry.
    ///
    /// # Arguments
    ///
    ///  * `pdpt` - The physical address of the pdpt table.
    ///  * `flags`- Additional flags for the entry.
    pub fn new(pdpt: PAddr, flags: PML4Entry) -> PML4Entry {
        assert!(pdpt % BASE_PAGE_SIZE == 0);
        PML4Entry { bits: pdpt | flags.bits }
    }

    /// Retrieves the physical address in this entry.
    pub fn get_address(self) -> PAddr {
        self.bits & ADDRESS_MASK
    }

    /// Convenience function to check if the present bit is set.
    pub fn is_present(self) -> bool {
        self.contains(PML4_P)
    }
}

impl fmt::Debug for PML4Entry {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.bits)
    }
}

/// PDPT Entry bits description.
bitflags! {
    flags PDPTEntry: u64 {
        /// Present; must be 1 to map a 1-GByte page or reference a page directory.
        const PDPT_P       = 0b00000001,
        /// Read/write; if 0, writes may not be allowed to the 1-GByte region controlled by this entry
        const PDPT_RW      = 0b00000010,
        /// User/supervisor; user-mode accesses are not allowed to the 1-GByte region controlled by this entry.
        const PDPT_US      = 0b00000100,
        /// Page-level write-through.
        const PDPT_PWT     = 0b00001000,
        /// Page-level cache disable.
        const PDPT_PCD     = 0b00010000,
        /// Accessed; if PDPT_PS set indicates whether software has accessed the 1-GByte page
        /// else indicates whether this entry has been used for linear-address translation
        const PDPT_A       = 0b00100000,
        /// Dirty; if PDPT_PS indicates whether software has written to the 1-GByte page referenced by this entry.
        /// else ignored.
        const PDPT_D       = 0b01000000,
        /// Page size; if set this entry maps a 1-GByte page; otherwise, this entry references a page directory.
        /// if not PDPT_PS this is ignored.
        const PDPT_PS      = 0b10000000,
        /// Global; if PDPT_PS && CR4.PGE = 1, determines whether the translation is global; ignored otherwise
        /// if not PDPT_PS this is ignored.
        const PDPT_G       = 1<<8,
        /// Indirectly determines the memory type used to access the 1-GByte page referenced by this entry.
        const PDPT_PAT     = 1<<12,
        /// If IA32_EFER.NXE = 1, execute-disable
        /// If 1, instruction fetches are not allowed from the 512-GByte region.
        const PDPT_XD      = 1 << 63,
    }
}

impl PDPTEntry {

    /// Creates a new PDPTEntry.
    ///
    /// # Arguments
    ///
    ///  * `pd` - The physical address of the page directory.
    ///  * `flags`- Additional flags for the entry.
    pub fn new(pd: PAddr, flags: PDPTEntry) -> PDPTEntry {
        assert!(pd % BASE_PAGE_SIZE == 0);
        PDPTEntry { bits: pd | flags.bits }
    }

    /// Retrieves the physical address in this entry.
    pub fn get_address(self) -> PAddr {
        self.bits & ADDRESS_MASK
    }

    /// Convenience function to check if the present bit is set.
    pub fn is_present(self) -> bool {
        self.contains(PDPT_P)
    }
}

impl fmt::Debug for PDPTEntry {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.bits)
    }
}

/// PD Entry bits description.
bitflags! {
    flags PDEntry: u64 {
        /// Present; must be 1 to map a 2-MByte page or reference a page table.
        const PD_P       = 0b00000001,
        /// Read/write; if 0, writes may not be allowed to the 2-MByte region controlled by this entry
        const PD_RW      = 0b00000010,
        /// User/supervisor; user-mode accesses are not allowed to the 2-MByte region controlled by this entry.
        const PD_US      = 0b00000100,
        /// Page-level write-through.
        const PD_PWT     = 0b00001000,
        /// Page-level cache disable.
        const PD_PCD     = 0b00010000,
        /// Accessed; if PD_PS set indicates whether software has accessed the 2-MByte page
        /// else indicates whether this entry has been used for linear-address translation
        const PD_A       = 0b00100000,
        /// Dirty; if PD_PS indicates whether software has written to the 2-MByte page referenced by this entry.
        /// else ignored.
        const PD_D       = 0b01000000,
        /// Page size; if set this entry maps a 2-MByte page; otherwise, this entry references a page directory.
        const PD_PS      = 0b10000000,
        /// Global; if PD_PS && CR4.PGE = 1, determines whether the translation is global; ignored otherwise
        /// if not PD_PS this is ignored.
        const PD_G       = 1<<8,
        /// Indirectly determines the memory type used to access the 2-MByte page referenced by this entry.
        /// if not PD_PS this is ignored.
        const PD_PAT     = 1<<12,
        /// If IA32_EFER.NXE = 1, execute-disable
        /// If 1, instruction fetches are not allowed from the 512-GByte region.
        const PD_XD      = 1 << 63,
    }
}

impl PDEntry {

    /// Creates a new PDEntry.
    ///
    /// # Arguments
    ///
    ///  * `pt` - The physical address of the page table.
    ///  * `flags`- Additional flags for the entry.
    pub fn new(pt: PAddr, flags: PDEntry) -> PDEntry {
        assert!(pt % BASE_PAGE_SIZE == 0);
        PDEntry { bits: pt | flags.bits }
    }

    /// Retrieves the physical address in this entry.
    pub fn get_address(self) -> PAddr {
        self.bits & ADDRESS_MASK
    }

    /// Convenience function to check if the present bit is set.
    pub fn is_present(self) -> bool {
        self.contains(PD_P)
    }
}

impl fmt::Debug for PDEntry {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.bits)
    }
}

/// PT Entry bits description.
bitflags! {
    flags PTEntry: u64 {
        /// Present; must be 1 to map a 4-KByte page.
        const PT_P       = 0b00000001,
        /// Read/write; if 0, writes may not be allowed to the 4-KByte region controlled by this entry
        const PT_RW      = 0b00000010,
        /// User/supervisor; user-mode accesses are not allowed to the 4-KByte region controlled by this entry.
        const PT_US      = 0b00000100,
        /// Page-level write-through.
        const PT_PWT     = 0b00001000,
        /// Page-level cache disable.
        const PT_PCD     = 0b00010000,
        /// Accessed; indicates whether software has accessed the 4-KByte page
        const PT_A       = 0b00100000,
        /// Dirty; indicates whether software has written to the 4-KByte page referenced by this entry.
        const PT_D       = 0b01000000,
        /// Global; if CR4.PGE = 1, determines whether the translation is global (see Section 4.10); ignored otherwise
        const PT_G       = 1<<8,
        /// If IA32_EFER.NXE = 1, execute-disable
        /// If 1, instruction fetches are not allowed from the 512-GByte region.
        const PT_XD      = 1 << 63,
    }
}


impl PTEntry {

    /// Creates a new PTEntry.
    ///
    /// # Arguments
    ///
    ///  * `page` - The physical address of the backing 4 KiB page.
    ///  * `flags`- Additional flags for the entry.
    pub fn new(page: PAddr, flags: PTEntry) -> PTEntry {
        assert!(page % BASE_PAGE_SIZE == 0);
        PTEntry{ bits: page | flags.bits }
    }

    /// Retrieves the physical address in this entry.
    pub fn get_address(self) -> PAddr {
        self.bits & ADDRESS_MASK
    }

    /// Convenience function to check if the present bit is set.
    pub fn is_present(self) -> bool {
        self.contains(PT_P)
    }
}

impl fmt::Debug for PTEntry {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.bits)
    }
}
