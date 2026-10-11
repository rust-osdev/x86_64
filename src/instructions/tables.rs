//! Functions to load GDT, IDT, LDT, and TSS structures.

use crate::VirtAddr;
use crate::structures::gdt::SegmentSelector;
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

/// Get the address of the current GDT.
#[inline]
pub fn sgdt() -> DescriptorTablePointer {
    let mut gdt: DescriptorTablePointer = DescriptorTablePointer {
        limit: 0,
        base: VirtAddr::new(0),
    };
    unsafe {
        asm!("sgdt [{}]", in(reg) &mut gdt, options(nostack, preserves_flags));
    }
    gdt
}

/// Get the address of the current IDT.
#[inline]
pub fn sidt() -> DescriptorTablePointer {
    let mut idt: DescriptorTablePointer = DescriptorTablePointer {
        limit: 0,
        base: VirtAddr::new(0),
    };
    unsafe {
        asm!("sidt [{}]", in(reg) &mut idt, options(nostack, preserves_flags));
    }
    idt
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

/// Load the local descriptor table register using the `lldt` instruction.
///
/// The processor copies the base and limit of the LDT descriptor into the
/// LDTR, so a later change to that GDT entry takes effect only once the
/// selector is loaded again. Unlike [`load_tss`], `lldt` does not write to
/// the descriptor, so the same LDT can be loaded on several CPUs.
///
/// Loading [`SegmentSelector::NULL`] marks the LDTR as invalid. Calling
/// `lldt` with any other selector that does not point to an LDT entry in the
/// GDT results in a `#GP` exception.
///
/// ## Safety
///
/// This function is unsafe because the caller must ensure that the given
/// `SegmentSelector` is null or points to a valid LDT entry in the GDT and
/// that the LDT stays valid for as long as it is loaded.
#[inline]
pub unsafe fn lldt(sel: SegmentSelector) {
    unsafe {
        asm!("lldt {0:x}", in(reg) sel.0, options(readonly, nostack, preserves_flags));
    }
}

/// Get the segment selector of the current LDT using the `sldt` instruction.
///
/// This is the selector last loaded into the LDTR. If its GDT entry changed
/// since, the processor still uses the base and limit it loaded then.
#[inline]
pub fn sldt() -> SegmentSelector {
    let sel: u16;
    unsafe {
        asm!("sldt {0:x}", out(reg) sel, options(nomem, nostack, preserves_flags));
    }
    SegmentSelector(sel)
}
