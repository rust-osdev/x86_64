//! Description of the data-structures for IA-32e paging mode.

/// Represents a physical memory address
pub type PAddr = u64;
pub type VAddr = usize;

pub const BASE_PAGE_SIZE: u64 = 4096; // 4 KiB
pub const LARGE_PAGE_SIZE: u64 = 1024 * 1024 * 2; // 2 MiB
pub const HUGE_PAGE_SIZE: u64 = 1024 * 1024 * 1024; // 1 GiB
pub const CACHE_LINE_SIZE: usize = 64; // 64 Bytes

/// MAXPHYADDR, which is at most 52; (use CPUID for finding system value).
pub const MAXPHYADDR: u64 = 52;

/// Mask to find the physical address of an entry in a page-table.
const ADDRESS_MASK: u64 = ((1 << MAXPHYADDR) - 1) & !0xfff;

/// A PML4 table.
/// In practice this has only 4 entries but it still needs to be the size of a 4K page.
pub type PML4 = [PML4Entry; 512];

/// A page directory pointer table.
pub type PDPT = [PDPTEntry; 512];

/// A page directory.
pub type PD = [PDEntry; 512];

/// A page table.
pub type PT = [PTEntry; 512];

/// Given virtual address calculate corresponding entry in PML4.
#[inline]
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
    #[repr(transparent)]
    pub struct PML4Entry: u64 {
        /// Present; must be 1 to reference a page-directory-pointer table
        const P       = bit!(0);
        /// Read/write; if 0, writes may not be allowed to the 512-GByte region
        /// controlled by this entry (see Section 4.6)
        const RW      = bit!(1);
        /// User/supervisor; if 0, user-mode accesses are not allowed
        /// to the 512-GByte region controlled by this entry.
        const US      = bit!(2);
        /// Page-level write-through; indirectly determines the memory type used to
        /// access the page-directory-pointer table referenced by this entry.
        const PWT     = bit!(3);
        /// Page-level cache disable; indirectly determines the memory type used to
        /// access the page-directory-pointer table referenced by this entry.
        const PCD     = bit!(4);
        /// Accessed; indicates whether this entry has been used for linear-address translation.
        const A       = bit!(5);
        /// If IA32_EFER.NXE = 1, execute-disable
        /// If 1, instruction fetches are not allowed from the 512-GByte region.
        const XD      = bit!(63);
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
        let pdpt_val = pdpt;
        assert!(pdpt_val % BASE_PAGE_SIZE == 0);
        PML4Entry {
            bits: pdpt_val | flags.bits,
        }
    }

    /// Retrieves the physical address in this entry.
    pub fn get_address(self) -> PAddr {
        self.bits & ADDRESS_MASK
    }

    check_flag!(doc = "Is page present?", is_present, PML4Entry::P);
    check_flag!(doc = "Read/write; if 0, writes may not be allowed to the 512-GByte region, controlled by this entry (see Section 4.6)",
                is_writeable, PML4Entry::RW);
    check_flag!(doc = "User/supervisor; if 0, user-mode accesses are not allowed to the 512-GByte region controlled by this entry.",
                is_user_mode_allowed, PML4Entry::US);
    check_flag!(doc = "Page-level write-through; indirectly determines the memory type used to access the page-directory-pointer table referenced by this entry.",
                is_page_write_through, PML4Entry::PWT);
    check_flag!(doc = "Page-level cache disable; indirectly determines the memory type used to access the page-directory-pointer table referenced by this entry.",
                is_page_level_cache_disabled, PML4Entry::PCD);
    check_flag!(
        doc =
            "Accessed; indicates whether this entry has been used for linear-address translation.",
        is_accessed,
        PML4Entry::A
    );
    check_flag!(doc = "If IA32_EFER.NXE = 1, execute-disable. If 1, instruction fetches are not allowed from the 512-GByte region.",
                is_instruction_fetching_disabled, PML4Entry::XD);
}

/// PDPT Entry bits description.
bitflags! {
    #[repr(transparent)]
    pub struct PDPTEntry: u64 {
        /// Present; must be 1 to map a 1-GByte page or reference a page directory.
        const P       = bit!(0);
        /// Read/write; if 0, writes may not be allowed to the 1-GByte region controlled by this entry
        const RW      = bit!(1);
        /// User/supervisor; user-mode accesses are not allowed to the 1-GByte region controlled by this entry.
        const US      = bit!(2);
        /// Page-level write-through.
        const PWT     = bit!(3);
        /// Page-level cache disable.
        const PCD     = bit!(4);
        /// Accessed; if PS set indicates whether software has accessed the 1-GByte page
        /// else indicates whether this entry has been used for linear-address translation
        const A       = bit!(5);
        /// Dirty; if PS indicates whether software has written to the 1-GByte page referenced by this entry.
        /// else ignored.
        const D       = bit!(6);
        /// Page size; if set this entry maps a 1-GByte page; otherwise, this entry references a page directory.
        /// if not PS this is ignored.
        const PS      = bit!(7);
        /// Global; if PS && CR4.PGE = 1, determines whether the translation is global; ignored otherwise
        /// if not PS this is ignored.
        const G       = bit!(8);
        /// Indirectly determines the memory type used to access the 1-GByte page referenced by this entry.
        const PAT     = bit!(12);
        /// If IA32_EFER.NXE = 1, execute-disable
        /// If 1, instruction fetches are not allowed from the 512-GByte region.
        const XD      = bit!(63);
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
        let pd_val = pd;
        assert!(pd_val % BASE_PAGE_SIZE == 0);
        PDPTEntry {
            bits: pd_val | flags.bits,
        }
    }

    /// Retrieves the physical address in this entry.
    pub fn get_address(self) -> PAddr {
        self.bits & ADDRESS_MASK
    }

    check_flag!(doc = "Is page present?", is_present, PDPTEntry::P);
    check_flag!(doc = "Read/write; if 0, writes may not be allowed to the 1-GByte region controlled by this entry.",
                is_writeable, PDPTEntry::RW);
    check_flag!(doc = "User/supervisor; user-mode accesses are not allowed to the 1-GByte region controlled by this entry.",
                is_user_mode_allowed, PDPTEntry::US);
    check_flag!(
        doc = "Page-level write-through.",
        is_page_write_through,
        PDPTEntry::PWT
    );
    check_flag!(
        doc = "Page-level cache disable.",
        is_page_level_cache_disabled,
        PDPTEntry::PCD
    );
    check_flag!(
        doc =
            "Accessed; indicates whether this entry has been used for linear-address translation.",
        is_accessed,
        PDPTEntry::A
    );
    check_flag!(doc = "Indirectly determines the memory type used to access the 1-GByte page referenced by this entry. if not PS this is ignored.",
                is_pat, PDPTEntry::PAT);
    check_flag!(doc = "If IA32_EFER.NXE = 1, execute-disable. If 1, instruction fetches are not allowed from the 512-GByte region.",
                is_instruction_fetching_disabled, PDPTEntry::XD);
}

/// PD Entry bits description.
bitflags! {
    #[repr(transparent)]
    pub struct PDEntry: u64 {
        /// Present; must be 1 to map a 2-MByte page or reference a page table.
        const P       = bit!(0);
        /// Read/write; if 0, writes may not be allowed to the 2-MByte region controlled by this entry
        const RW      = bit!(1);
        /// User/supervisor; user-mode accesses are not allowed to the 2-MByte region controlled by this entry.
        const US      = bit!(2);
        /// Page-level write-through.
        const PWT     = bit!(3);
        /// Page-level cache disable.
        const PCD     = bit!(4);
        /// Accessed; if PS set indicates whether software has accessed the 2-MByte page
        /// else indicates whether this entry has been used for linear-address translation
        const A       = bit!(5);
        /// Dirty; if PS indicates whether software has written to the 2-MByte page referenced by this entry.
        /// else ignored.
        const D       = bit!(6);
        /// Page size; if set this entry maps a 2-MByte page; otherwise, this entry references a page directory.
        const PS      = bit!(7);
        /// Global; if PS && CR4.PGE = 1, determines whether the translation is global; ignored otherwise
        /// if not PS this is ignored.
        const G       = bit!(8);
        /// Indirectly determines the memory type used to access the 2-MByte page referenced by this entry.
        /// if not PS this is ignored.
        const PAT     = bit!(12);
        /// If IA32_EFER.NXE = 1, execute-disable
        /// If 1, instruction fetches are not allowed from the 512-GByte region.
        const XD      = bit!(63);
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
        let pt_val = pt;
        assert!(pt_val % BASE_PAGE_SIZE == 0);
        PDEntry {
            bits: pt_val | flags.bits,
        }
    }

    /// Retrieves the physical address in this entry.
    pub fn get_address(self) -> PAddr {
        self.bits & ADDRESS_MASK
    }

    check_flag!(
        doc = "Present; must be 1 to map a 2-MByte page or reference a page table.",
        is_present,
        PDEntry::P
    );
    check_flag!(doc = "Read/write; if 0, writes may not be allowed to the 2-MByte region controlled by this entry",
                is_writeable, PDEntry::RW);
    check_flag!(doc = "User/supervisor; user-mode accesses are not allowed to the 2-MByte region controlled by this entry.",
                is_user_mode_allowed, PDEntry::US);
    check_flag!(
        doc = "Page-level write-through.",
        is_page_write_through,
        PDEntry::PWT
    );
    check_flag!(
        doc = "Page-level cache disable.",
        is_page_level_cache_disabled,
        PDEntry::PCD
    );
    check_flag!(doc = "Accessed; if PS set indicates whether software has accessed the 2-MByte page else indicates whether this entry has been used for linear-address translation.",
                is_accessed, PDEntry::A);
    check_flag!(doc = "Dirty; if PS set indicates whether software has written to the 2-MByte page referenced by this entry else ignored.",
                is_dirty, PDEntry::D);
    check_flag!(doc = "Page size; if set this entry maps a 2-MByte page; otherwise, this entry references a page directory.",
                is_page, PDEntry::PS);
    check_flag!(doc = "Global; if PS && CR4.PGE = 1, determines whether the translation is global; ignored otherwise if not PS this is ignored.",
                is_global, PDEntry::G);
    check_flag!(doc = "Indirectly determines the memory type used to access the 2-MByte page referenced by this entry. if not PS this is ignored.",
                is_pat, PDEntry::PAT);
    check_flag!(doc = "If IA32_EFER.NXE = 1, execute-disable. If 1, instruction fetches are not allowed from the 2-Mbyte region.",
                is_instruction_fetching_disabled, PDEntry::XD);
}

/// PT Entry bits description.
bitflags! {
    #[repr(transparent)]
    pub struct PTEntry: u64 {
        /// Present; must be 1 to map a 4-KByte page.
        const P       = bit!(0);
        /// Read/write; if 0, writes may not be allowed to the 4-KByte region controlled by this entry
        const RW      = bit!(1);
        /// User/supervisor; user-mode accesses are not allowed to the 4-KByte region controlled by this entry.
        const US      = bit!(2);
        /// Page-level write-through.
        const PWT     = bit!(3);
        /// Page-level cache disable.
        const PCD     = bit!(4);
        /// Accessed; indicates whether software has accessed the 4-KByte page
        const A       = bit!(5);
        /// Dirty; indicates whether software has written to the 4-KByte page referenced by this entry.
        const D       = bit!(6);
        /// Global; if CR4.PGE = 1, determines whether the translation is global (see Section 4.10); ignored otherwise
        const G       = bit!(8);
        /// If IA32_EFER.NXE = 1, execute-disable
        /// If 1, instruction fetches are not allowed from the 512-GByte region.
        const XD      = bit!(63);
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
        let page_val = page;
        assert!(page_val % BASE_PAGE_SIZE == 0);
        PTEntry {
            bits: page_val | flags.bits,
        }
    }

    /// Retrieves the physical address in this entry.
    pub fn get_address(self) -> PAddr {
        self.bits & ADDRESS_MASK
    }

    check_flag!(
        doc = "Present; must be 1 to map a 4-KByte page or reference a page table.",
        is_present,
        PTEntry::P
    );
    check_flag!(doc = "Read/write; if 0, writes may not be allowed to the 4-KByte region controlled by this entry",
                is_writeable, PTEntry::RW);
    check_flag!(doc = "User/supervisor; user-mode accesses are not allowed to the 4-KByte region controlled by this entry.",
                is_user_mode_allowed, PTEntry::US);
    check_flag!(
        doc = "Page-level write-through.",
        is_page_write_through,
        PTEntry::PWT
    );
    check_flag!(
        doc = "Page-level cache disable.",
        is_page_level_cache_disabled,
        PTEntry::PCD
    );
    check_flag!(doc = "Accessed; if PS set indicates whether software has accessed the 4-KByte page else indicates whether this entry has been used for linear-address translation.",
                is_accessed, PTEntry::A);
    check_flag!(doc = "Dirty; if PD_PS set indicates whether software has written to the 4-KByte page referenced by this entry else ignored.",
                is_dirty, PTEntry::D);
    check_flag!(doc = "Global; if PS && CR4.PGE = 1, determines whether the translation is global; ignored otherwise if not PS this is ignored.",
                is_global, PTEntry::G);
    check_flag!(doc = "If IA32_EFER.NXE = 1, execute-disable. If 1, instruction fetches are not allowed from the 4-KByte region.",
                is_instruction_fetching_disabled, PTEntry::XD);
}
