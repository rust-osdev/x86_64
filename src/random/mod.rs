//! Support for build-in RNGs

#[derive(Debug)]
/// Used to obtain random numbers using x86_64's RDRAND opcode
pub struct RdRand(());

impl RdRand {
    /// Creates Some(RdRand) if RDRAND is supported, None otherwise
    #[cfg(target_arch = "x86_64")]
    pub fn new() -> Option<Self> {
        let cpuid = raw_cpuid::CpuId::new();
        let has_rdrand = match cpuid.get_feature_info() {
            Some(finfo) => finfo.has_rdrand(),
            None => false,
        };

        match has_rdrand {
            true => Some(RdRand(())),
            false => None,
        }
    }

    /// Uniformly sampled u64
    #[inline]
    #[cfg(target_arch = "x86_64")]
    pub fn get(&self) -> u64 {
        let res: u64;
        unsafe {
            asm!("rdrand %rax" : "={rax}"(res) ::: "volatile");
        }

        res
    }
}
