/// Read the time stamp counter.
pub unsafe fn rdtsc() -> u64 {
    let mut low: u64 = 0;
    let mut high: u64 = 0;

    asm!("rdtsc" : "={eax}" (low), "={edx}" (high));
    (high << 32) | low
}

