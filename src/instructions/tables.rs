//! Functions to load GDT, IDT, and TSS structures.

use crate::structures::gdt::SegmentSelector;
use crate::VirtAddr;
#[cfg(feature = "inline_asm")]
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
    #[cfg(feature = "inline_asm")]
    unsafe {
        asm!("lgdt [{}]", in(reg) gdt, options(readonly, nostack, preserves_flags));
    }

    #[cfg(not(feature = "inline_asm"))]
    unsafe {
        crate::asm::x86_64_asm_lgdt(gdt as *const _);
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
    #[cfg(feature = "inline_asm")]
    unsafe {
        asm!("lidt [{}]", in(reg) idt, options(readonly, nostack, preserves_flags));
    }

    #[cfg(not(feature = "inline_asm"))]
    unsafe {
        crate::asm::x86_64_asm_lidt(idt as *const _);
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
        #[cfg(feature = "inline_asm")]
        asm!("sgdt [{}]", in(reg) &mut gdt, options(nostack, preserves_flags));

        #[cfg(not(feature = "inline_asm"))]
        crate::asm::x86_64_asm_sgdt(&mut gdt as *mut _);
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
        #[cfg(feature = "inline_asm")]
        asm!("sidt [{}]", in(reg) &mut idt, options(nostack, preserves_flags));

        #[cfg(not(feature = "inline_asm"))]
        crate::asm::x86_64_asm_sidt(&mut idt as *mut _);
    }
    idt
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
    unsafe {
        asm!("ltr {0:x}", in(reg) sel.0, options(nomem, nostack, preserves_flags));
    }

    #[cfg(not(feature = "inline_asm"))]
    unsafe {
        crate::asm::x86_64_asm_ltr(sel.0);
    }
}
