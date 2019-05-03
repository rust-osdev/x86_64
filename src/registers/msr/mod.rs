//! Functions to read and write model specific registers.

mod efer;

pub use efer::Efer;

/// A model specific register.
#[derive(Debug)]
pub struct Msr(u32);

impl Msr {
    /// Create an instance from a register.
    pub const fn new(reg: u32) -> Msr {
        Msr(reg)
    }

    /// Read 64 bits msr register.
    pub fn read(&self) -> u64 {
        let (high, low): (u32, u32);
        unsafe {
            asm!(
                "rdmsr"
                : "={eax}" (low), "={edx}" (high)
                : "{ecx}" (self.0)
                : "memory"
                : "volatile"
            );
        }
        ((high as u64) << 32) | (low as u64)
    }

    /// Write 64 bits to msr register.
    pub unsafe fn write(&mut self, value: u64) {
        let low = value as u32;
        let high = (value >> 32) as u32;
        asm!(
            "wrmsr"
            :
            : "{ecx}" (self.0), "{eax}" (low), "{edx}" (high)
            : "memory"
            : "volatile"
        );
    }
}
