//! Low level functions for special x86 instructions.

use segmentation;

/// Enable hardware interrupts using the `sti` instruction.
pub unsafe fn enable_interrupts() {
    asm!("sti");
}

/// Disable hardware interrupts using the `cli` instruction.
pub unsafe fn disable_interrupts() {
    asm!("cli");
}

/// Generate a software interrupt.
/// This is a macro because the argument needs to be an immediate.
#[macro_export]
macro_rules! int {
    ( $x:expr ) => {
        {
            asm!("int $0" :: "N" ($x));
        }
    };
}

/// A struct describing a pointer to a descriptor table (GDT / IDT).
/// This is in a format suitable for giving to 'lgdt' or 'lidt'.
#[derive(Debug)]
#[repr(C, packed)]
pub struct DescriptorTablePointer {
    /// Size of the DT.
    pub limit: u16,
    /// Pointer to the memory region containing the DT.
    pub base: u64,
}

/// Load GDT table.
pub unsafe fn lgdt(gdt: &DescriptorTablePointer) {
    asm!("lgdt ($0)" :: "r" (gdt) : "memory");
}

/// Load LDT table.
pub unsafe fn lldt(ldt: &DescriptorTablePointer) {
    asm!("lldt ($0)" :: "r" (ldt) : "memory");
}

/// Load IDT table.
pub unsafe fn lidt(idt: &DescriptorTablePointer) {
    asm!("lidt ($0)" :: "r" (idt) : "memory");
}

/// Load the task state register.
pub unsafe fn load_tss(sel: segmentation::SegmentSelector) {
    asm!("ltr $0" :: "r" (sel.bits()));
}

/// Halts the CPU by executing the `hlt` instruction.
#[inline(always)]
pub unsafe fn halt() {
    asm!("hlt" :::: "volatile");
}

/// Read time stamp counters

/// Read the time stamp counter using the `RDTSC` instruction.
///
/// The `RDTSC` instruction is not a serializing instruction.
/// It does not necessarily wait until all previous instructions
/// have been executed before reading the counter. Similarly,
/// subsequent instructions may begin execution before the
/// read operation is performed. If software requires `RDTSC` to be
/// executed only after all previous instructions have completed locally,
/// it can either use `RDTSCP` or execute the sequence `LFENCE;RDTSC`.
pub fn rdtsc() -> u64 {
    let low: u32;
    let high: u32;
    unsafe {
        asm!("rdtsc" : "={eax}" (low), "={edx}" (high));
    }
    ((u64::from(high)) << 32) | (u64::from(low))
}

/// Read the time stamp counter using the `RDTSCP` instruction.
///
/// The `RDTSCP` instruction waits until all previous instructions
/// have been executed before reading the counter.
/// However, subsequent instructions may begin execution
/// before the read operation is performed.
///
/// Volatile is used here because the function may be used to act as
/// an instruction barrier.
pub fn rdtscp() -> u64 {
    let low: u32;
    let high: u32;
    unsafe {
        asm!("rdtscp" : "={eax}" (low), "={edx}" (high) ::: "volatile");
    }
    ((high as u64) << 32) | (low as u64)
}
