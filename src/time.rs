
/// Read the time stamp counter.
pub unsafe fn rdtsc() -> u64 {
    let mut low: u32;
    let mut high: u32;

    asm!("rdtsc" : "={eax}" (low), "={edx}" (high));
    ((high as u64) << 32) | (low as u64)
}
