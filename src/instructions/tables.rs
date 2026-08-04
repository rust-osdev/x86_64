//! Functions to load GDT, IDT, and TSS structures.

use crate::structures::gdt::SegmentSelector;
use crate::VirtAddr;
use core::arch::asm;

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
    unsafe {
        asm!("lgdt [{}]", in(reg) gdt, options(readonly, nostack, preserves_flags));
    }
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
    unsafe {
        asm!("lidt [{}]", in(reg) idt, options(readonly, nostack, preserves_flags));
    }
}

/// A raw descriptor-table register value.
///
/// Assembly writes plain integers here before the base is checked and wrapped in a semantic type.
#[repr(C, packed(2))]
struct RawDescriptorTablePointer {
    limit: u16,
    base: u64,
}

#[inline]
fn read_raw_gdt() -> RawDescriptorTablePointer {
    let mut pointer = RawDescriptorTablePointer { limit: 0, base: 0 };
    unsafe {
        asm!("sgdt [{}]", in(reg) &mut pointer, options(nostack, preserves_flags));
    }
    pointer
}

#[inline]
fn read_raw_idt() -> RawDescriptorTablePointer {
    let mut pointer = RawDescriptorTablePointer { limit: 0, base: 0 };
    unsafe {
        asm!("sidt [{}]", in(reg) &mut pointer, options(nostack, preserves_flags));
    }
    pointer
}

/// Get the address of the current GDT.
#[inline]
pub fn sgdt() -> DescriptorTablePointer {
    let raw = read_raw_gdt();
    DescriptorTablePointer {
        limit: raw.limit,
        base: VirtAddr::new(raw.base),
    }
}

/// Get the address of the current IDT.
#[inline]
pub fn sidt() -> DescriptorTablePointer {
    let raw = read_raw_idt();
    DescriptorTablePointer {
        limit: raw.limit,
        base: VirtAddr::new(raw.base),
    }
}

/// Load the task state register using the `ltr` instruction.
///
/// Note that loading a TSS segment selector marks the corresponding TSS
/// Descriptor in the GDT as "busy", preventing it from being loaded again
/// (either on this CPU or another CPU). TSS structures (including Descriptors
/// and Selectors) should generally be per-CPU. See
/// [`tss_segment`](crate::structures::gdt::Descriptor::tss_segment)
/// for more information.
///
/// Calling `load_tss` with a busy TSS selector results in a `#GP` exception.
///
/// ## Safety
///
/// This function is unsafe because the caller must ensure that the given
/// `SegmentSelector` points to a valid TSS entry in the GDT and that the
/// corresponding data in the TSS is valid.
#[inline]
pub unsafe fn load_tss(sel: SegmentSelector) {
    unsafe {
        asm!("ltr {0:x}", in(reg) sel.0, options(nostack, preserves_flags));
    }
}
