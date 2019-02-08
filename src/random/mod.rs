//! Support for build-in RNGs

#[derive(Copy, Clone, Debug)]
/// Used to obtain random numbers using x86_64's RDRAND opcode
pub struct RdRand(());

mod private {
    /// Trait for types that can be returned by the RDRAND instruction
    pub trait RdRandPrimitive {}
    impl RdRandPrimitive for u16 {}
    impl RdRandPrimitive for u32 {}
    impl RdRandPrimitive for u64 {}
}
use private::RdRandPrimitive;

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

    /// Uniformly sampled u64, u32, or u16
    #[inline]
    pub fn rand<T: RdRandPrimitive>(&self) -> Option<T> {
        let res: T;
        let ok: u8;
        unsafe {
            asm!("rdrand $0; setc $1;"
                : "=r"(res) "=r"(ok)
                :: "flags" : "volatile");
        }
        match ok {
            1 => Some(res),
            _ => None
        }
    }

    /// Uniformly sampled u64
    #[inline]
    pub fn get_u64(&self) -> Option<u64> {
        self.rand()
    }
    /// Uniformly sampled u32
    #[inline]
    pub fn get_u32(&self) -> Option<u32> {
        self.rand()
    }
    /// Uniformly sampled u16
    #[inline]
    pub fn get_u16(&self) -> Option<u16> {
        self.rand()
    }
}
