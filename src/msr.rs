
/// Extended Feature Enables
pub const IA32_EFER: u32 = 0xC0000080;

/// System Call Target Address (R/W)
pub const IA32_STAR: u32 = 0xC0000081;

/// IA-32e Mode System Call Target Address
pub const IA32_LSTAR: u32 = 0xC0000082;

/// System Call Flag Mask (R/W)
pub const IA32_FMASK: u32 = 0xC0000084;

/// Write 64 bits to msr register.
pub unsafe fn wrmsr(msr: u32, value: u64) {
    asm!("wrmsr" :: "{ecx}" (msr), "A" (value));
}

/// Read 64 bits msr register.
pub unsafe fn rdmsr(msr: u32) -> u64 {
    let mut val: u64;
    asm!("rdmsr" : "=A"(val) : "{ecx}" (msr));

    return val;
}