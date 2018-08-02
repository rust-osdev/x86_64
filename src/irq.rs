//! Shared interrupt description and set-up code.
//! See the `bits*::irq` modules for arch-specific portions.

use core::fmt;

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
        write!(
            f,
            "{} ({}, vec={}) {}",
            self.mnemonic, self.irqtype, self.vector, self.description
        )
    }
}

/// x86 External Interrupts (1-16).
pub static EXCEPTIONS: [InterruptDescription; 21] = [
    InterruptDescription {
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
        irqtype: "Fault / Trap",
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
        description: "Invalid Opcode (Undefined \
                      Opcode)",
        irqtype: "Fault",
        source: "UD2 instruction or reserved \
                 opcode.",
    },
    InterruptDescription {
        vector: 7,
        mnemonic: "#NM",
        description: "Device Not Available (No \
                      Math Coprocessor)",
        irqtype: "Fault",
        source: "Floating-point or WAIT/FWAIT \
                 instruction.",
    },
    InterruptDescription {
        vector: 8,
        mnemonic: "#DF",
        description: "Double Fault",
        irqtype: "Abort",
        source: "Any instruction that can \
                 generate an exception, an NMI, \
                 or an INTR.",
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
        source: "Loading segment registers or \
                 accessing system segments.",
    },
    InterruptDescription {
        vector: 12,
        mnemonic: "#SS",
        description: "Stack-Segment Fault",
        irqtype: "Fault",
        source: "Stack operations and SS register \
                 loads.",
    },
    InterruptDescription {
        vector: 13,
        mnemonic: "#GP",
        description: "General Protection",
        irqtype: "Fault",
        source: "Any memory reference and other \
                 protection checks.",
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
    },
];

bitflags!{
    // Taken from Intel Manual Section 4.7 Page-Fault Exceptions.
    pub struct PageFaultError: u32 {
        /// 0: The fault was caused by a non-present page.
        /// 1: The fault was caused by a page-level protection violation
        const P = bit!(0);

        /// 0: The access causing the fault was a read.
        /// 1: The access causing the fault was a write.
        const WR = bit!(1);

        /// 0: The access causing the fault originated when the processor
        /// was executing in supervisor mode.
        /// 1: The access causing the fault originated when the processor
        /// was executing in user mode.
        const US = bit!(2);

        /// 0: The fault was not caused by reserved bit violation.
        /// 1: The fault was caused by reserved bits set to 1 in a page directory.
        const RSVD = bit!(3);

        /// 0: The fault was not caused by an instruction fetch.
        /// 1: The fault was caused by an instruction fetch.
        const ID = bit!(4);

        /// 0: The fault was not by protection keys.
        /// 1: There was a protection key violation.
        const PK = bit!(5);
    }
}

impl fmt::Display for PageFaultError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let p = match self.contains(PageFaultError::P) {
            false => "The fault was caused by a non-present page.",
            true => "The fault was caused by a page-level protection violation.",
        };
        let wr = match self.contains(PageFaultError::WR) {
            false => "The access causing the fault was a read.",
            true => "The access causing the fault was a write.",
        };
        let us = match self.contains(PageFaultError::US) {
            false => {
                "The access causing the fault originated when the processor was executing in \
                 supervisor mode."
            }
            true => {
                "The access causing the fault originated when the processor was executing in user \
                 mode."
            }
        };
        let rsvd = match self.contains(PageFaultError::RSVD) {
            false => "The fault was not caused by reserved bit violation.",
            true => "The fault was caused by reserved bits set to 1 in a page directory.",
        };
        let id = match self.contains(PageFaultError::ID) {
            false => "The fault was not caused by an instruction fetch.",
            true => "The fault was caused by an instruction fetch.",
        };

        write!(f, "{}\n{}\n{}\n{}\n{}", p, wr, us, rsvd, id)
    }
}

#[test]
fn bit_macro() {
    assert!(PageFaultError::PK.bits() == 0b100000);
    assert!(PageFaultError::ID.bits() == 0b10000);
    assert!(PageFaultError::RSVD.bits() == 0b1000);
    assert!(PageFaultError::US.bits() == 0b100);
    assert!(PageFaultError::WR.bits() == 0b10);
    assert!(PageFaultError::P.bits() == 0b1);
}

/// Enable Interrupts.
pub unsafe fn enable() {
    asm!("sti");
}

/// Disable Interrupts.
pub unsafe fn disable() {
    asm!("cli");
}

/// Generate a software interrupt.
/// This is a macro argument needs to be an immediate.
#[macro_export]
macro_rules! int {
    ($x:expr) => {{
        asm!("int $0" :: "N" ($x));
    }};
}
