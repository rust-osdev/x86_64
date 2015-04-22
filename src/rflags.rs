/// RFLAGS description.
bitflags! {
    flags RFlags: u64 {
        /// ID Flag (ID)
        const RFlags_ID   = 1 << 21,
        /// Virtual Interrupt Pending (VIP)
        const RFlags_VIP  = 1 << 20,
        /// Virtual Interrupt Flag (VIF)
        const RFlags_VIF  = 1 << 19,
        /// Alignment Check (AC)
        const RFlags_AC   = 1 << 18,
        /// Virtual-8086 Mode (VM)
        const RFlags_VM   = 1 << 17,
        /// Resume Flag (RF)
        const RFlags_RF   = 1 << 16,
        /// Nested Task (NT)
        const RFlags_NT   = 1 << 14,
        /// I/O Privilege Level (IOPL) 0
        const RFlags_IOPL0 = 0 << 12,
        /// I/O Privilege Level (IOPL) 1
        const RFlags_IOPL1 = 1 << 12,
        /// I/O Privilege Level (IOPL) 2
        const RFlags_IOPL2 = 2 << 12,
        /// I/O Privilege Level (IOPL) 3
        const RFlags_IOPL3 = 3 << 12,
        /// Overflow Flag (OF)
        const RFlags_OF   = 1 << 11,
        /// Direction Flag (DF)
        const RFlags_DF   = 1 << 10,
        /// Interrupt Enable Flag (IF)
        const RFlags_IF   = 1 << 9,
        /// Trap Flag (TF)
        const RFlags_TF   = 1 << 8,
        /// Sign Flag (SF)
        const RFlags_SF   = 1 << 7,
        /// Zero Flag (ZF)
        const RFlags_ZF   = 1 << 6,
        /// Auxiliary Carry Flag (AF)
        const RFlags_AF   = 1 << 4,
        /// Parity Flag (PF)
        const RFlags_PF   = 1 << 2,
        /// Bit 1 is always 1.
        const RFlags_A1   = 1 << 1,
        /// Carry Flag (CF)
        const RFlags_CF   = 1 << 0,
    }
}

impl RFlags {
    /// Creates a new RFlags entry. Ensures bit 1 is set.
    pub fn new() -> RFlags {
        RFlags_A1
    }
}