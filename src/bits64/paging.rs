//! Description of the data-structures for IA-32e paging mode.
use core::convert::{From, Into};
use core::fmt;
use core::ops;

#[repr(transparent)]
#[derive(Copy, Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct PAddr(u64);

impl PAddr {
    /// Convert to `u64`
    pub const fn as_u64(&self) -> u64 {
        self.0
    }
    /// Convert from `64`
    pub const fn from_u64(p: u64) -> Self {
        PAddr(p)
    }
}

impl From<u64> for PAddr {
    fn from(num: u64) -> Self {
        PAddr(num)
    }
}

impl Into<u64> for PAddr {
    fn into(self) -> u64 {
        self.0
    }
}

impl ops::Add for PAddr {
    type Output = PAddr;

    fn add(self, rhs: PAddr) -> Self::Output {
        PAddr(self.0 + rhs.0)
    }
}

impl ops::Add<u64> for PAddr {
    type Output = PAddr;

    fn add(self, rhs: u64) -> Self::Output {
        PAddr::from(self.0 + rhs)
    }
}

impl ops::Add<usize> for PAddr {
    type Output = PAddr;

    fn add(self, rhs: usize) -> Self::Output {
        PAddr::from(self.0 + rhs as u64)
    }
}

impl ops::AddAssign for PAddr {
    fn add_assign(&mut self, other: PAddr) {
        *self = PAddr::from(self.0 + other.0);
    }
}

impl ops::AddAssign<u64> for PAddr {
    fn add_assign(&mut self, offset: u64) {
        *self = PAddr::from(self.0 + offset);
    }
}

impl ops::Sub for PAddr {
    type Output = PAddr;

    fn sub(self, rhs: PAddr) -> Self::Output {
        PAddr::from(self.0 - rhs.0)
    }
}

impl ops::Sub<u64> for PAddr {
    type Output = PAddr;

    fn sub(self, rhs: u64) -> Self::Output {
        PAddr::from(self.0 - rhs)
    }
}

impl ops::Sub<usize> for PAddr {
    type Output = PAddr;

    fn sub(self, rhs: usize) -> Self::Output {
        PAddr::from(self.0 - rhs as u64)
    }
}

impl ops::Rem for PAddr {
    type Output = PAddr;

    fn rem(self, rhs: PAddr) -> Self::Output {
        PAddr(self.0 % rhs.0)
    }
}

impl ops::Rem<u64> for PAddr {
    type Output = u64;

    fn rem(self, rhs: u64) -> Self::Output {
        self.0 % rhs
    }
}

impl ops::Rem<usize> for PAddr {
    type Output = usize;

    fn rem(self, rhs: usize) -> Self::Output {
        self.0 as usize % rhs
    }
}

impl ops::BitOr for PAddr {
    type Output = PAddr;

    fn bitor(self, rhs: PAddr) -> Self::Output {
        PAddr(self.0 | rhs.0)
    }
}

impl ops::BitOr<u64> for PAddr {
    type Output = u64;

    fn bitor(self, rhs: u64) -> Self::Output {
        self.0 | rhs
    }
}

impl ops::Shr<u64> for PAddr {
    type Output = u64;

    fn shr(self, rhs: u64) -> Self::Output {
        self.0 >> rhs
    }
}

impl fmt::Binary for PAddr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl fmt::Display for PAddr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl fmt::LowerHex for PAddr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl fmt::Octal for PAddr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl fmt::UpperHex for PAddr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

#[repr(transparent)]
#[derive(Copy, Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct VAddr(u64);

impl VAddr {
    /// Convert from `u64`
    pub const fn from_u64(v: u64) -> Self {
        VAddr(v)
    }

    /// Convert from `usize`
    pub const fn from_usize(v: usize) -> Self {
        VAddr(v as u64)
    }

    /// Convert to `u64`
    pub const fn as_u64(&self) -> u64 {
        self.0
    }

    /// Convert to `usize`
    pub const fn as_usize(&self) -> usize {
        self.0 as usize
    }

    pub fn as_ptr(&self) -> *mut u8 {
        self.0 as *mut u8
    }
}

impl From<u64> for VAddr {
    fn from(num: u64) -> Self {
        VAddr(num)
    }
}

impl Into<u64> for VAddr {
    fn into(self) -> u64 {
        self.0
    }
}

impl From<usize> for VAddr {
    fn from(num: usize) -> Self {
        VAddr(num as u64)
    }
}

impl Into<usize> for VAddr {
    fn into(self) -> usize {
        self.0 as usize
    }
}

impl ops::Add for VAddr {
    type Output = VAddr;

    fn add(self, rhs: VAddr) -> Self::Output {
        VAddr(self.0 + rhs.0)
    }
}

impl ops::Add<u64> for VAddr {
    type Output = VAddr;

    fn add(self, rhs: u64) -> Self::Output {
        VAddr(self.0 + rhs)
    }
}

impl ops::Add<usize> for VAddr {
    type Output = VAddr;

    fn add(self, rhs: usize) -> Self::Output {
        VAddr::from(self.0 + rhs as u64)
    }
}

impl ops::Sub for VAddr {
    type Output = VAddr;

    fn sub(self, rhs: VAddr) -> Self::Output {
        VAddr::from(self.0 - rhs.0)
    }
}

impl ops::Sub<u64> for VAddr {
    type Output = VAddr;

    fn sub(self, rhs: u64) -> Self::Output {
        VAddr::from(self.0 - rhs)
    }
}

impl ops::Sub<usize> for VAddr {
    type Output = VAddr;

    fn sub(self, rhs: usize) -> Self::Output {
        VAddr::from(self.0 - rhs as u64)
    }
}

impl ops::Rem for VAddr {
    type Output = VAddr;

    fn rem(self, rhs: VAddr) -> Self::Output {
        VAddr(self.0 % rhs.0)
    }
}

impl ops::Rem<u64> for VAddr {
    type Output = u64;

    fn rem(self, rhs: Self::Output) -> Self::Output {
        self.0 % rhs
    }
}

impl ops::Rem<usize> for VAddr {
    type Output = usize;

    fn rem(self, rhs: Self::Output) -> Self::Output {
        self.as_usize() % rhs
    }
}

impl ops::BitOr for VAddr {
    type Output = VAddr;

    fn bitor(self, rhs: VAddr) -> VAddr {
        VAddr(self.0 | rhs.0)
    }
}

impl ops::BitOr<u64> for VAddr {
    type Output = u64;

    fn bitor(self, rhs: Self::Output) -> Self::Output {
        self.0 | rhs
    }
}

impl ops::BitOr<usize> for VAddr {
    type Output = usize;

    fn bitor(self, rhs: Self::Output) -> Self::Output {
        self.as_usize() | rhs
    }
}

impl ops::Shr<u64> for VAddr {
    type Output = u64;

    fn shr(self, rhs: Self::Output) -> Self::Output {
        self.0 >> rhs
    }
}

impl ops::Shr<usize> for VAddr {
    type Output = usize;

    fn shr(self, rhs: Self::Output) -> Self::Output {
        self.as_usize() >> rhs
    }
}

impl fmt::Binary for VAddr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl fmt::Display for VAddr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl fmt::LowerHex for VAddr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl fmt::Octal for VAddr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl fmt::UpperHex for VAddr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

pub const BASE_PAGE_SHIFT: usize = 12;
pub const BASE_PAGE_SIZE: usize = 4096; // 4 KiB
pub const LARGE_PAGE_SIZE: usize = 1024 * 1024 * 2; // 2 MiB
pub const HUGE_PAGE_SIZE: usize = 1024 * 1024 * 1024; // 1 GiB
pub const CACHE_LINE_SIZE: usize = 64; // 64 Bytes

pub struct Page([u8; BASE_PAGE_SIZE]);
pub struct LargePage([u8; LARGE_PAGE_SIZE]);
pub struct HugePage([u8; HUGE_PAGE_SIZE]);

/// MAXPHYADDR, which is at most 52; (use CPUID for finding system value).
pub const MAXPHYADDR: u64 = 52;

/// Mask to find the physical address of an entry in a page-table.
const ADDRESS_MASK: u64 = ((1 << MAXPHYADDR) - 1) & !0xfff;

/// A PML4 table.
pub type PML4 = [PML4Entry; 512];

/// A page directory pointer table.
pub type PDPT = [PDPTEntry; 512];

/// A page directory.
pub type PD = [PDEntry; 512];

/// A page table.
pub type PT = [PTEntry; 512];

/// Given virtual address calculate corresponding entry in PML4.
#[cfg(target_arch = "x86_64")]
#[inline]
pub fn pml4_index(addr: VAddr) -> usize {
    (addr >> 39usize) & 0b111111111
}

/// Given virtual address calculate corresponding entry in PDPT.
#[inline]
pub fn pdpt_index(addr: VAddr) -> usize {
    (addr >> 30usize) & 0b111111111
}

/// Given virtual address calculate corresponding entry in PD.
#[inline]
pub fn pd_index(addr: VAddr) -> usize {
    (addr >> 21usize) & 0b111111111
}

/// Given virtual address calculate corresponding entry in PT.
#[inline]
pub fn pt_index(addr: VAddr) -> usize {
    (addr >> 12usize) & 0b111111111
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
        PAddr::from(self.bits & ADDRESS_MASK)
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
        PAddr::from(self.bits & ADDRESS_MASK)
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
        PAddr::from(self.bits & ADDRESS_MASK)
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
        PAddr::from(self.bits & ADDRESS_MASK)
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
