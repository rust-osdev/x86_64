//! Functions to read time stamp counters on x86.

/// Read the time stamp counter.
///
/// The RDTSC instruction is not a serializing instruction.
/// It does not necessarily wait until all previous instructions
/// have been executed before reading the counter. Similarly,
/// subsequent instructions may begin execution before the
/// read operation is performed. If software requires RDTSC to be
/// executed only after all previous instructions have completed locally,
/// it can either use RDTSCP or execute the sequence LFENCE;RDTSC.
///
/// # Safety
/// * Causes a GP fault if the TSD flag in register CR4 is set and the CPL
///   is greater than 0.
#[allow(unused_mut)]
pub unsafe fn rdtsc() -> u64 {
    let mut low: u32;
    let mut high: u32;

    asm!("rdtsc" : "={eax}" (low), "={edx}" (high));
    ((high as u64) << 32) | (low as u64)
}

/// Read the time stamp counter.
///
/// The RDTSCP instruction waits until all previous instructions
/// have been executed before reading the counter.
/// However, subsequent instructions may begin execution
/// before the read operation is performed.
///
/// Volatile is used here because the function may be used to act as
/// an instruction barrier.
///
/// # Safety
/// * Causes a GP fault if the TSD flag in register CR4 is set and the
///   CPL is greater than 0.
#[allow(unused_mut)]
pub unsafe fn rdtscp() -> u64 {
    let mut low: u32;
    let mut high: u32;

    asm!("rdtscp" : "={eax}" (low), "={edx}" (high) ::: "volatile");
    ((high as u64) << 32) | (low as u64)
}
