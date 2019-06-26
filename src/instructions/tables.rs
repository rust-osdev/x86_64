//! Functions to load GDT, IDT, and TSS structures.

use crate::structures::gdt::SegmentSelector;

pub use crate::structures::DescriptorTablePointer;

/// Load a GDT. Use the
/// [`GlobalDescriptorTable`](crate::structures::gdt::GlobalDescriptorTable) struct for a high-level
/// interface to loading a GDT.
#[inline]
pub unsafe fn lgdt(gdt: &DescriptorTablePointer) {
    asm!("lgdt ($0)" :: "r" (gdt) : "memory");
}

/// Load an IDT. Use the
/// [`InterruptDescriptorTable`](crate::structures::idt::InterruptDescriptorTable) struct for a high-level
/// interface to loading an IDT.
#[inline]
pub unsafe fn lidt(idt: &DescriptorTablePointer) {
    asm!("lidt ($0)" :: "r" (idt) : "memory");
}

/// Load the task state register using the `ltr` instruction.
#[inline]
pub unsafe fn load_tss(sel: SegmentSelector) {
    asm!("ltr $0" :: "r" (sel.0));
}
