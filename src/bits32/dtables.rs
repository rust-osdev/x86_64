//! Functions to load descriptor tables.
use dtables::DescriptorTablePointer;
use bits32::segmentation::Descriptor;

/// Load GDT table with 32bit descriptors
pub unsafe fn lgdt(gdt: &DescriptorTablePointer<Descriptor>) {
    asm!("lgdt ($0)" :: "r" (gdt) : "memory");
}

/// Load LDT table with 32bit descriptors.
pub unsafe fn lldt(ldt: &DescriptorTablePointer<Descriptor>) {
    asm!("lldt ($0)" :: "r" (ldt) : "memory");
}

/// Load IDT table with 32bit descriptors.
pub unsafe fn lidt(idt: &DescriptorTablePointer<Descriptor>) {
    asm!("lidt ($0)" :: "r" (idt) : "memory");
}

