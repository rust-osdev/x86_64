//! Functions to load descriptor tables.
use dtables::DescriptorTablePointer;
use bits64::segmentation::Descriptor64;

/// Load GDT table with 64-bits descriptors.
pub unsafe fn lgdt(gdt: &DescriptorTablePointer<Descriptor64>) {
    asm!("lgdt ($0)" :: "r" (gdt) : "memory");
}

/// Load LDT table with 64-bits descriptors.
pub unsafe fn lldt(ldt: &DescriptorTablePointer<Descriptor64>) {
    asm!("lldt ($0)" :: "r" (ldt) : "memory");
}

/// Load IDT table with 64-bits descriptors.
pub unsafe fn lidt(idt: &DescriptorTablePointer<Descriptor64>) {
    asm!("lidt ($0)" :: "r" (idt) : "memory");
}
