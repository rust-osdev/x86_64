//! ENCLU opcode wrappers

use crate::structures::paging::Page;
use crate::structures::sgx::*;
use core::arch::asm;

/// Error codes for EACCEPT
#[derive(Debug)]
pub enum AcceptError {
    /// Unknown error
    Unknown,
    /// CPU cores have not exited from the previous grace period.
    PageNotTracked,
    /// Attributes of the destination page are incorrect.
    PageAttributesMismatch,
}

/// Acknowledge host's request to add a new page or change its permissions.
#[inline]
pub fn accept(secinfo: &SecInfo, dest: Page) -> Result<(), AcceptError> {
    const EACCEPT: usize = 0x05;
    let ret;

    unsafe {
        asm!(
            "xchg       {RBX}, rbx",
            "enclu",
            "mov        rbx, {RBX}",

            RBX = inout(reg) secinfo => _,
            in("rax") EACCEPT,
            in("rcx") dest.start_address().as_u64(),
            lateout("rax") ret,
        );
    }

    match ret {
        0 => Ok(()),
        11 => Err(AcceptError::PageNotTracked),
        19 => Err(AcceptError::PageAttributesMismatch),
        _ => Err(AcceptError::Unknown),
    }
}

/// Error codes for EACCEPTCOPY
#[derive(Debug)]
pub enum AcceptCopyError {
    /// Unknown error
    Unknown,
    /// Attributes of the destination page are incorrect.
    PageAttributesMismatch,
}

/// Acknowledge host's request to add a new page, and populate the page with
/// the given permissions and data.
#[inline]
pub fn accept_copy(secinfo: &SecInfo, dest: Page, src: Page) -> Result<(), AcceptCopyError> {
    pub const EACCEPTCOPY: usize = 0x07;
    let ret;

    unsafe {
        asm!(
            "xchg       {RBX}, rbx",
            "enclu",
            "mov        rbx, {RBX}",

            RBX = inout(reg) secinfo => _,
            in("rax") EACCEPTCOPY,
            in("rcx") dest.start_address().as_u64(),
            in("rdx") src.start_address().as_u64(),
            lateout("rax") ret,
        );
    }

    match ret {
        0 => Ok(()),
        19 => Err(AcceptCopyError::PageAttributesMismatch),
        _ => Err(AcceptCopyError::Unknown),
    }
}

/// Extend page permissions.
#[inline]
pub fn extend_permissions(secinfo: &SecInfo, dest: Page) {
    pub const EMODPE: usize = 0x06;

    unsafe {
        asm!(
            "xchg       {RBX}, rbx",
            "enclu",
            "mov        rbx, {RBX}",

            RBX = inout(reg) secinfo => _,
            in("rax") EMODPE,
            in("rcx") dest.start_address().as_u64(),
        );
    }
}
