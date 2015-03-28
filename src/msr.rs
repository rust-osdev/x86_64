/// Write 64 bits to msr register.
pub unsafe fn wrmsr(msr: u32, value: u64) {
    asm!("wrmsr" : : "{ecx}" (msr), "A" (value));
}


/// Read 64 bits msr register.
pub unsafe fn rdmsr(msr: u32) -> u64 {
    
    let mut val: u64 = -1;
    asm!("rdmsr" : "=A"(val) : "{ecx}"(msr));
    
    return val;
}