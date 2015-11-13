use core::fmt;

/// x86 Exception description (see also Intel Vol. 3a Chapter 6).
#[derive(Debug)]
pub struct InterruptDescription {
    pub vector: u8,
    pub mnemonic: &'static str,
    pub description: &'static str,
    pub irqtype: &'static str,
    pub source: &'static str
}

impl fmt::Display for InterruptDescription {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} ({}, vec={}) {}", self.mnemonic, self.irqtype, self.vector, self.description)
    }
}


/// x86 External Interrupts (1-16).
pub static EXCEPTIONS: [InterruptDescription; 15] = [
    InterruptDescription{ vector: 0,  mnemonic: "#DE", description: "Divide Error", irqtype: "Fault", source: "DIV and IDIV instructions." },
    InterruptDescription{ vector: 1,  mnemonic: "#DB", description: "RESERVED", irqtype: "Fault/ Trap", source: "For Intel use only." },
    InterruptDescription{ vector: 2,  mnemonic: "NMI", description: "Interrupt", irqtype: "Interrupt", source: "Nonmaskable external interrupt." },
    InterruptDescription{ vector: 3,  mnemonic: "#BP", description: "Breakpoint", irqtype: "Trap", source: "INT 3 instruction." },
    InterruptDescription{ vector: 4,  mnemonic: "#OF", description: "Overflow", irqtype: "Trap", source: "INTO instruction." },
    InterruptDescription{ vector: 5,  mnemonic: "#BR", description: "BOUND Range Exceeded", irqtype: "Fault", source: "BOUND instruction." },
    InterruptDescription{ vector: 6,  mnemonic: "#UD", description: "Invalid Opcode (Undefined Opcode)", irqtype: "Fault", source: "UD2 instruction or reserved opcode." },
    InterruptDescription{ vector: 7,  mnemonic: "#NM", description: "Device Not Available (No Math Coprocessor)", irqtype: "Fault", source: "Floating-point or WAIT/FWAIT instruction." },
    InterruptDescription{ vector: 8,  mnemonic: "#DF", description: "Double Fault", irqtype: "Abort", source: "Any instruction that can generate an exception, an NMI, or an INTR." },
    InterruptDescription{ vector: 9,  mnemonic: ""   , description: "Coprocessor Segment Overrun", irqtype: "Fault", source: "Floating-point instruction." },
    InterruptDescription{ vector: 10, mnemonic: "#TS", description: "Invalid TSS", irqtype: "Fault", source: "Task switch or TSS access." },
    InterruptDescription{ vector: 11, mnemonic: "#NP", description: "Segment Not Present", irqtype: "Fault", source: "Loading segment registers or accessing system segments." },
    InterruptDescription{ vector: 12, mnemonic: "#SS", description: "Stack-Segment Fault", irqtype: "Fault", source: "Stack operations and SS register loads." },
    InterruptDescription{ vector: 13, mnemonic: "#GP", description: "General Protection", irqtype: "Fault", source: "Any memory reference and other protection checks." },
    InterruptDescription{ vector: 14, mnemonic: "#PF", description: "Page Fault", irqtype: "Fault", source: "Any memory reference." }
];


/// Enable Interrupts.
pub unsafe fn enable()  {
    asm!("sti");
}

/// Disable Interrupts.
pub unsafe fn disable()  {
    asm!("cli");
}

/// Generate a software interrupt.
/// This is a macro argument needs to be an immediate.
#[macro_export]
macro_rules! int {
    ( $x:expr ) => {
        {
            asm!("int $0" :: "N" ($x));
        }
    };
}

/// A struct describing an interrupt gate.  See the Intel manual mentioned
/// above for details, specifically, the section "6.14.1 64-Bit Mode IDT"
/// and "Table 3-2. System-Segment and Gate-Descriptor Types".
#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
pub struct IdtEntry {
    /// Lower 16 bits of ISR.
    pub base_lo: u16,
    /// Segment selector.
    pub sel: u16,
    /// This must always be zero.
    pub res0: u8,
    /// Flags.
    pub flags: u8,
    /// The upper 48 bits of ISR (the last 16 bits must be zero).
    pub base_hi: u64,
    /// Must be zero.
    pub res1: u16
}

impl IdtEntry {

    /// Create a "missing" IdtEntry.  This is a `const` function, so we can
    /// call it at compile time to initialize static variables.
    ///
    /// If the CPU tries to invoke a missing interrupt, it will instead
    /// send a General Protection fault (13), with the interrupt number and
    /// some other data stored in the error code.
    pub const fn missing() -> IdtEntry {
        IdtEntry {
            base_lo: 0,
            sel: 0,
            res0: 0,
            flags: 0,
            base_hi: 0,
            res1: 0,
        }
    }

    /// Create a new IdtEntry pointing at `handler`, which must be a
    /// function with interrupt calling conventions.  (This must be
    /// currently defined in assembly language.)  The `gdt_code_selector`
    /// value must be the offset of code segment entry in the GDT.
    ///
    /// Create an interrupt gate with the "Present" flag set, which is the
    /// most common case.  If you need something else, you can construct it
    /// manually.
    pub fn new(gdt_code_selector: u16, handler: *const u8) -> IdtEntry {
        IdtEntry {
            base_lo: ((handler as u64) & 0xFFFF) as u16,
            sel: gdt_code_selector,
            res0: 0,
            // Bit 7: "Present" flag set.
            // Bits 0-4: This is an interrupt gate.
            flags: 0b1000_1110,
            base_hi: (handler as u64) >> 16,
            res1: 0,
        }
    }
}

bitflags!{
    // Taken from Intel Manual Section 4.7 Page-Fault Exceptions.
    flags PageFaultError: u32 {
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

impl fmt::Debug for PageFaultError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let p = match self.contains(PFAULT_ERROR_P) {
            false => "The fault was caused by a non-present page.",
            true => "The fault was caused by a page-level protection violation."
        };
        let wr = match self.contains(PFAULT_ERROR_WR) {
            false => "The access causing the fault was a read.",
            true => "The access causing the fault was a write."
        };
        let us = match self.contains(PFAULT_ERROR_US) {
            false => "The access causing the fault originated when the processor was executing in supervisor mode.",
            true => "The access causing the fault originated when the processor was executing in user mode."
        };
        let rsvd = match self.contains(PFAULT_ERROR_RSVD) {
            false => "The fault was not caused by reserved bit violation.",
            true => "The fault was caused by reserved bits set to 1 in a page directory."
        };
        let id = match self.contains(PFAULT_ERROR_ID) {
            false => "The fault was not caused by an instruction fetch.",
            true => "The fault was caused by an instruction fetch."
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