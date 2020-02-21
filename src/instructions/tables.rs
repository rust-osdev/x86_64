//! Functions to load GDT, IDT, and TSS structures.

use crate::structures::gdt::SegmentSelector;

pub use crate::structures::DescriptorTablePointer;

/// Load a GDT. Use the
/// [`GlobalDescriptorTable`](crate::structures::gdt::GlobalDescriptorTable) struct for a high-level
/// interface to loading a GDT.
#[inline]
pub unsafe fn lgdt(gdt: &DescriptorTablePointer) {
    #[cfg(feature = "inline_asm")]
    asm!("lgdt ($0)" :: "r" (gdt) : "memory");

    #[cfg(not(feature = "inline_asm"))]
    crate::asm::x86_64_asm_lgdt(gdt as *const _);
}

/// Load an IDT. Use the
/// [`InterruptDescriptorTable`](crate::structures::idt::InterruptDescriptorTable) struct for a high-level
/// interface to loading an IDT.
#[inline]
pub unsafe fn lidt(idt: &DescriptorTablePointer) {
    #[cfg(feature = "inline_asm")]
    asm!("lidt ($0)" :: "r" (idt) : "memory");

    #[cfg(not(feature = "inline_asm"))]
    crate::asm::x86_64_asm_lidt(idt as *const _);
}

/// Load the task state register using the `ltr` instruction.
#[inline]
pub unsafe fn load_tss(sel: SegmentSelector) {
    #[cfg(feature = "inline_asm")]
    asm!("ltr $0" :: "r" (sel.0));

    #[cfg(not(feature = "inline_asm"))]
    crate::asm::x86_64_asm_ltr(sel.0)
}
