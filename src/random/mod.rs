//! Support for build-in RNGs

#[derive(Copy, Clone, Debug)]
/// Used to obtain random numbers using x86_64's RDRAND opcode
pub struct RdRand(());

mod private {
    /// Trait for types that can be returned by the RDRAND instruction
    pub trait RdRandPrimitive: Sized {}
    impl RdRandPrimitive for u16 {}
    impl RdRandPrimitive for u32 {}
    #[cfg(target_arch = "x86_64")]
    impl RdRandPrimitive for u64 {}
}
use private::RdRandPrimitive;

impl RdRand {
    /// Creates Some(RdRand) if RDRAND is supported, None otherwise
    #[cfg(target_arch = "x86_64")] // <-- raw_cpuid is only a dep for x86_64 targets
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

    /// Uniformly sampled u64, u32, or u16
    #[inline]
    pub fn get<T: RdRandPrimitive>(&self) -> Option<T> {
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
    #[cfg(target_arch = "x86_64")]
    #[inline]
    pub fn get_u64(&self) -> Option<u64> {
        self.get()
    }
    /// Uniformly sampled u32
    #[inline]
    pub fn get_u32(&self) -> Option<u32> {
        self.get()
    }
    /// Uniformly sampled u16
    #[inline]
    pub fn get_u16(&self) -> Option<u16> {
        self.get()
    }
}
