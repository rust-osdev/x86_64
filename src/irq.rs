//! Interrupt description and set-up code.

use VirtualAddress;
use segmentation::{self, SegmentSelector};
use core::fmt;
use bit_field::BitField;

/// An Interrupt Descriptor Table entry.
///
/// See AMD64 Vol 2, Section 4.8.4 for details.
#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
pub struct IdtEntry {
    /// Lower 16 bits of ISR pointer.
    pub pointer_low: u16,
    /// Segment selector.
    pub selector: SegmentSelector,
    // Flags and IST index.
    pub options: IdtEntryOptions,
    /// Bits 16-31 of ISR pointer.
    pub pointer_middle: u16,
    /// Bits 32-63 of ISR pointer.
    pub pointer_high: u32,
    /// Must be zero.
    reserved: u32,
}

impl IdtEntry {
    /// A "missing" IdtEntry.
    ///
    /// If the CPU tries to invoke a missing interrupt, it will instead
    /// send a Segment-Not-Present Exception (11), with the segment descriptor as error code.
    pub fn missing() -> IdtEntry {
        IdtEntry {
            selector: SegmentSelector::new(0, ::PrivilegeLevel::Ring0),
            pointer_low: 0,
            pointer_middle: 0,
            pointer_high: 0,
            options: IdtEntryOptions::minimal(),
            reserved: 0,
        }
    }

    /// Create a new IdtEntry pointing at `handler`, which must be a function
    /// with interrupt calling conventions.  (This must be currently defined in
    /// assembly language.)  The `gdt_code_selector` value must be the offset of
    /// code segment entry in the GDT.
    ///
    /// This function sets the "Present" flag, which is the most common case. It also
    /// sets the GDT selector to the currently active code segment. All defaults can
    /// be overwritten through the provided methods.
    pub fn new(handler: VirtualAddress) -> IdtEntry {
        let pointer = handler.as_usize();
        let selector = segmentation::cs();
        let options = IdtEntryOptions::new();

        IdtEntry {
            selector: selector,
            pointer_low: pointer as u16,
            pointer_middle: (pointer >> 16) as u16,
            pointer_high: (pointer >> 32) as u32,
            options: options,
            reserved: 0,
        }
    }
}

/// Describes options of an IDT entry.
#[derive(Debug, Clone, Copy)]
pub struct IdtEntryOptions(u16);

impl IdtEntryOptions {
    /// A minimal set of flags. Only the bits to define a interrupt gate are set.
    fn minimal() -> Self {
        let mut options = 0;
        options.set_range(9..12, 0b111); // 'must-be-one' bits
        IdtEntryOptions(options)
    }

    /// A default set of options that includes the `present` bit and disables interrupts (trap gate).
    fn new() -> Self {
        let mut options = Self::minimal();
        options.set_present(true).disable_interrupts(true);
        options
    }

    /// Update the `present` bit.
    pub fn set_present(&mut self, present: bool) -> &mut Self {
        self.0.set_bit(15, present);
        self
    }

    /// Control if interrupts should be disabled when the handler function is called.
    pub fn disable_interrupts(&mut self, disable: bool) -> &mut Self {
        self.0.set_bit(8, !disable);
        self
    }

    /// Set the minimal privilege level required to invoke this interrupt.
    #[allow(dead_code)]
    pub fn set_privilege_level(&mut self, dpl: u8) -> &mut Self {
        self.0.set_range(13..15, dpl.into());
        self
    }

    /// Set the IST index. If `None` is passed, the stack switching mechanism is disabled. If
    /// `Some(i)` is passed, the CPU will switch to the i-th stack in the IST of the loaded TSS
    /// (index starts at 0).
    #[allow(dead_code)]
    pub fn set_stack_index(&mut self, index: Option<u8>) -> &mut Self {
        let value = match index {
            Some(i) => (i + 1).into(), // the IST index field starts at index 1
            None => 0,
        };
        self.0.set_range(0..3, value);
        self
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

/// x86 Exception description (see also Intel Vol. 3a Chapter 6).
#[derive(Debug)]
pub struct InterruptDescription {
    pub vector: u8,
    pub mnemonic: &'static str,
    pub description: &'static str,
    pub irqtype: &'static str,
    pub source: &'static str,
}

impl fmt::Display for InterruptDescription {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,
               "{} ({}, vec={}) {}",
               self.mnemonic,
               self.irqtype,
               self.vector,
               self.description)
    }
}


/// x86 External Interrupts (1-16).
pub static EXCEPTIONS: [InterruptDescription; 21] =
    [InterruptDescription {
         vector: 0,
         mnemonic: "#DE",
         description: "Divide Error",
         irqtype: "Fault",
         source: "DIV and IDIV instructions.",
     },
     InterruptDescription {
         vector: 1,
         mnemonic: "#DB",
         description: "Debug",
         irqtype: "Fault/ Trap",
         source: "Debug condition",
     },
     InterruptDescription {
         vector: 2,
         mnemonic: "NMI",
         description: "Nonmaskable Interrupt",
         irqtype: "Interrupt",
         source: "Nonmaskable external interrupt.",
     },
     InterruptDescription {
         vector: 3,
         mnemonic: "#BP",
         description: "Breakpoint",
         irqtype: "Trap",
         source: "INT 3 instruction.",
     },
     InterruptDescription {
         vector: 4,
         mnemonic: "#OF",
         description: "Overflow",
         irqtype: "Trap",
         source: "INTO instruction.",
     },
     InterruptDescription {
         vector: 5,
         mnemonic: "#BR",
         description: "BOUND Range Exceeded",
         irqtype: "Fault",
         source: "BOUND instruction.",
     },
     InterruptDescription {
         vector: 6,
         mnemonic: "#UD",
         description: "Invalid Opcode (Undefined Opcode)",
         irqtype: "Fault",
         source: "UD2 instruction or reserved opcode.",
     },
     InterruptDescription {
         vector: 7,
         mnemonic: "#NM",
         description: "Device Not Available (No Math Coprocessor)",
         irqtype: "Fault",
         source: "Floating-point or WAIT/FWAIT instruction.",
     },
     InterruptDescription {
         vector: 8,
         mnemonic: "#DF",
         description: "Double Fault",
         irqtype: "Abort",
         source: "Any instruction that can generate an exception, an NMI, or an INTR.",
     },
     InterruptDescription {
         vector: 9,
         mnemonic: "",
         description: "Coprocessor Segment Overrun",
         irqtype: "Fault",
         source: "Floating-point instruction.",
     },
     InterruptDescription {
         vector: 10,
         mnemonic: "#TS",
         description: "Invalid TSS",
         irqtype: "Fault",
         source: "Task switch or TSS access.",
     },
     InterruptDescription {
         vector: 11,
         mnemonic: "#NP",
         description: "Segment Not Present",
         irqtype: "Fault",
         source: "Loading segment registers or accessing system segments.",
     },
     InterruptDescription {
         vector: 12,
         mnemonic: "#SS",
         description: "Stack-Segment Fault",
         irqtype: "Fault",
         source: "Stack operations and SS register loads.",
     },
     InterruptDescription {
         vector: 13,
         mnemonic: "#GP",
         description: "General Protection",
         irqtype: "Fault",
         source: "Any memory reference and other protection checks.",
     },
     InterruptDescription {
         vector: 14,
         mnemonic: "#PF",
         description: "Page Fault",
         irqtype: "Fault",
         source: "Any memory reference.",
     },
     InterruptDescription {
         vector: 15,
         mnemonic: "",
         description: "RESERVED",
         irqtype: "",
         source: "None.",
     },
     InterruptDescription {
         vector: 16,
         mnemonic: "#MF",
         description: "x87 FPU Floating-Point",
         irqtype: "Fault",
         source: "x87 FPU instructions.",
     },
     InterruptDescription {
         vector: 17,
         mnemonic: "#AC",
         description: "Alignment Check",
         irqtype: "Fault",
         source: "Unaligned memory access.",
     },
     InterruptDescription {
         vector: 18,
         mnemonic: "#MC",
         description: "Machine Check",
         irqtype: "Abort",
         source: "Internal machine error.",
     },
     InterruptDescription {
         vector: 19,
         mnemonic: "#XM",
         description: "SIMD Floating-Point",
         irqtype: "Fault",
         source: "SSE SIMD instructions.",
     },
     InterruptDescription {
         vector: 20,
         mnemonic: "#VE",
         description: "Virtualization",
         irqtype: "Fault",
         source: "EPT violation.",
     }];

/// Enable Interrupts.
pub unsafe fn enable() {
    asm!("sti");
}

/// Disable Interrupts.
pub unsafe fn disable() {
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
