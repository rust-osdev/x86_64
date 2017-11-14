//! Interrupt description and set-up code.

use core::fmt;

use bits64::segmentation::SegmentSelector;
use shared::descriptor::*;
use shared::paging::VAddr;
use shared::PrivilegeLevel;

/// An interrupt gate descriptor.
///
/// See Intel manual 3a for details, specifically section "6.14.1 64-Bit Mode
/// IDT" and "Figure 6-7. 64-Bit IDT Gate Descriptors".
#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
pub struct IdtEntry {
    /// Lower 16 bits of ISR.
    pub base_lo: u16,
    /// Segment selector.
    pub selector: SegmentSelector,
    /// This must always be zero.
    pub ist_index: u8,
    /// Flags.
    pub flags: Flags,
    /// The upper 48 bits of ISR (the last 16 bits must be zero).
    pub base_hi: u64,
    /// Must be zero.
    pub reserved1: u16,
}

pub enum Type {
    InterruptGate,
    TrapGate,
}

impl Type {
    pub fn pack(self) -> Flags {
        match self {
            Type::InterruptGate => FLAGS_TYPE_SYS_NATIVE_INTERRUPT_GATE,
            Type::TrapGate => FLAGS_TYPE_SYS_NATIVE_TRAP_GATE,
        }
    }
}

impl IdtEntry {
    /// A "missing" IdtEntry.
    ///
    /// If the CPU tries to invoke a missing interrupt, it will instead
    /// send a General Protection fault (13), with the interrupt number and
    /// some other data stored in the error code.
    pub const MISSING: IdtEntry = IdtEntry {
        base_lo: 0,
        selector: SegmentSelector::from_raw(0),
        ist_index: 0,
        flags: Flags::BLANK,
        base_hi: 0,
        reserved1: 0,
    };

    /// Create a new IdtEntry pointing at `handler`, which must be a function
    /// with interrupt calling conventions.  (This must be currently defined in
    /// assembly language.)  The `gdt_code_selector` value must be the offset of
    /// code segment entry in the GDT.
    ///
    /// The "Present" flag set, which is the most common case.  If you need
    /// something else, you can construct it manually.
    pub fn new(handler: VAddr, gdt_code_selector: SegmentSelector,
               dpl: PrivilegeLevel, ty: Type, ist_index: u8) -> IdtEntry {
        assert!(ist_index < 0b1000);
        IdtEntry {
            base_lo: ((handler.as_usize() as u64) & 0xFFFF) as u16,
            base_hi: handler.as_usize() as u64 >> 16,
            selector: gdt_code_selector,
            ist_index: ist_index,
            flags: Flags::from_priv(dpl)
                |  ty.pack()
                |  FLAGS_PRESENT,
            reserved1: 0,
        }
    }
}

bitflags!{
    // Taken from Intel Manual Section 4.7 Page-Fault Exceptions.
    pub flags PageFaultError: u32 {
        /// 0: The fault was caused by a non-present page.
        /// 1: The fault was caused by a page-level protection violation
        const PFAULT_ERROR_P = bit!(0),

        /// 0: The access causing the fault was a read.
        /// 1: The access causing the fault was a write.
        const PFAULT_ERROR_WR = bit!(1),

        /// 0: The access causing the fault originated when the processor
        /// was executing in supervisor mode.
        /// 1: The access causing the fault originated when the processor
        /// was executing in user mode.
        const PFAULT_ERROR_US = bit!(2),

        /// 0: The fault was not caused by reserved bit violation.
        /// 1: The fault was caused by reserved bits set to 1 in a page directory.
        const PFAULT_ERROR_RSVD = bit!(3),

        /// 0: The fault was not caused by an instruction fetch.
        /// 1: The fault was caused by an instruction fetch.
        const PFAULT_ERROR_ID = bit!(4),

        /// 0: The fault was not by protection keys.
        /// 1: There was a protection key violation.
        const PFAULT_ERROR_PK = bit!(5),
    }
}

impl fmt::Display for PageFaultError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let p = match self.contains(PFAULT_ERROR_P) {
            false => "The fault was caused by a non-present page.",
            true => "The fault was caused by a page-level protection violation.",
        };
        let wr = match self.contains(PFAULT_ERROR_WR) {
            false => "The access causing the fault was a read.",
            true => "The access causing the fault was a write.",
        };
        let us = match self.contains(PFAULT_ERROR_US) {
            false => {
                "The access causing the fault originated when the processor was executing in \
                 supervisor mode."
            }
            true => {
                "The access causing the fault originated when the processor was executing in user \
                 mode."
            }
        };
        let rsvd = match self.contains(PFAULT_ERROR_RSVD) {
            false => "The fault was not caused by reserved bit violation.",
            true => "The fault was caused by reserved bits set to 1 in a page directory.",
        };
        let id = match self.contains(PFAULT_ERROR_ID) {
            false => "The fault was not caused by an instruction fetch.",
            true => "The fault was caused by an instruction fetch.",
        };

        write!(f, "{}\n{}\n{}\n{}\n{}", p, wr, us, rsvd, id)
    }
}

#[test]
fn bit_macro() {
    assert!(PFAULT_ERROR_PK.bits() == 0b100000);
    assert!(PFAULT_ERROR_ID.bits() == 0b10000);
    assert!(PFAULT_ERROR_RSVD.bits() == 0b1000);
    assert!(PFAULT_ERROR_US.bits() == 0b100);
    assert!(PFAULT_ERROR_WR.bits() == 0b10);
    assert!(PFAULT_ERROR_P.bits() == 0b1);
}
