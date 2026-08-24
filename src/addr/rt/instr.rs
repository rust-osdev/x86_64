//! Runtime virtual-address width operations for the `RuntimeValidity` policy.

#[cfg(feature = "virt_addr_57")]
use core::convert::TryFrom;
use core::sync::atomic::{AtomicU8, Ordering};

#[cfg(feature = "virt_addr_57")]
use crate::addr::VirtAddr57;
use crate::addr::{
    align_down, align_up, canonicalize_with_bits, new_truncate_with_bits, try_new_with_bits,
    ArithmeticValidity, VirtAddrGeneric, VirtAddrNotValid, VirtAddrValidity,
};

use super::RuntimeValidity;
#[cfg(any(test, feature = "virt_addr_57"))]
use super::VirtAddrRT;

impl ArithmeticValidity for RuntimeValidity {}

/// The cached virtual-address width for the active address-space mode.
///
/// Zero indicates that the cache has not been initialized yet.
static CURRENT_VIRTUAL_ADDRESS_BITS: AtomicU8 = AtomicU8::new(0);

/// Returns a lazily initialized virtual-address width from the given cache.
#[inline]
fn cached_virtual_address_bits_with(
    cache: &AtomicU8,
    read_current_bits: impl FnOnce() -> u8,
) -> usize {
    let cached = cache.load(Ordering::Relaxed);
    if cached != 0 {
        return usize::from(cached);
    }

    let current = read_current_bits();
    debug_assert!(current == 48 || current == 57);
    usize::from(
        match cache.compare_exchange(0, current, Ordering::Relaxed, Ordering::Relaxed) {
            Ok(_) => current,
            Err(initialized) => initialized,
        },
    )
}

/// Replaces the virtual-address width in the given cache.
#[inline]
fn refetch_virtual_address_bits_with(cache: &AtomicU8, read_current_bits: impl FnOnce() -> u8) {
    let current = read_current_bits();
    debug_assert!(current == 48 || current == 57);
    cache.store(current, Ordering::Relaxed);
}

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

/// Refetches and caches the virtual-address width for the active address-space mode.
///
/// This function must execute in Ring 0.
#[inline]
fn refetch_virtual_address_bits() {
    refetch_virtual_address_bits_with(
        &CURRENT_VIRTUAL_ADDRESS_BITS,
        read_current_virtual_address_bits,
    );
}

/// Returns the cached virtual-address width for the active address-space mode.
///
/// This function must execute in Ring 0 if the cache has not been initialized yet.
#[inline]
pub(super) fn cached_virtual_address_bits() -> usize {
    cached_virtual_address_bits_with(
        &CURRENT_VIRTUAL_ADDRESS_BITS,
        read_current_virtual_address_bits,
    )
}

impl VirtAddrGeneric<RuntimeValidity> {
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
        refetch_virtual_address_bits();
    }

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
        new_truncate_with_bits(addr, cached_virtual_address_bits())
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
        self.align_down_u64(align.into())
    }

    /// Aligns the virtual address downwards to the given alignment.
    ///
    /// This variant accepts the alignment as a `u64` for internal users.
    #[inline]
    pub(crate) fn align_down_u64(self, align: u64) -> Self {
        Self::new_truncate(align_down(self.0, align))
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
    use core::sync::atomic::{AtomicU8, AtomicUsize, Ordering};

    #[cfg(feature = "step_trait")]
    use core::iter::Step;

    use crate::addr::VirtAddr48;

    use super::*;

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
    fn runtime_virtual_address_bits_are_cached_and_updateable() {
        let cache = AtomicU8::new(0);
        let reads = AtomicUsize::new(0);

        assert_eq!(
            cached_virtual_address_bits_with(&cache, || {
                reads.fetch_add(1, Ordering::Relaxed);
                57
            }),
            57
        );
        assert_eq!(
            cached_virtual_address_bits_with(&cache, || {
                reads.fetch_add(1, Ordering::Relaxed);
                48
            }),
            57
        );
        assert_eq!(reads.load(Ordering::Relaxed), 1);

        refetch_virtual_address_bits_with(&cache, || {
            reads.fetch_add(1, Ordering::Relaxed);
            48
        });
        assert_eq!(cached_virtual_address_bits_with(&cache, || 57), 48);
        assert_eq!(reads.load(Ordering::Relaxed), 2);

        let _: fn() = VirtAddrRT::refetch_virtual_address_bits;
    }

    #[test]
    fn current_validity_check_is_available_for_fixed_addresses() {
        let _: fn(VirtAddr48) -> bool = VirtAddr48::is_valid_currently;

        #[cfg(feature = "virt_addr_57")]
        let _: fn(VirtAddr57) -> bool = VirtAddr57::is_valid_currently;
    }
}
