//! Functions to read and write control registers.

pub use x86_64_types::registers::Efer as EferFlags;

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
