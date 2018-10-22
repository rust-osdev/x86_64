//! Functions to load GDT, IDT, and TSS structures.

use structures::gdt::SegmentSelector;

pub use structures::DescriptorTablePointer;

/// Load GDT table.
pub unsafe fn lgdt(gdt: &DescriptorTablePointer) {
    asm!("lgdt ($0)" :: "r" (gdt) : "memory");
}

/// Load IDT table.
pub unsafe fn lidt(idt: &DescriptorTablePointer) {
    asm!("lidt ($0)" :: "r" (idt) : "memory");
}

/// Load the task state register using the `ltr` instruction.
pub unsafe fn load_tss(sel: SegmentSelector) {
    asm!("ltr $0" :: "r" (sel.0));
}
