//! Interrupt description and set-up code.

use shared::descriptor::*;
use shared::paging::VAddr;
use shared::PrivilegeLevel;

/// An interrupt gate or trap gate descriptor.
///
/// See Intel manual 3a for details, specifically section 6.11.
#[derive(Debug, Copy, Clone)]
#[repr(C, packed)]
pub struct IdtEntry {
    /// Lower 16 bits of ISR.
    pub offset_lo: u16,
    /// Segment selector.
    pub selector: u16,
    /// This must always be zero.
    pub reserved: u8,
    /// flags.
    pub flags: Flags,
    /// The upper 16 bits of ISR.
    pub offset_hi: u16
}

impl IdtEntry {
    pub const MISSING: IdtEntry = IdtEntry {
        offset_lo: 0,
        selector: 0,
        reserved: 0,
        flags: Flags::BLANK,
        offset_hi: 0
    };

    /// Create a new IdtEntry pointing at `handler`, which must be a function
    /// with interrupt calling conventions.  (This must be currently defined in
    /// assembly language.)  The `gdt_code_selector` value must be the offset of
    /// code segment entry in the GDT.
    ///
    /// The "Present" flag set, which is the most common case.  If you need
    /// something else, you can construct it manually.
    pub const fn new(handler: VAddr, gdt_code_selector: u16,
                     dpl: PrivilegeLevel, block: bool) -> IdtEntry {
        IdtEntry {
            offset_lo: ((handler.as_usize() as u32) & 0xFFFF) as u16,
            offset_hi: ((handler.as_usize() as u32 & 0xFFFF0000) >> 16) as u16,
            selector: gdt_code_selector,
            reserved: 0,
            // Nice bitflags operations don't work in const fn, hence these
            // ad-hoc methods.
            flags: Flags::from_priv(dpl)
                .const_or(FLAGS_TYPE_SYS_NATIVE_INTERRUPT_GATE
                          .const_mux(FLAGS_TYPE_SYS_NATIVE_TRAP_GATE,
                                        block))
                .const_or(FLAGS_PRESENT),
        }
    }
}
