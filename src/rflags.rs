//! Description of RFlag values that store the results of operations and the state of the processor.

/// RFLAGS description.
bitflags! {
    flags RFlags: u64 {
        /// ID Flag (ID)
        const RFLAGS_ID   = 1 << 21,
        /// Virtual Interrupt Pending (VIP)
        const RFLAGS_VIP  = 1 << 20,
        /// Virtual Interrupt Flag (VIF)
        const RFLAGS_VIF  = 1 << 19,
        /// Alignment Check (AC)
        const RFLAGS_AC   = 1 << 18,
        /// Virtual-8086 Mode (VM)
        const RFLAGS_VM   = 1 << 17,
        /// Resume Flag (RF)
        const RFLAGS_RF   = 1 << 16,
        /// Nested Task (NT)
        const RFLAGS_NT   = 1 << 14,
        /// I/O Privilege Level (IOPL) 0
        const RFLAGS_IOPL0 = 0 << 12,
        /// I/O Privilege Level (IOPL) 1
        const RFLAGS_IOPL1 = 1 << 12,
        /// I/O Privilege Level (IOPL) 2
        const RFLAGS_IOPL2 = 2 << 12,
        /// I/O Privilege Level (IOPL) 3
        const RFLAGS_IOPL3 = 3 << 12,
        /// Overflow Flag (OF)
        const RFLAGS_OF   = 1 << 11,
        /// Direction Flag (DF)
        const RFLAGS_DF   = 1 << 10,
        /// Interrupt Enable Flag (IF)
        const RFLAGS_IF   = 1 << 9,
        /// Trap Flag (TF)
        const RFLAGS_TF   = 1 << 8,
        /// Sign Flag (SF)
        const RFLAGS_SF   = 1 << 7,
        /// Zero Flag (ZF)
        const RFLAGS_ZF   = 1 << 6,
        /// Auxiliary Carry Flag (AF)
        const RFLAGS_AF   = 1 << 4,
        /// Parity Flag (PF)
        const RFLAGS_PF   = 1 << 2,
        /// Bit 1 is always 1.
        const RFLAGS_A1   = 1 << 1,
        /// Carry Flag (CF)
        const RFLAGS_CF   = 1 << 0,
    }
}

impl RFlags {
    /// Creates a new RFlags entry. Ensures bit 1 is set.
    pub fn new() -> RFlags {
        RFLAGS_A1
    }
}
