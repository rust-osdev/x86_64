//! Support for build-in RNGs

#[derive(Copy, Clone, Debug)]
/// Used to obtain random numbers using x86_64's RDRAND opcode
pub struct RdRand(());

#[cfg(target_arch = "x86_64")]
impl RdRand {
    /// Creates Some(RdRand) if RDRAND is supported, None otherwise
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
    #[deprecated(note = "This method does not check for errors and will be removed in the next breaking release. Use get_u64() instead.")]
    #[inline]
    pub fn get(&self) -> u64 {
        let res: u64;
        unsafe {
            asm!("rdrand %rax" : "={rax}"(res) ::: "volatile");
        }

        res
    }
    /// Uniformly sampled u64.
    /// May fail in rare circumstances or heavy load.
    #[inline]
    pub fn get_u64(&self) -> Option<u64> {
        let res: u64;
        let ok: u8;
        unsafe {
            asm!("rdrand %rax; setc $1;"
                : "={rax}"(res) "=r"(ok)
                :: "flags" : "volatile");
        }
        match ok {
            1 => Some(res),
            _ => None
        }
    }
    /// Uniformly sampled u32.
    /// May fail in rare circumstances or heavy load.
    #[inline]
    pub fn get_u32(&self) -> Option<u32> {
        let res: u32;
        let ok: u8;
        unsafe {
            asm!("rdrand %eax; setc $1;"
                : "={eax}"(res) "=r"(ok)
                :: "flags" : "volatile");
        }
        match ok {
            1 => Some(res),
            _ => None
        }
    }
    /// Uniformly sampled u16.
    /// May fail in rare circumstances or heavy load.
    #[inline]
    pub fn get_u16(&self) -> Option<u16> {
        let res: u16;
        let ok: u8;
        unsafe {
            asm!("rdrand %ax; setc $1;"
                : "={ax}"(res) "=r"(ok)
                :: "flags" : "volatile");
        }
        match ok {
            1 => Some(res),
            _ => None
        }
    }
}