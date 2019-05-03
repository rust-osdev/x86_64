//! Processor state stored in the RFLAGS register.

pub use x86_64_types::registers::RFlags;

#[cfg(target_arch = "x86_64")]
pub use self::x86_64::*;

#[cfg(target_arch = "x86_64")]
mod x86_64 {
    use super::*;

    /// Returns the current value of the RFLAGS register.
    ///
    /// Drops any unknown bits.
    pub fn read() -> RFlags {
        RFlags::from_bits_truncate(read_raw())
    }

    /// Returns the raw current value of the RFLAGS register.
    pub fn read_raw() -> u64 {
        let r: u64;
        unsafe { asm!("pushfq; popq $0" : "=r"(r) :: "memory") };
        r
    }

    /// Writes the RFLAGS register, preserves reserved bits.
    pub fn write(flags: RFlags) {
        let old_value = read_raw();
        let reserved = old_value & !(RFlags::all().bits());
        let new_value = reserved | flags.bits();

        write_raw(new_value);
    }

    /// Writes the RFLAGS register.
    ///
    /// Does not preserve any bits, including reserved bits.
    pub fn write_raw(val: u64) {
        unsafe { asm!("pushq $0; popfq" :: "r"(val) : "memory" "flags") };
    }
}
