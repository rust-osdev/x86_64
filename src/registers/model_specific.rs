//! Functions to read and write control registers.

/// A model specific register.
pub struct Msr(u32);

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

/// The Extended Feature Enable Register.
pub struct Efer;

impl Efer {
    /// The underlying model specific register.
    pub const MSR: Msr = Msr(0xC0000080);

    /// Read the current EFER flags.
    pub fn read() -> EferFlags {
        EferFlags::from_bits_truncate(unsafe { Self::MSR.read() })
    }

    /// Write the EFER flags.
    /// 
    /// Preserves the value of reserved fields. Unsafe because it's possible to break memory
    /// safety, e.g. by disabling long mode.
    pub unsafe fn write(flags: EferFlags) {
        let mut value = Self::MSR.read();
        value |= flags.bits();
        Self::MSR.write(value);
    }

    /// Update EFER flags.
    /// 
    /// Preserves the value of reserved fields. Unsafe because it's possible to break memory
    /// safety, e.g. by disabling long mode.
    pub unsafe fn update<F>(f: F) where F: FnOnce(&mut EferFlags) {
        let mut flags = Self::read();
        f(&mut flags);
        Self::write(flags);
    }
}

bitflags! {
    pub struct EferFlags: u64 {
        const SYSTEM_CALL_EXTENSIONS = 1 << 0;
        const LONG_MODE_ENABLE = 1 << 8;
        const LONG_MODE_ACTIVE = 1 << 10;
        const NO_EXECUTE_ENABLE = 1 << 11;
        const SECURE_VIRTUAL_MACHINE_ENABLE = 1 << 12;
        const LONG_MODE_SEGMENT_LIMIT_ENABLE = 1 << 13;
        const FAST_FXSAVE_FXRSTOR = 1 << 14;
        const TRANSLATION_CACHE_EXTENSION = 1 << 15;
    }
}
