//! Provides functions to read and write segment registers.

use crate::structures::gdt::SegmentSelector;

/// Reload code segment register.
///
/// Note this is special since we can not directly move
/// to %cs. Instead we push the new segment selector
/// and return value on the stack and use lretq
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
        llvm_asm!("pushq $0; \
              leaq  1f(%rip), %rax; \
              pushq %rax; \
              lretq; \
              1:" :: "ri" (u64::from(sel.0)) : "rax" "memory");
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
    llvm_asm!("movw $0, %ss " :: "r" (sel.0) : "memory");

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
    llvm_asm!("movw $0, %ds " :: "r" (sel.0) : "memory");

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
    llvm_asm!("movw $0, %es " :: "r" (sel.0) : "memory");

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
    llvm_asm!("movw $0, %fs " :: "r" (sel.0) : "memory");

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
    llvm_asm!("movw $0, %gs " :: "r" (sel.0) : "memory");

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
    llvm_asm!("swapgs" ::: "memory" : "volatile");

    #[cfg(not(feature = "inline_asm"))]
    crate::asm::x86_64_asm_swapgs();
}

/// Returns the current value of the code segment register.
#[inline]
pub fn cs() -> SegmentSelector {
    #[cfg(feature = "inline_asm")]
    {
        let segment: u16;
        unsafe { llvm_asm!("mov %cs, $0" : "=r" (segment) ) };
        SegmentSelector(segment)
    }

    #[cfg(not(feature = "inline_asm"))]
    {
        let segment: u16 = unsafe { crate::asm::x86_64_asm_get_cs() };
        SegmentSelector(segment)
    }
}
