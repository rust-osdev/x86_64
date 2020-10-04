//! Provides functions to read and write segment registers.

use crate::structures::gdt::SegmentSelector;

/// Reload code segment register.
///
/// Note this is special since we can not directly move
/// to cs. Instead we push the new segment selector
/// and return value on the stack and use retf
/// to reload cs and continue at 1:.
///
/// ## Safety
///
/// This function is unsafe because the caller must ensure that `sel`
/// is a valid code segment descriptor.
#[inline]
pub unsafe fn set_cs(sel: SegmentSelector) {
    #[cfg(feature = "inline_asm")]
    #[inline(always)]
    unsafe fn inner(sel: SegmentSelector) {
        asm!(
            "push {sel}",
            "lea {tmp}, [1f + rip]",
            "push {tmp}",
            "retfq",
            "1:",
            sel = in(reg) u64::from(sel.0),
            tmp = lateout(reg) _,
        );
    }

    #[cfg(not(feature = "inline_asm"))]
    #[inline(always)]
    unsafe fn inner(sel: SegmentSelector) {
        crate::asm::x86_64_asm_set_cs(u64::from(sel.0))
    }

    inner(sel)
}

/// Reload stack segment register.
///
/// ## Safety
///
/// This function is unsafe because the caller must ensure that `sel`
/// is a valid stack segment descriptor.
#[inline]
pub unsafe fn load_ss(sel: SegmentSelector) {
    #[cfg(feature = "inline_asm")]
    asm!("mov ss, {0:x}", in(reg) sel.0, options(nostack));

    #[cfg(not(feature = "inline_asm"))]
    crate::asm::x86_64_asm_load_ss(sel.0);
}

/// Reload data segment register.
///
/// ## Safety
///
/// This function is unsafe because the caller must ensure that `sel`
/// is a valid data segment descriptor.
#[inline]
pub unsafe fn load_ds(sel: SegmentSelector) {
    #[cfg(feature = "inline_asm")]
    asm!("mov ds, {0:x}", in(reg) sel.0, options(nostack));

    #[cfg(not(feature = "inline_asm"))]
    crate::asm::x86_64_asm_load_ds(sel.0);
}

/// Reload es segment register.
///
/// ## Safety
///
/// This function is unsafe because the caller must ensure that `sel`
/// is a valid extra segment descriptor.
#[inline]
pub unsafe fn load_es(sel: SegmentSelector) {
    #[cfg(feature = "inline_asm")]
    asm!("mov es, {0:x}", in(reg) sel.0, options(nostack));

    #[cfg(not(feature = "inline_asm"))]
    crate::asm::x86_64_asm_load_es(sel.0);
}

/// Reload fs segment register.
///
/// ## Safety
///
/// This function is unsafe because the caller must ensure that `sel`
/// is a valid fs segment descriptor.
#[inline]
pub unsafe fn load_fs(sel: SegmentSelector) {
    #[cfg(feature = "inline_asm")]
    asm!("mov fs, {0:x}", in(reg) sel.0, options(nostack));

    #[cfg(not(feature = "inline_asm"))]
    crate::asm::x86_64_asm_load_fs(sel.0);
}

/// Reload gs segment register.
///
/// ## Safety
///
/// This function is unsafe because the caller must ensure that `sel`
/// is a valid gs segment descriptor.
#[inline]
pub unsafe fn load_gs(sel: SegmentSelector) {
    #[cfg(feature = "inline_asm")]
    asm!("mov gs, {0:x}", in(reg) sel.0, options(nostack));

    #[cfg(not(feature = "inline_asm"))]
    crate::asm::x86_64_asm_load_gs(sel.0);
}

/// Swap `KernelGsBase` MSR and `GsBase` MSR.
///
/// ## Safety
///
/// This function is unsafe because the caller must ensure that the
/// swap operation cannot lead to undefined behavior.
#[inline]
pub unsafe fn swap_gs() {
    #[cfg(feature = "inline_asm")]
    asm!("swapgs", options(nostack));

    #[cfg(not(feature = "inline_asm"))]
    crate::asm::x86_64_asm_swapgs();
}

/// Returns the current value of the code segment register.
#[inline]
pub fn cs() -> SegmentSelector {
    #[cfg(feature = "inline_asm")]
    {
        let segment: u16;
        unsafe { asm!("mov {0:x}, cs", out(reg) segment, options(nostack, nomem)) };
        SegmentSelector(segment)
    }

    #[cfg(not(feature = "inline_asm"))]
    {
        let segment: u16 = unsafe { crate::asm::x86_64_asm_get_cs() };
        SegmentSelector(segment)
    }
}

/// Writes the FS segment base address
///
/// ## Safety
///
/// If `CR4.FSGSBASE` is not set, this instruction will throw an `#UD`.
///
/// The caller must ensure that this write operation has no unsafe side
/// effects, as the FS segment base address is often used for thread
/// local storage.
#[inline]
pub unsafe fn wrfsbase(val: u64) {
    #[cfg(feature = "inline_asm")]
    #[inline(always)]
    unsafe fn inner(val: u64) {
        asm!("wrfsbase {}", in(reg) val, options(nomem, nostack));
    }

    #[cfg(not(feature = "inline_asm"))]
    #[inline(always)]
    unsafe fn inner(val: u64) {
        crate::asm::x86_64_asm_wrfsbase(val)
    }

    inner(val)
}

/// Reads the FS segment base address
///
/// ## Safety
///
/// If `CR4.FSGSBASE` is not set, this instruction will throw an `#UD`.
#[inline]
pub unsafe fn rdfsbase() -> u64 {
    #[cfg(feature = "inline_asm")]
    #[inline(always)]
    unsafe fn inner() -> u64 {
        let val: u64;
        asm!("rdfsbase {}", out(reg) val, options(nomem, nostack));
        val
    }

    #[cfg(not(feature = "inline_asm"))]
    #[inline(always)]
    unsafe fn inner() -> u64 {
        crate::asm::x86_64_asm_rdfsbase()
    }

    inner()
}

/// Writes the GS segment base address
///
/// ## Safety
///
/// If `CR4.FSGSBASE` is not set, this instruction will throw an `#UD`.
///
/// The caller must ensure that this write operation has no unsafe side
/// effects, as the GS segment base address might be in use.
#[inline]
pub unsafe fn wrgsbase(val: u64) {
    #[cfg(feature = "inline_asm")]
    #[inline(always)]
    unsafe fn inner(val: u64) {
        asm!("wrgsbase {}", in(reg) val, options(nomem, nostack))
    }

    #[cfg(not(feature = "inline_asm"))]
    #[inline(always)]
    unsafe fn inner(val: u64) {
        crate::asm::x86_64_asm_wrgsbase(val)
    }

    inner(val)
}

/// Reads the GS segment base address
///
/// ## Safety
///
/// If `CR4.FSGSBASE` is not set, this instruction will throw an `#UD`.
#[inline]
pub unsafe fn rdgsbase() -> u64 {
    #[cfg(feature = "inline_asm")]
    #[inline(always)]
    unsafe fn inner() -> u64 {
        let val: u64;
        asm!("rdgsbase {}", out(reg) val, options(nomem, nostack));
        val
    }

    #[cfg(not(feature = "inline_asm"))]
    #[inline(always)]
    unsafe fn inner() -> u64 {
        crate::asm::x86_64_asm_rdgsbase()
    }

    inner()
}
