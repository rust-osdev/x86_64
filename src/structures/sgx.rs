//! Provides architectural structures for Intel SGX.

bitflags::bitflags! {
    /// Page permissions and wait state
    #[repr(transparent)]
    pub struct PageFlags: u8 {
        /// Read access
        const READ = 1 << 0;
        /// Write access
        const WRITE = 1 << 1;
        /// Execution access
        const EXEC = 1 << 2;
        /// EAUG waiting for EACCEPT or EACCEPTCOPY
        const PENDING = 1 << 3;
        /// EMODT waiting for EACCEPT
        const MODIFIED = 1 << 4;
        /// EMODPR waiting for EACCEPT
        const RESTRICTED = 1 << 5;
    }
}

/// Page type
#[derive(Copy, Clone, Debug)]
#[repr(u8)]
#[non_exhaustive]
pub enum PageType {
    /// SGX Enclave Control Structure (SECS)
    Secs = 0,
    /// Thread Control Structure (TCS)
    Tcs = 1,
    /// Regular page
    Regular = 2,
    /// Version Array (VA) page
    VersionArray = 3,
    /// Removable from a running enclave
    Trimmed = 4,
    /// The first page of a shadow stack
    ShadowStackFirst = 5,
    /// A shadow stack page
    ShadowStackRest = 6,
}

/// Page state
#[derive(Copy, Clone, Debug)]
#[repr(C, align(64))]
pub struct SecInfo {
    page_flags: PageFlags,
    page_type: PageType,
    reserved: [u8; 62],
}

impl SecInfo {
    /// Create a new instance.
    #[inline]
    pub const fn new(page_type: PageType, page_flags: PageFlags) -> SecInfo {
        SecInfo {
            page_flags,
            page_type,
            reserved: [0; 62],
        }
    }

    /// Return page type of the page.
    pub fn page_type(&self) -> PageType {
        self.page_type
    }

    /// Return state flags of the page.
    pub fn page_flags(&self) -> PageFlags {
        self.page_flags
    }
}
