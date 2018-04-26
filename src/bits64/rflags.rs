//! Processor state stored in the RFLAGS register.
//!
//! In 64-bit mode, EFLAGS is extended to 64 bits and called RFLAGS. 
//! The upper 32 bits of RFLAGS register is reserved. 
//! The lower 32 bits of RFLAGS is the same as EFLAGS.

use ::Ring;

/// The RFLAGS register.
/// This is duplicated code from bits32 eflags.rs.
bitflags! {
    pub struct RFlags: u64 {
        /// ID Flag (ID)
        const FLAGS_ID = 1 << 21;
        /// Virtual Interrupt Pending (VIP)
        const FLAGS_VIP = 1 << 20;
        /// Virtual Interrupt Flag (VIF)
        const FLAGS_VIF = 1 << 19;
        /// Alignment Check (AC)
        const FLAGS_AC = 1 << 18;
        /// Virtual-8086 Mode (VM)
        const FLAGS_VM = 1 << 17;
        /// Resume Flag (RF)
        const FLAGS_RF = 1 << 16;
        /// Nested Task (NT)
        const FLAGS_NT = 1 << 14;
        /// I/O Privilege Level (IOPL) 0
        const FLAGS_IOPL0 = 0b00 << 12;
        /// I/O Privilege Level (IOPL) 1
        const FLAGS_IOPL1 = 0b01 << 12;
        /// I/O Privilege Level (IOPL) 2
        const FLAGS_IOPL2 = 0b10 << 12;
        /// I/O Privilege Level (IOPL) 3
        const FLAGS_IOPL3 = 0b11 << 12;
        /// Overflow Flag (OF)
        const FLAGS_OF = 1 << 11;
        /// Direction Flag (DF)
        const FLAGS_DF = 1 << 10;
        /// Interrupt Enable Flag (IF)
        const FLAGS_IF = 1 << 9;
        /// Trap Flag (TF)
        const FLAGS_TF = 1 << 8;
        /// Sign Flag (SF)
        const FLAGS_SF = 1 << 7;
        /// Zero Flag (ZF)
        const FLAGS_ZF = 1 << 6;
        /// Auxiliary Carry Flag (AF)
        const FLAGS_AF = 1 << 4;
        /// Parity Flag (PF)
        const FLAGS_PF = 1 << 2;
        /// Bit 1 is always 1.
        const FLAGS_A1 = 1 << 1;
        /// Carry Flag (CF)
        const FLAGS_CF = 1 << 0;
    }
}

impl RFlags {
    /// Creates a new Flags entry. Ensures bit 1 is set.
    pub const fn new() -> RFlags {
        RFlags::FLAGS_A1
    }

    /// Creates a new Flags with the given I/O privilege level.
    pub const fn from_priv(iopl: Ring) -> RFlags {
        RFlags { bits: (iopl as u64) << 12 }
    }
}

#[cfg(target_arch="x86-64")]
pub unsafe fn read() -> RFlags {
    let r: u64;
    asm!("pushfq; popq $0" : "=r"(r) :: "memory");
    RFlags::from_bits_truncate(r)
}

#[cfg(target_arch="x86-64")]
pub unsafe fn set(val: RFlags) {
    asm!("pushq $0; popfq" :: "r"(val.bits()) : "memory" "flags");
}
