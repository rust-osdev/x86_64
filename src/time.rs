/// Read the time stamp counter.
pub unsafe fn rdtsc() -> u64 {
    let mut value: u64 = 0;
    asm!("rdtsc" : "=A" (value));
    value
}

