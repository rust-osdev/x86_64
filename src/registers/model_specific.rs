//! Functions to read and write control registers.

use bitflags::bitflags;

/// A model specific register.
#[derive(Debug)]
pub struct Msr(u32);

impl Msr {
    /// Create an instance from a register.
    pub const fn new(reg: u32) -> Msr {
        Msr(reg)
    }
}

/// The Extended Feature Enable Register.
#[derive(Debug)]
pub struct Efer;

impl Efer {
    /// The underlying model specific register.
    pub const MSR: Msr = Msr(0xC0000080);
}

bitflags! {
    /// Flags of the Extended Feature Enable Register.
    pub struct EferFlags: u64 {
        /// Enables the `syscall` and `sysret` instructions.
        const SYSTEM_CALL_EXTENSIONS = 1 << 0;
        /// Activates long mode, requires activating paging.
        const LONG_MODE_ENABLE = 1 << 8;
        /// Indicates that long mode is active.
        const LONG_MODE_ACTIVE = 1 << 10;
        /// Enables the no-execute page-protection feature.
        const NO_EXECUTE_ENABLE = 1 << 11;
        /// Enables SVM extensions.
        const SECURE_VIRTUAL_MACHINE_ENABLE = 1 << 12;
        /// Enable certain limit checks in 64-bit mode.
        const LONG_MODE_SEGMENT_LIMIT_ENABLE = 1 << 13;
        /// Enable the `fxsave` and `fxrstor` instructions to execute faster in 64-bit mode.
        const FAST_FXSAVE_FXRSTOR = 1 << 14;
        /// Changes how the `invlpg` instruction operates on TLB entries of upper-level entries.
        const TRANSLATION_CACHE_EXTENSION = 1 << 15;
    }
}

#[cfg(target_arch = "x86_64")]
mod x86_64 {
    use super::*;

    impl Msr {
        /// Read 64 bits msr register.
        pub unsafe fn read(&self) -> u64 {
            let (high, low): (u32, u32);
            asm!("rdmsr" : "={eax}" (low), "={edx}" (high) : "{ecx}" (self.0) : "memory" : "volatile");
            ((high as u64) << 32) | (low as u64)
        }

        /// Write 64 bits to msr register.
        pub unsafe fn write(&mut self, value: u64) {
            let low = value as u32;
            let high = (value >> 32) as u32;
            asm!("wrmsr" :: "{ecx}" (self.0), "{eax}" (low), "{edx}" (high) : "memory" : "volatile" );
        }
    }

    impl Efer {
        /// Read the current EFER flags.
        pub fn read() -> EferFlags {
            EferFlags::from_bits_truncate(Self::read_raw())
        }

        /// Read the current raw EFER flags.
        pub fn read_raw() -> u64 {
            unsafe { Self::MSR.read() }
        }

        /// Write the EFER flags, preserving reserved values.
        ///
        /// Preserves the value of reserved fields. Unsafe because it's possible to break memory
        /// safety, e.g. by disabling long mode.
        pub unsafe fn write(flags: EferFlags) {
            let old_value = Self::read_raw();
            let reserved = old_value & !(EferFlags::all().bits());
            let new_value = reserved | flags.bits();

            Self::write_raw(new_value);
        }

        /// Write the EFER flags.
        ///
        /// Does not preserve any bits, including reserved fields. Unsafe because it's possible to
        /// break memory safety, e.g. by disabling long mode.
        pub unsafe fn write_raw(flags: u64) {
            Self::MSR.write(flags);
        }

        /// Update EFER flags.
        ///
        /// Preserves the value of reserved fields. Unsafe because it's possible to break memory
        /// safety, e.g. by disabling long mode.
        pub unsafe fn update<F>(f: F)
        where
            F: FnOnce(&mut EferFlags),
        {
            let mut flags = Self::read();
            f(&mut flags);
            Self::write(flags);
        }
    }
}
