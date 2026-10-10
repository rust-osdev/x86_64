//! Runtime virtual-address validity policy and operations.

use core::convert::TryFrom;
use core::sync::atomic::{AtomicU8, Ordering};

#[cfg(feature = "virt_addr_57")]
use super::VirtAddr57;
use super::{
    VirtAddr48, VirtAddrGeneric, VirtAddrNotValid, VirtAddrValidity, align_down, align_up,
    canonicalize_with_bits, new_truncate_with_bits, try_new_with_bits,
};

/// The runtime virtual-address validity policy.
///
/// This policy checks the currently active address-space mode using a global cache of
/// `CR4.LA57`. The first operation that needs the active mode initializes the cache. This policy
/// is available only on `x86_64` with the `virt_addr_rt` feature, which enables `instructions`.
/// Checked construction, canonicalization, and address-producing arithmetic must execute in Ring 0.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RuntimeValidity;

/// A virtual address checked against the current address-space mode when created.
///
/// This alias is available only on `x86_64` with the `virt_addr_rt` feature.
pub type VirtAddrRT = VirtAddrGeneric<RuntimeValidity>;

impl crate::sealed::Sealed for RuntimeValidity {}

impl VirtAddrValidity for RuntimeValidity {
    fn bits() -> usize {
        cached_virtual_address_bits()
    }
}

impl From<VirtAddr48> for VirtAddrRT {
    #[inline]
    fn from(address: VirtAddr48) -> Self {
        unsafe { Self::new_unsafe(address.as_u64()) }
    }
}

#[cfg(feature = "virt_addr_57")]
impl From<VirtAddrRT> for VirtAddr57 {
    #[inline]
    fn from(address: VirtAddrRT) -> Self {
        unsafe { Self::new_unsafe(address.as_u64()) }
    }
}

impl TryFrom<VirtAddrRT> for VirtAddr48 {
    type Error = VirtAddrNotValid;

    #[inline]
    fn try_from(address: VirtAddrRT) -> Result<Self, Self::Error> {
        Self::try_new(address.as_u64())
    }
}

/// The cached virtual-address width for the active address-space mode.
///
/// Zero indicates that the cache has not been initialized yet.
static CURRENT_VIRTUAL_ADDRESS_BITS: AtomicU8 = AtomicU8::new(0);

/// Reads the virtual-address width for the currently active address-space mode.
///
/// This function must execute in Ring 0.
#[inline]
fn read_current_virtual_address_bits() -> u8 {
    use crate::registers::control::{Cr4, Cr4Flags};

    if Cr4::read().contains(Cr4Flags::L5_PAGING) {
        57
    } else {
        48
    }
}

/// Returns the cached virtual-address width for the active address-space mode.
///
/// This function must execute in Ring 0 if the cache has not been initialized yet.
#[inline]
fn cached_virtual_address_bits() -> usize {
    let cached = CURRENT_VIRTUAL_ADDRESS_BITS.load(Ordering::Relaxed);
    if cached != 0 {
        return usize::from(cached);
    }

    let current = read_current_virtual_address_bits();
    usize::from(
        match CURRENT_VIRTUAL_ADDRESS_BITS.compare_exchange(
            0,
            current,
            Ordering::Relaxed,
            Ordering::Relaxed,
        ) {
            Ok(_) => current,
            Err(initialized) => initialized,
        },
    )
}

impl VirtAddrGeneric<RuntimeValidity> {
    /// Creates a new virtual address valid in the current address-space mode.
    ///
    /// # Panics
    ///
    /// This function panics if the address is not canonical under the currently active mode.
    #[inline]
    pub fn new(addr: u64) -> Self {
        match Self::try_new(addr) {
            Ok(address) => address,
            Err(_) => panic!("virtual address must be canonical in the current address-space mode"),
        }
    }

    /// Tries to create a virtual address valid in the current address-space mode.
    ///
    /// This function checks the address using the cached active canonical width. The first runtime
    /// address operation initializes the cache from `CR4.LA57`.
    #[inline]
    pub fn try_new(addr: u64) -> Result<Self, VirtAddrNotValid> {
        // SAFETY: `cached_virtual_address_bits()` is valid, at least when the cache is initialized,
        // so this is safe.
        unsafe { try_new_with_bits(addr, cached_virtual_address_bits()) }
    }

    /// Creates a virtual address by canonicalizing it for the current address-space mode.
    ///
    /// This function uses the cached active canonical width to sign-extend the address. The first
    /// runtime address operation initializes the cache from `CR4.LA57`.
    #[inline]
    pub fn new_truncate(addr: u64) -> Self {
        // SAFETY: `cached_virtual_address_bits()` is valid, at least when the cache is initialized,
        // so this is safe.
        unsafe { new_truncate_with_bits(addr, cached_virtual_address_bits()) }
    }

    /// Creates a virtual address from the given pointer.
    ///
    /// The pointer address must be canonical in the current address-space mode.
    #[cfg(target_pointer_width = "64")]
    #[inline]
    pub fn from_ptr<T: ?Sized>(ptr: *const T) -> Self {
        Self::new(ptr as *const () as u64)
    }

    /// Aligns the virtual address upwards to the given alignment.
    ///
    /// The result is canonicalized using the current address-space mode.
    #[inline]
    pub fn align_up<U>(self, align: U) -> Self
    where
        U: Into<u64>,
    {
        Self::new_truncate(align_up(self.0, align.into()))
    }

    /// Aligns the virtual address downwards to the given alignment.
    ///
    /// The result is canonicalized using the current address-space mode.
    #[inline]
    pub fn align_down<U>(self, align: U) -> Self
    where
        U: Into<u64>,
    {
        Self::new_truncate(align_down(self.0, align.into()))
    }

    /// Refetches the virtual-address width from the active address-space mode and updates the
    /// cached value.
    ///
    /// The first runtime-valid address operation initializes the cache automatically. Call this
    /// method after changing `CR4.LA57` and before resuming operations that create or validate
    /// runtime-valid addresses. The caller is responsible for synchronizing the mode change with
    /// other processors and threads.
    ///
    /// This method reads `CR4.LA57`, so it must execute in Ring 0.
    #[inline]
    pub fn refetch_virtual_address_bits() {
        CURRENT_VIRTUAL_ADDRESS_BITS.store(read_current_virtual_address_bits(), Ordering::Relaxed);
    }
}

impl<V: VirtAddrValidity> VirtAddrGeneric<V> {
    /// Checks whether the address is canonical in the currently active address-space mode.
    ///
    /// This method checks the address against the cached active canonical width even though it was
    /// valid for its policy when created.
    #[inline]
    pub fn is_valid_currently(self) -> bool {
        canonicalize_with_bits(self.0, cached_virtual_address_bits()) == self.0
    }
}

#[cfg(feature = "virt_addr_57")]
impl TryFrom<VirtAddr57> for VirtAddrRT {
    type Error = VirtAddrNotValid;

    #[inline]
    fn try_from(address: VirtAddr57) -> Result<Self, Self::Error> {
        Self::try_new(address.as_u64())
    }
}

#[cfg(test)]
mod tests {
    use core::ops::{Add, AddAssign, Sub, SubAssign};

    #[cfg(feature = "step_trait")]
    use core::iter::Step;

    use super::*;

    #[test]
    fn runtime_l5_index_constructors_preserve_indices_or_reject() {
        use crate::structures::paging::{Page, PageTableIndex, Size1GiB, Size2MiB, Size4KiB};

        const TEST_WIDTH: &str = "X86_64_TEST_L5_ADDRESS_BITS";
        let Ok(bits) = std::env::var(TEST_WIDTH) else {
            // This test deliberately launches two child test processes. That is unusual, but
            // necessary: each process must run only this test so its global
            // `CURRENT_VIRTUAL_ADDRESS_BITS` cache cannot be shared with, or affected by, any
            // other test that uses the cache. The environment variable seeds the cache without
            // requiring a privileged CR4 read. If a better way to isolate this process-global
            // state becomes available, this subprocess-based test should be replaced.
            for bits in [48, 57] {
                let output = std::process::Command::new(std::env::current_exe().unwrap())
                    .args([
                        "--exact",
                        "addr::rt::tests::runtime_l5_index_constructors_preserve_indices_or_reject",
                        "--nocapture",
                    ])
                    .env(TEST_WIDTH, bits.to_string())
                    .output()
                    .unwrap();
                assert!(
                    output.status.success()
                        && std::string::String::from_utf8_lossy(&output.stdout)
                            .contains("1 passed; 0 failed"),
                    "runtime width {bits}:\n{}\n{}",
                    std::string::String::from_utf8_lossy(&output.stdout),
                    std::string::String::from_utf8_lossy(&output.stderr),
                );
            }
            return;
        };
        let bits = bits.parse::<u8>().unwrap();
        assert!(matches!(bits, 48 | 57));
        CURRENT_VIRTUAL_ADDRESS_BITS.store(bits, Ordering::Relaxed);

        for p5 in [0, 1, 255, 256, 510, 511] {
            for p4 in [0, 255, 256, 511] {
                let p5_index = PageTableIndex::new(p5);
                let p4_index = PageTableIndex::new(p4);
                let results = [
                    std::panic::catch_unwind(|| {
                        Page::<Size1GiB, RuntimeValidity>::from_page_table_indices_1gib_l5(
                            p5_index,
                            p4_index,
                            PageTableIndex::new(3),
                        )
                        .start_address()
                        .as_u64()
                    }),
                    std::panic::catch_unwind(|| {
                        Page::<Size2MiB, RuntimeValidity>::from_page_table_indices_2mib_l5(
                            p5_index,
                            p4_index,
                            PageTableIndex::new(3),
                            PageTableIndex::new(4),
                        )
                        .start_address()
                        .as_u64()
                    }),
                    std::panic::catch_unwind(|| {
                        Page::<Size4KiB, RuntimeValidity>::from_page_table_indices_l5(
                            p5_index,
                            p4_index,
                            PageTableIndex::new(3),
                            PageTableIndex::new(4),
                            PageTableIndex::new(5),
                        )
                        .start_address()
                        .as_u64()
                    }),
                ];
                for (result, low_bits) in results.into_iter().zip([
                    3 << 30,
                    (3 << 30) | (4 << 21),
                    (3 << 30) | (4 << 21) | (5 << 12),
                ]) {
                    let fits = bits == 57 || (p5 == 0 && p4 < 256) || (p5 == 511 && p4 >= 256);
                    if !fits {
                        assert!(result.is_err(), "P5={p5}, P4={p4} must be rejected");
                        continue;
                    }
                    let address = result.unwrap();
                    assert_eq!((address >> 48) & 511, u64::from(p5));
                    assert_eq!((address >> 39) & 511, u64::from(p4));
                    assert_eq!(address & ((1 << 39) - 1), low_bits);
                    assert_eq!(address >> 57, if p5 < 256 { 0 } else { 127 });
                }
            }
        }
    }

    #[test]
    fn runtime_virtaddr_arithmetic_traits_are_available() {
        fn assert_arithmetic<T>()
        where
            T: Add<u64, Output = T> + AddAssign<u64> + Sub<u64, Output = T> + SubAssign<u64>,
        {
        }

        assert_arithmetic::<VirtAddrRT>();

        #[cfg(feature = "step_trait")]
        {
            fn assert_step<T: Step>() {}
            assert_step::<VirtAddrRT>();
        }
    }

    #[test]
    fn current_validity_check_is_available_for_fixed_addresses() {
        let _: fn(VirtAddr48) -> bool = VirtAddr48::is_valid_currently;

        #[cfg(feature = "virt_addr_57")]
        let _: fn(VirtAddr57) -> bool = VirtAddr57::is_valid_currently;
    }

    #[test]
    fn runtime_fixed_conversions_preserve_or_check_values() {
        let address48 = VirtAddr48::new(0xffff_8000_0000_1234);
        let address_rt = VirtAddrRT::from(address48);

        assert_eq!(address_rt.as_u64(), address48.as_u64());
        assert_eq!(VirtAddr48::try_from(address_rt).unwrap(), address48);

        #[cfg(feature = "virt_addr_57")]
        {
            let address57 = VirtAddr57::from(address_rt);
            assert_eq!(address57.as_u64(), address48.as_u64());
        }
    }
}
