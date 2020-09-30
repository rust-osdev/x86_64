//! Functions to load GDT, IDT, and TSS structures.

use crate::structures::gdt::SegmentSelector;

pub use crate::structures::DescriptorTablePointer;

/// Load a GDT.
///
/// Use the
/// [`GlobalDescriptorTable`](crate::structures::gdt::GlobalDescriptorTable) struct for a high-level
/// interface to loading a GDT.
///
/// ## Safety
///
/// This function is unsafe because the caller must ensure that the given
/// `DescriptorTablePointer` points to a valid GDT and that loading this
/// GDT is safe.
#[inline]
pub unsafe fn lgdt(gdt: &DescriptorTablePointer) {
    #[cfg(feature = "inline_asm")]
    asm!("lgdt [{}]", in(reg) gdt, options(nostack));

    #[cfg(not(feature = "inline_asm"))]
    crate::asm::x86_64_asm_lgdt(gdt as *const _);
}

/// Load an IDT.
///
/// Use the
/// [`InterruptDescriptorTable`](crate::structures::idt::InterruptDescriptorTable) struct for a high-level
/// interface to loading an IDT.
///
/// ## Safety
///
/// This function is unsafe because the caller must ensure that the given
/// `DescriptorTablePointer` points to a valid IDT and that loading this
/// IDT is safe.
#[inline]
pub unsafe fn lidt(idt: &DescriptorTablePointer) {
    #[cfg(feature = "inline_asm")]
    asm!("lidt [{}]", in(reg) idt, options(nostack));

    #[cfg(not(feature = "inline_asm"))]
    crate::asm::x86_64_asm_lidt(idt as *const _);
}

/// Load the task state register using the `ltr` instruction.
///
/// ## Safety
///
/// This function is unsafe because the caller must ensure that the given
/// `SegmentSelector` points to a valid TSS entry in the GDT and that loading
/// this TSS is safe.
#[inline]
pub unsafe fn load_tss(sel: SegmentSelector) {
    #[cfg(feature = "inline_asm")]
    asm!("ltr {0:x}", in(reg) sel.0, options(nostack, nomem));

    #[cfg(not(feature = "inline_asm"))]
    crate::asm::x86_64_asm_ltr(sel.0)
}
