//! Description of the data-structures for IA-32e paging mode.
use core::convert::{From, Into};
use core::fmt;
use core::ops;

macro_rules! check_flag {
    ($doc:meta, $fun:ident, $flag:expr) => (
        #[$doc]
        pub fn $fun(self) -> bool {
            self.flags().contains($flag)
        }
    )
}

/// A wrapper for a physical address.
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

impl ops::BitAnd for PAddr {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self {
        PAddr(self.0 & rhs.0)
    }
}

impl ops::BitAnd<u64> for PAddr {
    type Output = u64;

    fn bitand(self, rhs: u64) -> Self::Output {
        self.as_u64() & rhs
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

/// A wrapper for a virtual address.
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

impl ops::BitAnd for VAddr {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        VAddr(self.0 & rhs.0)
    }
}

impl ops::BitAnd<usize> for VAddr {
    type Output = usize;

    fn bitand(self, rhs: usize) -> Self::Output {
        self.as_usize() & rhs
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

/// Log2 of base page size (12 bits).
pub const BASE_PAGE_SHIFT: usize = 12;

/// Size of a base page (4 KiB)
pub const BASE_PAGE_SIZE: usize = 4096;

/// Size of a large page (2 MiB)
pub const LARGE_PAGE_SIZE: usize = 1024 * 1024 * 2;

/// Size of a huge page (1 GiB)
pub const HUGE_PAGE_SIZE: usize = 1024 * 1024 * 1024;

/// Size of a region covered by a PML4 Entry (512 GiB)
pub const PML4_SLOT_SIZE: usize = HUGE_PAGE_SIZE * 512;

/// Size of a cache-line
pub const CACHE_LINE_SIZE: usize = 64;

/// A type wrapping a base page with a 4 KiB buffer.
pub struct Page([u8; BASE_PAGE_SIZE]);

/// A type wrapping a large page with a 2 MiB buffer.
pub struct LargePage([u8; LARGE_PAGE_SIZE]);

/// A type wrapping a huge page with a 1 GiB buffer.
pub struct HugePage([u8; HUGE_PAGE_SIZE]);

/// MAXPHYADDR, which is at most 52; (use CPUID for finding system value).
pub const MAXPHYADDR: u64 = 52;

/// Mask to find the physical address of an entry in a page-table.
const ADDRESS_MASK: u64 = ((1 << MAXPHYADDR) - 1) & !0xfff;

/// Page tables have 512 = 4096 / 64 entries.
pub const PAGE_SIZE_ENTRIES: usize = 512;

/// A PML4 table.
pub type PML4 = [PML4Entry; PAGE_SIZE_ENTRIES];

/// A page directory pointer table.
pub type PDPT = [PDPTEntry; PAGE_SIZE_ENTRIES];

/// A page directory.
pub type PD = [PDEntry; PAGE_SIZE_ENTRIES];

/// A page table.
pub type PT = [PTEntry; PAGE_SIZE_ENTRIES];

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

bitflags! {
    /// PML4 configuration bit description.
    #[repr(transparent)]
    pub struct PML4Flags: u64 {
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

/// A PML4 Entry consists of an address and a bunch of flags.
#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct PML4Entry(pub u64);

impl fmt::Debug for PML4Entry {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "PML4Entry {{ {:#x}, {:?} }}",
            self.address(),
            self.flags()
        )
    }
}

impl PML4Entry {
    /// Creates a new PML4Entry.
    ///
    /// # Arguments
    ///
    ///  * `pdpt` - The physical address of the pdpt table.
    ///  * `flags`- Additional flags for the entry.
    pub fn new(pml4: PAddr, flags: PML4Flags) -> PML4Entry {
        let pml4_val = pml4 & ADDRESS_MASK;
        assert!(pml4_val == pml4.into());
        assert!(pml4 % BASE_PAGE_SIZE == 0);
        PML4Entry(pml4_val | flags.bits)
    }

    /// Retrieves the physical address in this entry.
    pub fn address(self) -> PAddr {
        PAddr::from(self.0 & ADDRESS_MASK)
    }

    pub fn flags(self) -> PML4Flags {
        PML4Flags::from_bits_truncate(self.0)
    }

    check_flag!(doc = "Is page present?", is_present, PML4Flags::P);
    check_flag!(doc = "Read/write; if 0, writes may not be allowed to the 512-GByte region, controlled by this entry (see Section 4.6)",
                is_writeable, PML4Flags::RW);
    check_flag!(doc = "User/supervisor; if 0, user-mode accesses are not allowed to the 512-GByte region controlled by this entry.",
                is_user_mode_allowed, PML4Flags::US);
    check_flag!(doc = "Page-level write-through; indirectly determines the memory type used to access the page-directory-pointer table referenced by this entry.",
                is_page_write_through, PML4Flags::PWT);
    check_flag!(doc = "Page-level cache disable; indirectly determines the memory type used to access the page-directory-pointer table referenced by this entry.",
                is_page_level_cache_disabled, PML4Flags::PCD);
    check_flag!(
        doc =
            "Accessed; indicates whether this entry has been used for linear-address translation.",
        is_accessed,
        PML4Flags::A
    );
    check_flag!(doc = "If IA32_EFER.NXE = 1, execute-disable. If 1, instruction fetches are not allowed from the 512-GByte region.",
                is_instruction_fetching_disabled, PML4Flags::XD);
}

bitflags! {
    /// PDPT configuration bit description.
    #[repr(transparent)]
    pub struct PDPTFlags: u64 {
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

/// A PDPT Entry consists of an address and a bunch of flags.
#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct PDPTEntry(pub u64);

impl fmt::Debug for PDPTEntry {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "PDPTEntry {{ {:#x}, {:?} }}",
            self.address(),
            self.flags()
        )
    }
}

impl PDPTEntry {
    /// Creates a new PDPTEntry.
    ///
    /// # Arguments
    ///
    ///  * `pd` - The physical address of the page directory.
    ///  * `flags`- Additional flags for the entry.
    pub fn new(pd: PAddr, flags: PDPTFlags) -> PDPTEntry {
        let pd_val = pd & ADDRESS_MASK;
        assert!(pd_val == pd.into());
        assert!(pd % BASE_PAGE_SIZE == 0);
        PDPTEntry(pd_val | flags.bits)
    }

    /// Retrieves the physical address in this entry.
    pub fn address(self) -> PAddr {
        PAddr::from(self.0 & ADDRESS_MASK)
    }

    /// Returns the flags corresponding to this entry.
    pub fn flags(self) -> PDPTFlags {
        PDPTFlags::from_bits_truncate(self.0)
    }

    check_flag!(doc = "Is page present?", is_present, PDPTFlags::P);
    check_flag!(doc = "Read/write; if 0, writes may not be allowed to the 1-GByte region controlled by this entry.",
                is_writeable, PDPTFlags::RW);
    check_flag!(doc = "User/supervisor; user-mode accesses are not allowed to the 1-GByte region controlled by this entry.",
                is_user_mode_allowed, PDPTFlags::US);
    check_flag!(
        doc = "Page-level write-through.",
        is_page_write_through,
        PDPTFlags::PWT
    );
    check_flag!(
        doc = "Page-level cache disable.",
        is_page_level_cache_disabled,
        PDPTFlags::PCD
    );
    check_flag!(
        doc =
            "Accessed; indicates whether this entry has been used for linear-address translation.",
        is_accessed,
        PDPTFlags::A
    );
    check_flag!(doc = "Indirectly determines the memory type used to access the 1-GByte page referenced by this entry. if not PS this is ignored.",
                is_pat, PDPTFlags::PAT);
    check_flag!(doc = "If IA32_EFER.NXE = 1, execute-disable. If 1, instruction fetches are not allowed from the 512-GByte region.",
                is_instruction_fetching_disabled, PDPTFlags::XD);
    check_flag!(doc = "Page size; if set this entry maps a 1-GByte page; otherwise, this entry references a page directory.",
                is_page, PDPTFlags::PS);
}

/// PD configuration bits description.
bitflags! {
    /// PDPT configuration bit description.
    #[repr(transparent)]
    pub struct PDFlags: u64 {
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

/// A PD Entry consists of an address and a bunch of flags.
#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct PDEntry(pub u64);

impl fmt::Debug for PDEntry {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "PDEntry {{ {:#x}, {:?} }}", self.address(), self.flags())
    }
}

impl PDEntry {
    /// Creates a new PDEntry.
    ///
    /// # Arguments
    ///
    ///  * `pt` - The physical address of the page table.
    ///  * `flags`- Additional flags for the entry.
    pub fn new(pt: PAddr, flags: PDFlags) -> PDEntry {
        let pt_val = pt & ADDRESS_MASK;
        assert!(pt_val == pt.into());
        assert!(pt % BASE_PAGE_SIZE == 0);
        PDEntry(pt_val | flags.bits)
    }

    /// Retrieves the physical address in this entry.
    pub fn address(self) -> PAddr {
        PAddr::from(self.0 & ADDRESS_MASK)
    }

    /// Returns the flags corresponding to this entry.
    pub fn flags(self) -> PDFlags {
        PDFlags::from_bits_truncate(self.0)
    }

    check_flag!(
        doc = "Present; must be 1 to map a 2-MByte page or reference a page table.",
        is_present,
        PDFlags::P
    );
    check_flag!(doc = "Read/write; if 0, writes may not be allowed to the 2-MByte region controlled by this entry",
                is_writeable, PDFlags::RW);
    check_flag!(doc = "User/supervisor; user-mode accesses are not allowed to the 2-MByte region controlled by this entry.",
                is_user_mode_allowed, PDFlags::US);
    check_flag!(
        doc = "Page-level write-through.",
        is_page_write_through,
        PDFlags::PWT
    );
    check_flag!(
        doc = "Page-level cache disable.",
        is_page_level_cache_disabled,
        PDFlags::PCD
    );
    check_flag!(doc = "Accessed; if PS set indicates whether software has accessed the 2-MByte page else indicates whether this entry has been used for linear-address translation.",
                is_accessed, PDFlags::A);
    check_flag!(doc = "Dirty; if PS set indicates whether software has written to the 2-MByte page referenced by this entry else ignored.",
                is_dirty, PDFlags::D);
    check_flag!(doc = "Page size; if set this entry maps a 2-MByte page; otherwise, this entry references a page directory.",
                is_page, PDFlags::PS);
    check_flag!(doc = "Global; if PS && CR4.PGE = 1, determines whether the translation is global; ignored otherwise if not PS this is ignored.",
                is_global, PDFlags::G);
    check_flag!(doc = "Indirectly determines the memory type used to access the 2-MByte page referenced by this entry. if not PS this is ignored.",
                is_pat, PDFlags::PAT);
    check_flag!(doc = "If IA32_EFER.NXE = 1, execute-disable. If 1, instruction fetches are not allowed from the 2-Mbyte region.",
                is_instruction_fetching_disabled, PDFlags::XD);
}

/// PT Entry bits description.
bitflags! {
    /// PT configuration bit description.
    #[repr(transparent)]
    pub struct PTFlags: u64 {
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

/// A PT Entry consists of an address and a bunch of flags.
#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct PTEntry(pub u64);

impl fmt::Debug for PTEntry {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "PTEntry {{ {:#x}, {:?} }}", self.address(), self.flags())
    }
}

impl PTEntry {
    /// Creates a new PTEntry.
    ///
    /// # Arguments
    ///
    ///  * `page` - The physical address of the backing 4 KiB page.
    ///  * `flags`- Additional flags for the entry.
    pub fn new(page: PAddr, flags: PTFlags) -> PTEntry {
        let page_val = page & ADDRESS_MASK;
        assert!(page_val == page.into());
        assert!(page % BASE_PAGE_SIZE == 0);
        PTEntry(page_val | flags.bits)
    }

    /// Retrieves the physical address in this entry.
    pub fn address(self) -> PAddr {
        PAddr::from(self.0 & ADDRESS_MASK)
    }

    /// Returns the flags corresponding to this entry.
    pub fn flags(self) -> PTFlags {
        PTFlags::from_bits_truncate(self.0)
    }

    check_flag!(
        doc = "Present; must be 1 to map a 4-KByte page or reference a page table.",
        is_present,
        PTFlags::P
    );
    check_flag!(doc = "Read/write; if 0, writes may not be allowed to the 4-KByte region controlled by this entry",
                is_writeable, PTFlags::RW);
    check_flag!(doc = "User/supervisor; user-mode accesses are not allowed to the 4-KByte region controlled by this entry.",
                is_user_mode_allowed, PTFlags::US);
    check_flag!(
        doc = "Page-level write-through.",
        is_page_write_through,
        PTFlags::PWT
    );
    check_flag!(
        doc = "Page-level cache disable.",
        is_page_level_cache_disabled,
        PTFlags::PCD
    );
    check_flag!(doc = "Accessed; if PS set indicates whether software has accessed the 4-KByte page else indicates whether this entry has been used for linear-address translation.",
                is_accessed, PTFlags::A);
    check_flag!(doc = "Dirty; if PD_PS set indicates whether software has written to the 4-KByte page referenced by this entry else ignored.",
                is_dirty, PTFlags::D);
    check_flag!(doc = "Global; if PS && CR4.PGE = 1, determines whether the translation is global; ignored otherwise if not PS this is ignored.",
                is_global, PTFlags::G);
    check_flag!(doc = "If IA32_EFER.NXE = 1, execute-disable. If 1, instruction fetches are not allowed from the 4-KByte region.",
                is_instruction_fetching_disabled, PTFlags::XD);
}
