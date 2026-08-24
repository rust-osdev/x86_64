//! Physical and virtual addresses manipulation

use core::convert::TryFrom;
use core::fmt;
use core::hash::Hash;
#[cfg(feature = "step_trait")]
use core::iter::Step;
use core::marker::PhantomData;
use core::ops::{Add, AddAssign, Sub, SubAssign};
#[cfg(feature = "memory_encryption")]
use core::sync::atomic::Ordering;

#[cfg(feature = "memory_encryption")]
use crate::structures::mem_encrypt::ENC_BIT_MASK;
use crate::structures::paging::page_table::PageTableLevel;
use crate::structures::paging::{PageOffset, PageTableIndex};

use dep_const_fn::const_fn;

#[cfg(feature = "virt_addr_rt")]
mod rt;
mod validity;

#[cfg(feature = "virt_addr_rt")]
pub use rt::{RuntimeValidity, VirtAddrRT};
pub(crate) use validity::ArithmeticValidity;
pub use validity::{FixedValidity, VirtAddrValidity};

/// Canonicalizes the given address with the given number of bits.
#[inline]
const fn canonicalize_with_bits(addr: u64, bits: usize) -> u64 {
    let shift = 64 - bits;
    ((addr << shift) as i64 >> shift) as u64
}

/// Tries to create a new canonical virtual address with the given number of bits.
///
/// # Safety
///
/// The caller must ensure that `bits` is valid for the selected validity policy. This is not
/// checked.
#[inline]
#[rustversion::attr(since(1.61), const)]
unsafe fn try_new_with_bits<V: VirtAddrValidity>(
    addr: u64,
    bits: usize,
) -> Result<VirtAddrGeneric<V>, VirtAddrNotValid> {
    let canonicalized = canonicalize_with_bits(addr, bits);
    if canonicalized == addr {
        Ok(VirtAddrGeneric(canonicalized, PhantomData))
    } else {
        Err(VirtAddrNotValid(addr))
    }
}

/// Creates a canonical virtual address by discarding invalid high bits, with the given number of
/// bits.
#[inline]
#[rustversion::attr(since(1.61), const)]
fn new_truncate_with_bits<V: VirtAddrValidity>(addr: u64, bits: usize) -> VirtAddrGeneric<V> {
    VirtAddrGeneric(canonicalize_with_bits(addr, bits), PhantomData)
}

/// A canonical 64-bit virtual memory address.
///
/// This is a wrapper type around an `u64`, so it is always 8 bytes, even when compiled
/// on non 64-bit systems. The
/// [`TryFrom`](https://doc.rust-lang.org/std/convert/trait.TryFrom.html) trait can be used for performing conversions
/// between `u64` and `usize`.
///
/// On `x86_64`, virtual addresses are canonical when all bits above the most significant valid bit
/// are copies of that bit. Currently, two address-space modes are supported on `x86_64`:
///
/// - Four-level paging (48-bit): The most significant valid bit is bit 47.
/// - Five-level paging (57-bit): The most significant valid bit is bit 56.
///
/// [`VirtAddrGeneric`] uses [`VirtAddrValidity`] to create different types of virtual addresses for
/// different modes:
///
/// - [`VirtAddr48`]: A virtual address that is canonical under four-level paging. (A 48-bit
///   canonical virtual address.)
/// - `VirtAddr57` (with `virt_addr_57`): A virtual address that is canonical under five-level
///   paging. (A 57-bit canonical virtual address.)
/// - `VirtAddrRT` (with `virt_addr_rt`): A virtual address that is canonical under the currently
///   active address-space mode. Validity is checked only when an address is created. A later
///   address-space mode change does not invalidate existing values.
///
/// [`VirtAddr48`] and `VirtAddr57` provide const-capable constructors and accessors.
/// `VirtAddrRT` can be stored, compared, formatted, inspected, and created through
/// [`zero`](Self::zero) or unsafe [`new_unsafe`](Self::new_unsafe) on all targets. Operations that
/// check the current address-space mode or produce a new runtime-valid address use a cached
/// virtual-address width. They require the `instructions` feature and an `x86_64` target, and they
/// must execute in Ring 0. The first such operation initializes the cache from `CR4.LA57`. Call
/// `VirtAddrRT::refetch_virtual_address_bits` after changing the active address-space mode.
///
/// Validity is checked only when an address is created. A later address-space mode change does not
/// invalidate existing values. Operations that subsequently produce a new address check the
/// result against the cached mode at that time. After changing `CR4.LA57`, update the cache before
/// creating or validating runtime-valid addresses. Use `is_valid_currently` to explicitly
/// revalidate an existing address when current-mode checks are available.
///
/// The validity parameter is intentionally required. Use [`VirtAddr`] when the validity should
/// follow the crate's feature-selected default.
///
/// ```compile_fail
/// use x86_64::addr::VirtAddrGeneric;
///
/// let _ = VirtAddrGeneric::zero();
/// ```
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct VirtAddrGeneric<V: VirtAddrValidity>(u64, PhantomData<V>);

/// A virtual address that is canonical under four-level paging.
pub type VirtAddr48 = VirtAddrGeneric<FixedValidity<48>>;

/// A virtual address that is canonical under five-level paging.
///
/// This alias is available with the `virt_addr_57` feature.
#[cfg(feature = "virt_addr_57")]
#[cfg_attr(feature = "doc_cfg", doc(cfg(feature = "virt_addr_57")))]
pub type VirtAddr57 = VirtAddrGeneric<FixedValidity<57>>;

/// The default virtual-address validity policy.
///
/// This is [`FixedValidity<48>`] by default and [`FixedValidity<57>`] when the
/// `default_virt_addr_57` feature is enabled.
#[cfg(not(feature = "default_virt_addr_57"))]
pub type DefaultVirtAddrValidity = FixedValidity<48>;

/// The default virtual-address validity policy.
///
/// This is [`FixedValidity<48>`] by default and [`FixedValidity<57>`] when the
/// `default_virt_addr_57` feature is enabled.
#[cfg(feature = "default_virt_addr_57")]
pub type DefaultVirtAddrValidity = FixedValidity<57>;

/// The default virtual address type.
///
/// This is an alias for [`VirtAddr48`] by default and `VirtAddr57` when the
/// `default_virt_addr_57` feature is enabled.
#[cfg(not(feature = "default_virt_addr_57"))]
pub type VirtAddr = VirtAddr48;

/// The default virtual address type.
///
/// This is an alias for [`VirtAddr48`] by default and [`VirtAddr57`] when the
/// `default_virt_addr_57` feature is enabled.
#[cfg(feature = "default_virt_addr_57")]
pub type VirtAddr = VirtAddr57;

/// A 64-bit physical memory address.
///
/// This is a wrapper type around an `u64`, so it is always 8 bytes, even when compiled
/// on non 64-bit systems. The
/// [`TryFrom`](https://doc.rust-lang.org/std/convert/trait.TryFrom.html) trait can be used for performing conversions
/// between `u64` and `usize`.
///
/// On `x86_64`, only the 52 lower bits of a physical address can be used. The top 12 bits need
/// to be zero. This type guarantees that it always represents a valid physical address.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct PhysAddr(u64);

/// A passed `u64` was not a valid virtual address.
///
/// Automatic sign extension for the selected validity policy would have overwritten possibly
/// meaningful bits. This likely indicates a bug, for example an invalid address calculation.
///
/// Contains the invalid address.
pub struct VirtAddrNotValid(pub u64);

impl core::fmt::Debug for VirtAddrNotValid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("VirtAddrNotValid")
            .field(&format_args!("{:#x}", self.0))
            .finish()
    }
}

impl<const BITS: usize> VirtAddrGeneric<FixedValidity<BITS>>
where
    FixedValidity<BITS>: VirtAddrValidity,
{
    /// Creates a new canonical virtual address, with provided fixed width.
    ///
    /// The provided address should already be canonical. If you want to check
    /// whether an address is canonical, use [`try_new`](Self::try_new).
    ///
    /// ## Panics
    ///
    /// This function panics if the address is not canonical for the selected fixed width.
    #[inline]
    #[rustversion::attr(since(1.61), const)]
    pub fn new(addr: u64) -> Self {
        // TODO: Replace with .ok().expect(msg) when that works on stable.
        match Self::try_new(addr) {
            Ok(v) => v,
            Err(_) => panic!("virtual address must be canonical for the selected fixed width"),
        }
    }

    /// Tries to create a new canonical virtual address, with provided fixed width.
    ///
    /// This function checks whether the given address is canonical for the selected fixed width
    /// and returns an error otherwise.
    #[inline]
    #[rustversion::attr(since(1.61), const)]
    pub fn try_new(addr: u64) -> Result<Self, VirtAddrNotValid> {
        // SAFETY: `BITS` is valid for `FixedValidity<BITS>`, so this is safe.
        unsafe { try_new_with_bits(addr, BITS) }
    }

    /// Creates a canonical virtual address by discarding invalid high bits, with provided fixed
    /// width.
    ///
    /// This function sign-extends the selected fixed-width sign bit. If you want to check whether
    /// an address is canonical, use [`new`](Self::new) or [`try_new`](Self::try_new).
    #[inline]
    #[rustversion::attr(since(1.61), const)]
    pub fn new_truncate(addr: u64) -> Self {
        new_truncate_with_bits(addr, BITS)
    }

    /// Creates a fixed-width virtual address from the given pointer.
    ///
    /// The pointer address must be canonical under the selected fixed validity policy.
    #[cfg(target_pointer_width = "64")]
    #[inline]
    pub fn from_ptr<T: ?Sized>(ptr: *const T) -> Self {
        Self::new(ptr as *const () as u64)
    }

    /// Aligns the virtual address upwards to the given alignment.
    ///
    /// See the [`align_up`] function for more information.
    #[inline]
    pub fn align_up<U>(self, align: U) -> Self
    where
        U: Into<u64>,
    {
        Self::new_truncate(align_up(self.0, align.into()))
    }

    /// Aligns the virtual address downwards to the given alignment.
    ///
    /// See the [`align_down`] function for more information.
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
    #[rustversion::attr(since(1.61), const)]
    pub(crate) fn align_down_u64(self, align: u64) -> Self {
        Self::new_truncate(align_down(self.0, align))
    }
}

impl<V: VirtAddrValidity> VirtAddrGeneric<V> {
    /// Creates a new virtual address, without any checks.
    ///
    /// ## Safety
    ///
    /// The caller must ensure that `addr` is valid for `V`. This is not checked.
    #[inline]
    #[rustversion::attr(since(1.61), const)]
    pub unsafe fn new_unsafe(addr: u64) -> Self {
        VirtAddrGeneric(addr, PhantomData)
    }

    /// Creates a virtual address that points to `0`.
    #[inline]
    #[rustversion::attr(since(1.61), const)]
    pub fn zero() -> Self {
        VirtAddrGeneric(0, PhantomData)
    }

    /// Converts the address to an `u64`.
    #[inline]
    #[rustversion::attr(since(1.61), const)]
    pub fn as_u64(self) -> u64 {
        self.0
    }

    /// Converts the address to a raw pointer.
    #[cfg(target_pointer_width = "64")]
    #[inline]
    #[rustversion::attr(since(1.61), const)]
    pub fn as_ptr<T>(self) -> *const T {
        self.as_u64() as *const T
    }

    /// Converts the address to a mutable raw pointer.
    #[cfg(target_pointer_width = "64")]
    #[inline]
    #[rustversion::attr(since(1.61), const)]
    pub fn as_mut_ptr<T>(self) -> *mut T {
        self.as_ptr::<T>() as *mut T
    }

    /// Convenience method for checking if a virtual address is null.
    #[inline]
    #[rustversion::attr(since(1.61), const)]
    pub fn is_null(self) -> bool {
        self.0 == 0
    }

    /// Returns the 12-bit page offset of this virtual address.
    #[inline]
    #[rustversion::attr(since(1.61), const)]
    pub fn page_offset(self) -> PageOffset {
        PageOffset::new_truncate(self.0 as u16)
    }

    /// Returns the 9-bit level 1 page table index.
    #[inline]
    #[rustversion::attr(since(1.61), const)]
    pub fn p1_index(self) -> PageTableIndex {
        PageTableIndex::new_truncate((self.0 >> 12) as u16)
    }

    /// Returns the 9-bit level 2 page table index.
    #[inline]
    #[rustversion::attr(since(1.61), const)]
    pub fn p2_index(self) -> PageTableIndex {
        PageTableIndex::new_truncate((self.0 >> 12 >> 9) as u16)
    }

    /// Returns the 9-bit level 3 page table index.
    #[inline]
    #[rustversion::attr(since(1.61), const)]
    pub fn p3_index(self) -> PageTableIndex {
        PageTableIndex::new_truncate((self.0 >> 12 >> 9 >> 9) as u16)
    }

    /// Returns the 9-bit level 4 page table index.
    #[inline]
    #[rustversion::attr(since(1.61), const)]
    pub fn p4_index(self) -> PageTableIndex {
        PageTableIndex::new_truncate((self.0 >> 12 >> 9 >> 9 >> 9) as u16)
    }

    /// Returns the 9-bit level page table index.
    #[inline]
    #[rustversion::attr(since(1.61), const)]
    pub fn page_table_index(self, level: PageTableLevel) -> PageTableIndex {
        PageTableIndex::new_truncate((self.0 >> 12 >> ((level as u8 - 1) * 9)) as u16)
    }
}

#[cfg(feature = "virt_addr_57")]
impl VirtAddrGeneric<FixedValidity<57>> {
    /// Returns the 9-bit level 5 page table index.
    #[inline]
    #[rustversion::attr(since(1.61), const)]
    pub fn p5_index(self) -> PageTableIndex {
        PageTableIndex::new_truncate((self.0 >> 12 >> 9 >> 9 >> 9 >> 9) as u16)
    }
}

impl<V: VirtAddrValidity> VirtAddrGeneric<V> {
    /// Checks whether the virtual address has the demanded alignment.
    #[inline]
    pub fn is_aligned<U>(self, align: U) -> bool
    where
        U: Into<u64>,
    {
        self.is_aligned_u64(align.into())
    }

    /// Checks whether the virtual address has the demanded alignment.
    #[inline]
    #[rustversion::attr(since(1.61), const)]
    pub(crate) fn is_aligned_u64(self, align: u64) -> bool {
        align_down(self.0, align) == self.0
    }

    /// Creates a checked virtual address for an internal policy-generic API.
    ///
    /// Runtime policies use the cached current address-space mode during this construction.
    #[inline]
    pub(crate) fn new_with_validity(addr: u64) -> Self {
        // SAFETY: `V::bits()` is valid for `V`, so this is safe.
        match unsafe { try_new_with_bits(addr, V::bits()) } {
            Ok(address) => address,
            Err(_) => panic!("virtual address must be canonical for its validity policy"),
        }
    }

    /// Returns the first address in the upper canonical half for this policy.
    #[inline]
    pub(crate) fn upper_half_start() -> Self {
        new_truncate_with_bits(1u64 << (V::bits() - 1), V::bits())
    }

    /// Returns the final address in the lower canonical half for this policy.
    #[inline]
    pub(crate) fn lower_half_end() -> Self {
        unsafe { Self::new_unsafe((1u64 << (V::bits() - 1)) - 1) }
    }

    /// Returns the greatest canonical address for this policy.
    #[inline]
    pub(crate) fn max_value() -> Self {
        unsafe { Self::new_unsafe(u64::MAX) }
    }

    /// Tries to create a checked virtual address for an internal policy-generic API.
    ///
    /// Runtime policies use the cached current address-space mode during this construction.
    #[inline]
    pub(crate) fn try_new_with_validity(addr: u64) -> Result<Self, VirtAddrNotValid> {
        // SAFETY: `V::bits()` is valid for `V`, so this is safe.
        unsafe { try_new_with_bits(addr, V::bits()) }
    }

    #[inline]
    fn new_truncate_with_validity(addr: u64) -> Self {
        VirtAddrGeneric(canonicalize_with_bits(addr, V::bits()), PhantomData)
    }

    // FIXME: Move this into the `Step` impl, once `Step` is stabilized.
    #[cfg(feature = "step_trait")]
    pub(crate) fn steps_between_impl(start: &Self, end: &Self) -> (usize, Option<usize>) {
        if let Some(steps) = Self::steps_between_u64(start, end) {
            let steps = usize::try_from(steps).ok();
            (steps.unwrap_or(usize::MAX), steps)
        } else {
            (0, None)
        }
    }

    /// An implementation of steps_between that returns u64. Note that this
    /// function always returns the exact bound, so it doesn't need to return a
    /// lower and upper bound like steps_between does.
    pub(crate) fn steps_between_u64(start: &Self, end: &Self) -> Option<u64> {
        let mask = (1u64 << V::bits()) - 1;
        (end.0 & mask).checked_sub(start.0 & mask)
    }

    // FIXME: Move this into the `Step` impl, once `Step` is stabilized.
    #[inline]
    pub(crate) fn forward_checked_impl(start: Self, count: usize) -> Option<Self> {
        Self::forward_checked_u64(start, u64::try_from(count).ok()?)
    }

    /// An implementation of forward_checked that takes u64 instead of usize.
    #[inline]
    pub(crate) fn forward_checked_u64(start: Self, count: u64) -> Option<Self> {
        let mask = (1u64 << V::bits()) - 1;
        let addr = (start.0 & mask).checked_add(count)?;
        if addr > mask {
            None
        } else {
            Some(Self::new_truncate_with_validity(addr))
        }
    }

    /// An implementation of backward_checked that takes u64 instead of usize.
    #[cfg(feature = "step_trait")]
    #[inline]
    pub(crate) fn backward_checked_u64(start: Self, count: u64) -> Option<Self> {
        let mask = (1u64 << V::bits()) - 1;
        let addr = (start.0 & mask).checked_sub(count)?;
        Some(Self::new_truncate_with_validity(addr))
    }
}

impl<V: VirtAddrValidity> fmt::Debug for VirtAddrGeneric<V> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_tuple("VirtAddr")
            .field(&format_args!("{:#x}", self.0))
            .finish()
    }
}

impl<V: VirtAddrValidity> fmt::Binary for VirtAddrGeneric<V> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Binary::fmt(&self.0, f)
    }
}

impl<V: VirtAddrValidity> fmt::LowerHex for VirtAddrGeneric<V> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::LowerHex::fmt(&self.0, f)
    }
}

impl<V: VirtAddrValidity> fmt::Octal for VirtAddrGeneric<V> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Octal::fmt(&self.0, f)
    }
}

impl<V: VirtAddrValidity> fmt::UpperHex for VirtAddrGeneric<V> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::UpperHex::fmt(&self.0, f)
    }
}

impl<V: VirtAddrValidity> fmt::Pointer for VirtAddrGeneric<V> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Pointer::fmt(&(self.0 as *const ()), f)
    }
}

impl<V: ArithmeticValidity> Add<u64> for VirtAddrGeneric<V> {
    type Output = Self;

    #[cfg_attr(not(feature = "step_trait"), allow(rustdoc::broken_intra_doc_links))]
    /// Add an offset to a virtual address.
    ///
    /// This function performs normal arithmetic addition and doesn't jump the
    /// address gap. If you're looking for a successor operation that jumps the
    /// address gap, use [`Step::forward`].
    ///
    /// # Panics
    ///
    /// This function will panic on overflow or if the result is not a
    /// canonical address.
    #[inline]
    fn add(self, rhs: u64) -> Self::Output {
        Self::try_new_with_validity(
            self.0
                .checked_add(rhs)
                .expect("attempt to add with overflow"),
        )
        .expect("attempt to add resulted in non-canonical virtual address")
    }
}

impl<V: ArithmeticValidity> AddAssign<u64> for VirtAddrGeneric<V> {
    #[cfg_attr(not(feature = "step_trait"), allow(rustdoc::broken_intra_doc_links))]
    /// Add an offset to a virtual address.
    ///
    /// This function performs normal arithmetic addition and doesn't jump the
    /// address gap. If you're looking for a successor operation that jumps the
    /// address gap, use [`Step::forward`].
    ///
    /// # Panics
    ///
    /// This function will panic on overflow or if the result is not a
    /// canonical address.
    #[inline]
    fn add_assign(&mut self, rhs: u64) {
        *self = *self + rhs;
    }
}

impl<V: ArithmeticValidity> Sub<u64> for VirtAddrGeneric<V> {
    type Output = Self;

    #[cfg_attr(not(feature = "step_trait"), allow(rustdoc::broken_intra_doc_links))]
    /// Subtract an offset from a virtual address.
    ///
    /// This function performs normal arithmetic subtraction and doesn't jump
    /// the address gap. If you're looking for a predecessor operation that
    /// jumps the address gap, use [`Step::backward`].
    ///
    /// # Panics
    ///
    /// This function will panic on overflow or if the result is not a
    /// canonical address.
    #[inline]
    fn sub(self, rhs: u64) -> Self::Output {
        Self::try_new_with_validity(
            self.0
                .checked_sub(rhs)
                .expect("attempt to subtract with overflow"),
        )
        .expect("attempt to subtract resulted in non-canonical virtual address")
    }
}

impl<V: ArithmeticValidity> SubAssign<u64> for VirtAddrGeneric<V> {
    #[cfg_attr(not(feature = "step_trait"), allow(rustdoc::broken_intra_doc_links))]
    /// Subtract an offset from a virtual address.
    ///
    /// This function performs normal arithmetic subtraction and doesn't jump
    /// the address gap. If you're looking for a predecessor operation that
    /// jumps the address gap, use [`Step::backward`].
    ///
    /// # Panics
    ///
    /// This function will panic on overflow or if the result is not a
    /// canonical address.
    #[inline]
    fn sub_assign(&mut self, rhs: u64) {
        *self = *self - rhs;
    }
}

impl<V: VirtAddrValidity> Sub<VirtAddrGeneric<V>> for VirtAddrGeneric<V> {
    type Output = u64;

    /// Returns the difference between two addresses.
    ///
    /// # Panics
    ///
    /// This function will panic on overflow.
    #[inline]
    fn sub(self, rhs: VirtAddrGeneric<V>) -> Self::Output {
        self.as_u64()
            .checked_sub(rhs.as_u64())
            .expect("attempt to subtract with overflow")
    }
}

#[cfg(feature = "virt_addr_57")]
impl From<VirtAddr48> for VirtAddr57 {
    #[inline]
    fn from(address: VirtAddr48) -> Self {
        unsafe { Self::new_unsafe(address.as_u64()) }
    }
}

#[cfg(feature = "virt_addr_57")]
impl TryFrom<VirtAddr57> for VirtAddr48 {
    type Error = VirtAddrNotValid;

    #[inline]
    fn try_from(address: VirtAddr57) -> Result<Self, Self::Error> {
        Self::try_new(address.as_u64())
    }
}

#[cfg(feature = "step_trait")]
impl<V: ArithmeticValidity> Step for VirtAddrGeneric<V> {
    #[inline]
    fn steps_between(start: &Self, end: &Self) -> (usize, Option<usize>) {
        Self::steps_between_impl(start, end)
    }

    #[inline]
    fn forward_checked(start: Self, count: usize) -> Option<Self> {
        Self::forward_checked_impl(start, count)
    }

    #[inline]
    fn backward_checked(start: Self, count: usize) -> Option<Self> {
        Self::backward_checked_u64(start, u64::try_from(count).ok()?)
    }

    // Kani's bundled toolchain predates these methods being added to `Step`.
    // Exclude them there so the crate still compiles under `cargo kani`.
    // This can be removed once Kani upgrades its bundled toolchain to nightly-2026-07-10 or later.
    #[cfg(not(kani))]
    #[inline]
    fn forward_overflowing(start: Self, count: usize) -> (Self, bool) {
        match Self::forward_checked(start, count) {
            Some(next) => (next, false),
            None => (start, true),
        }
    }

    // Kani's bundled toolchain predates these methods being added to `Step`.
    // Exclude them there so the crate still compiles under `cargo kani`.
    // This can be removed once Kani upgrades its bundled toolchain to nightly-2026-07-10 or later.
    #[cfg(not(kani))]
    #[inline]
    fn backward_overflowing(start: Self, count: usize) -> (Self, bool) {
        match Self::backward_checked(start, count) {
            Some(next) => (next, false),
            None => (start, true),
        }
    }
}

#[cfg(kani)]
impl<const BITS: usize> kani::Arbitrary for VirtAddrGeneric<FixedValidity<BITS>>
where
    FixedValidity<BITS>: VirtAddrValidity,
{
    fn any() -> Self {
        Self::new_truncate(kani::any())
    }
}

/// A passed `u64` was not a valid physical address.
///
/// This means that bits 52 to 64 were not all null.
///
/// Contains the invalid address.
pub struct PhysAddrNotValid(pub u64);

impl core::fmt::Debug for PhysAddrNotValid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("PhysAddrNotValid")
            .field(&format_args!("{:#x}", self.0))
            .finish()
    }
}

impl PhysAddr {
    /// Creates a new physical address.
    ///
    /// ## Panics
    ///
    /// This function panics if a bit in the range 52 to 64 is set.
    ///
    /// If the `memory_encryption` feature has been enabled and an encryption bit has been
    /// configured, this also panics if the encryption bit is manually set in the address.
    #[inline]
    #[const_fn(cfg(not(feature = "memory_encryption")))]
    pub const fn new(addr: u64) -> Self {
        // TODO: Replace with .ok().expect(msg) when that works on stable.
        match Self::try_new(addr) {
            Ok(p) => p,
            Err(_) => panic!("physical addresses must not have any bits in the range 52 to 64 set"),
        }
    }

    /// Creates a new physical address, throwing bits 52..64 away.
    #[cfg(not(feature = "memory_encryption"))]
    #[inline]
    pub const fn new_truncate(addr: u64) -> PhysAddr {
        PhysAddr(addr % (1 << 52))
    }

    /// Creates a new physical address, throwing bits 52..64 and the encryption bit away.
    #[cfg(feature = "memory_encryption")]
    #[inline]
    pub fn new_truncate(addr: u64) -> PhysAddr {
        PhysAddr((addr % (1 << 52)) & !ENC_BIT_MASK.load(Ordering::Relaxed))
    }

    /// Creates a new physical address, without any checks.
    ///
    /// ## Safety
    ///
    /// You must make sure bits 52..64 are zero. This is not checked.
    #[inline]
    pub const unsafe fn new_unsafe(addr: u64) -> PhysAddr {
        PhysAddr(addr)
    }

    /// Tries to create a new physical address.
    ///
    /// Fails if any bits in the range 52 to 64 are set.
    /// If the `memory_encryption` feature has been enabled and an encryption bit has been
    /// configured, this also fails if the encryption bit is manually set in the address.
    #[inline]
    #[const_fn(cfg(not(feature = "memory_encryption")))]
    pub const fn try_new(addr: u64) -> Result<Self, PhysAddrNotValid> {
        let p = Self::new_truncate(addr);
        if p.0 == addr {
            Ok(p)
        } else {
            Err(PhysAddrNotValid(addr))
        }
    }

    /// Creates a physical address that points to `0`.
    #[inline]
    pub const fn zero() -> PhysAddr {
        PhysAddr(0)
    }

    /// Converts the address to an `u64`.
    #[inline]
    pub const fn as_u64(self) -> u64 {
        self.0
    }

    /// Convenience method for checking if a physical address is null.
    #[inline]
    pub const fn is_null(self) -> bool {
        self.0 == 0
    }

    /// Aligns the physical address upwards to the given alignment.
    ///
    /// See the `align_up` function for more information.
    ///
    /// # Panics
    ///
    /// This function panics if the resulting address has a bit in the range 52
    /// to 64 set.
    #[inline]
    pub fn align_up<U>(self, align: U) -> Self
    where
        U: Into<u64>,
    {
        PhysAddr::new(align_up(self.0, align.into()))
    }

    /// Aligns the physical address downwards to the given alignment.
    ///
    /// See the `align_down` function for more information.
    #[inline]
    pub fn align_down<U>(self, align: U) -> Self
    where
        U: Into<u64>,
    {
        self.align_down_u64(align.into())
    }

    /// Aligns the physical address downwards to the given alignment.
    ///
    /// See the `align_down` function for more information.
    #[inline]
    pub(crate) const fn align_down_u64(self, align: u64) -> Self {
        PhysAddr(align_down(self.0, align))
    }

    /// Checks whether the physical address has the demanded alignment.
    #[inline]
    pub fn is_aligned<U>(self, align: U) -> bool
    where
        U: Into<u64>,
    {
        self.is_aligned_u64(align.into())
    }

    /// Checks whether the physical address has the demanded alignment.
    #[inline]
    pub(crate) const fn is_aligned_u64(self, align: u64) -> bool {
        self.align_down_u64(align).as_u64() == self.as_u64()
    }
}

impl fmt::Debug for PhysAddr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_tuple("PhysAddr")
            .field(&format_args!("{:#x}", self.0))
            .finish()
    }
}

impl fmt::Binary for PhysAddr {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Binary::fmt(&self.0, f)
    }
}

impl fmt::LowerHex for PhysAddr {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::LowerHex::fmt(&self.0, f)
    }
}

impl fmt::Octal for PhysAddr {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Octal::fmt(&self.0, f)
    }
}

impl fmt::UpperHex for PhysAddr {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::UpperHex::fmt(&self.0, f)
    }
}

impl fmt::Pointer for PhysAddr {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Pointer::fmt(&(self.0 as *const ()), f)
    }
}

impl Add<u64> for PhysAddr {
    type Output = Self;
    #[inline]
    fn add(self, rhs: u64) -> Self::Output {
        PhysAddr::new(self.0.checked_add(rhs).unwrap())
    }
}

impl AddAssign<u64> for PhysAddr {
    #[inline]
    fn add_assign(&mut self, rhs: u64) {
        *self = *self + rhs;
    }
}

impl Sub<u64> for PhysAddr {
    type Output = Self;
    #[inline]
    fn sub(self, rhs: u64) -> Self::Output {
        PhysAddr::new(self.0.checked_sub(rhs).unwrap())
    }
}

impl SubAssign<u64> for PhysAddr {
    #[inline]
    fn sub_assign(&mut self, rhs: u64) {
        *self = *self - rhs;
    }
}

impl Sub<PhysAddr> for PhysAddr {
    type Output = u64;
    #[inline]
    fn sub(self, rhs: PhysAddr) -> Self::Output {
        self.as_u64().checked_sub(rhs.as_u64()).unwrap()
    }
}

#[cfg(kani)]
impl kani::Arbitrary for PhysAddr {
    fn any() -> Self {
        Self::new_truncate(kani::any())
    }
}

/// Align address downwards.
///
/// Returns the greatest `x` with alignment `align` so that `x <= addr`.
///
/// Panics if the alignment is not a power of two.
#[inline]
pub const fn align_down(addr: u64, align: u64) -> u64 {
    assert!(align.is_power_of_two(), "`align` must be a power of two");
    addr & !(align - 1)
}

/// Align address upwards.
///
/// Returns the smallest `x` with alignment `align` so that `x >= addr`.
///
/// Panics if the alignment is not a power of two or if an overflow occurs.
#[inline]
pub const fn align_up(addr: u64, align: u64) -> u64 {
    assert!(align.is_power_of_two(), "`align` must be a power of two");
    let align_mask = align - 1;
    if addr & align_mask == 0 {
        addr // already aligned
    } else {
        // FIXME: Replace with .expect, once `Option::expect` is const.
        if let Some(aligned) = (addr | align_mask).checked_add(1) {
            aligned
        } else {
            panic!("attempt to add with overflow")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Constructs an unchecked VA48 value for tests of internal arithmetic behavior.
    ///
    /// This helper preserves the concise tuple-constructor spelling used by the original tests.
    #[allow(non_snake_case)]
    fn VirtAddr(addr: u64) -> VirtAddr48 {
        unsafe { VirtAddr48::new_unsafe(addr) }
    }

    #[rustversion::since(1.61)]
    const UNSAFE_VIRT_ADDR_48: VirtAddr48 = unsafe { VirtAddr48::new_unsafe(0x1234) };
    #[rustversion::since(1.61)]
    #[cfg(feature = "virt_addr_57")]
    const UNSAFE_VIRT_ADDR_57: VirtAddr57 = unsafe { VirtAddr57::new_unsafe(0x1234) };
    #[rustversion::since(1.61)]
    #[cfg(feature = "virt_addr_rt")]
    const UNSAFE_VIRT_ADDR_RT: VirtAddrRT = unsafe { VirtAddrRT::new_unsafe(0x1234) };

    #[rustversion::since(1.61)]
    #[test]
    #[cfg(not(feature = "default_virt_addr_57"))]
    fn default_virtaddr_is_va48() {
        let _: fn(u64) -> VirtAddr48 = crate::VirtAddr::new;

        const FIXED48: VirtAddr48 = VirtAddr48::new(0x1234);
        const GENERIC48: VirtAddrGeneric<FixedValidity<48>> =
            VirtAddrGeneric::<FixedValidity<48>>::new(0x1234);
        assert_eq!(FIXED48.as_u64(), 0x1234);
        assert_eq!(GENERIC48.as_u64(), 0x1234);
    }

    #[rustversion::since(1.61)]
    #[test]
    #[cfg(feature = "default_virt_addr_57")]
    fn configured_default_virtaddr_is_va57() {
        let _: fn(u64) -> VirtAddr57 = crate::VirtAddr::new;

        const FIXED57: VirtAddr57 = VirtAddr57::new(0x00ff_0000_0000_0000);
        assert_eq!(FIXED57.as_u64(), 0x00ff_0000_0000_0000);
    }

    #[test]
    fn fixed_virtaddr_canonicality() {
        assert!(VirtAddr48::try_new(0x0000_7fff_ffff_ffff).is_ok());
        assert!(VirtAddr48::try_new(0x0000_8000_0000_0000).is_err());
        assert!(VirtAddr48::try_new(0xffff_8000_0000_0000).is_ok());

        #[cfg(feature = "virt_addr_57")]
        {
            assert!(VirtAddr57::try_new(0x00ff_ffff_ffff_ffff).is_ok());
            assert!(VirtAddr57::try_new(0x0100_0000_0000_0000).is_err());
            assert!(VirtAddr57::try_new(0xff00_0000_0000_0000).is_ok());
            assert!(VirtAddr57::try_new(0x0000_8000_0000_0000).is_ok());
        }
    }

    #[test]
    fn pure_canonicalization_uses_selected_width() {
        assert_eq!(canonicalize_with_bits(1 << 47, 48), 0xffff_8000_0000_0000);
        assert_eq!(canonicalize_with_bits(1 << 56, 57), 0xff00_0000_0000_0000);
        assert_eq!(canonicalize_with_bits((1 << 47) - 1, 48), (1 << 47) - 1);
        assert_eq!(canonicalize_with_bits((1 << 56) - 1, 57), (1 << 56) - 1);
    }

    #[test]
    #[cfg(all(feature = "step_trait", feature = "virt_addr_57"))]
    fn fixed_virtaddr_operations_use_policy_width() {
        let low_end = VirtAddr57::new(0x00ff_ffff_ffff_fffe);
        assert_eq!((low_end + 1).as_u64(), 0x00ff_ffff_ffff_ffff);
        assert_eq!(
            Step::forward(low_end + 1, 1).as_u64(),
            0xff00_0000_0000_0000
        );
        assert_eq!(
            Step::backward(VirtAddr57::new(0xff00_0000_0000_0000), 1).as_u64(),
            0x00ff_ffff_ffff_ffff
        );
        assert_eq!(
            VirtAddr57::new(0x00ff_ffff_ffff_ffff)
                .align_up(2u64)
                .as_u64(),
            0xff00_0000_0000_0000
        );
    }

    #[test]
    #[cfg(feature = "virt_addr_57")]
    fn fixed_virtaddr_conversions_preserve_or_check_values() {
        let address48 = VirtAddr48::new(0xffff_8000_0000_1234);
        let address57 = VirtAddr57::from(address48);

        assert_eq!(address57.as_u64(), address48.as_u64());
        assert_eq!(VirtAddr48::try_from(address57).unwrap(), address48);

        let la57_only = VirtAddr57::new(0x0000_8000_0000_0000);
        assert!(VirtAddr48::try_from(la57_only).is_err());
    }

    #[test]
    fn virtaddr_policy_layout_is_transparent() {
        assert_eq!(
            core::mem::size_of::<VirtAddr48>(),
            core::mem::size_of::<u64>()
        );
        #[cfg(feature = "virt_addr_57")]
        assert_eq!(
            core::mem::size_of::<VirtAddr57>(),
            core::mem::size_of::<u64>()
        );
        #[cfg(feature = "virt_addr_rt")]
        assert_eq!(
            core::mem::size_of::<VirtAddrRT>(),
            core::mem::size_of::<u64>()
        );
        assert_eq!(
            core::mem::align_of::<VirtAddr48>(),
            core::mem::align_of::<u64>()
        );
        #[cfg(feature = "virt_addr_57")]
        assert_eq!(
            core::mem::align_of::<VirtAddr57>(),
            core::mem::align_of::<u64>()
        );
        #[cfg(feature = "virt_addr_rt")]
        assert_eq!(
            core::mem::align_of::<VirtAddrRT>(),
            core::mem::align_of::<u64>()
        );
    }

    #[rustversion::since(1.61)]
    #[test]
    fn new_unsafe_is_const_for_all_policies() {
        assert_eq!(UNSAFE_VIRT_ADDR_48.as_u64(), 0x1234);
        #[cfg(feature = "virt_addr_57")]
        assert_eq!(UNSAFE_VIRT_ADDR_57.as_u64(), 0x1234);
        #[cfg(feature = "virt_addr_rt")]
        assert_eq!(UNSAFE_VIRT_ADDR_RT.as_u64(), 0x1234);
    }

    #[test]
    #[should_panic]
    pub fn add_overflow_virtaddr() {
        let _ = VirtAddr48::new(0xffff_ffff_ffff_ffff) + 1;
    }

    #[test]
    #[should_panic]
    pub fn add_overflow_physaddr() {
        let _ = PhysAddr::new(0x000f_ffff_ffff_ffff) + 0xffff_0000_0000_0000;
    }

    #[test]
    #[should_panic]
    pub fn sub_underflow_virtaddr() {
        let _ = VirtAddr48::new(0) - 1;
    }

    #[test]
    #[should_panic]
    pub fn sub_overflow_physaddr() {
        let _ = PhysAddr::new(0) - 1;
    }

    #[test]
    pub fn virtaddr_new_truncate() {
        assert_eq!(VirtAddr48::new_truncate(0), VirtAddr(0));
        assert_eq!(VirtAddr48::new_truncate(1 << 47), VirtAddr(0xfffff << 47));
        assert_eq!(VirtAddr48::new_truncate(123), VirtAddr(123));
        assert_eq!(VirtAddr48::new_truncate(123 << 47), VirtAddr(0xfffff << 47));
    }

    #[test]
    #[cfg(feature = "step_trait")]
    fn virtaddr_step_forward() {
        assert_eq!(Step::forward(VirtAddr(0), 0), VirtAddr(0));
        assert_eq!(Step::forward(VirtAddr(0), 1), VirtAddr(1));
        assert_eq!(
            Step::forward(VirtAddr(0x7fff_ffff_ffff), 1),
            VirtAddr(0xffff_8000_0000_0000)
        );
        assert_eq!(
            Step::forward(VirtAddr(0xffff_8000_0000_0000), 1),
            VirtAddr(0xffff_8000_0000_0001)
        );
        assert_eq!(
            Step::forward_checked(VirtAddr(0xffff_ffff_ffff_ffff), 1),
            None
        );
        #[cfg(target_pointer_width = "64")]
        assert_eq!(
            Step::forward(VirtAddr(0x7fff_ffff_ffff), 0x1234_5678_9abd),
            VirtAddr(0xffff_9234_5678_9abc)
        );
        #[cfg(target_pointer_width = "64")]
        assert_eq!(
            Step::forward(VirtAddr(0x7fff_ffff_ffff), 0x8000_0000_0000),
            VirtAddr(0xffff_ffff_ffff_ffff)
        );
        #[cfg(target_pointer_width = "64")]
        assert_eq!(
            Step::forward(VirtAddr(0x7fff_ffff_ff00), 0x8000_0000_00ff),
            VirtAddr(0xffff_ffff_ffff_ffff)
        );
        #[cfg(target_pointer_width = "64")]
        assert_eq!(
            Step::forward_checked(VirtAddr(0x7fff_ffff_ff00), 0x8000_0000_0100),
            None
        );
        #[cfg(target_pointer_width = "64")]
        assert_eq!(
            Step::forward_checked(VirtAddr(0x7fff_ffff_ffff), 0x8000_0000_0001),
            None
        );
    }

    #[test]
    #[cfg(feature = "step_trait")]
    fn virtaddr_step_backward() {
        assert_eq!(Step::backward(VirtAddr(0), 0), VirtAddr(0));
        assert_eq!(Step::backward_checked(VirtAddr(0), 1), None);
        assert_eq!(Step::backward(VirtAddr(1), 1), VirtAddr(0));
        assert_eq!(
            Step::backward(VirtAddr(0xffff_8000_0000_0000), 1),
            VirtAddr(0x7fff_ffff_ffff)
        );
        assert_eq!(
            Step::backward(VirtAddr(0xffff_8000_0000_0001), 1),
            VirtAddr(0xffff_8000_0000_0000)
        );
        #[cfg(target_pointer_width = "64")]
        assert_eq!(
            Step::backward(VirtAddr(0xffff_9234_5678_9abc), 0x1234_5678_9abd),
            VirtAddr(0x7fff_ffff_ffff)
        );
        #[cfg(target_pointer_width = "64")]
        assert_eq!(
            Step::backward(VirtAddr(0xffff_8000_0000_0000), 0x8000_0000_0000),
            VirtAddr(0)
        );
        #[cfg(target_pointer_width = "64")]
        assert_eq!(
            Step::backward(VirtAddr(0xffff_8000_0000_0000), 0x7fff_ffff_ff01),
            VirtAddr(0xff)
        );
        #[cfg(target_pointer_width = "64")]
        assert_eq!(
            Step::backward_checked(VirtAddr(0xffff_8000_0000_0000), 0x8000_0000_0001),
            None
        );
    }

    #[test]
    #[cfg(feature = "step_trait")]
    fn virtaddr_steps_between() {
        assert_eq!(
            Step::steps_between(&VirtAddr(0), &VirtAddr(0)),
            (0, Some(0))
        );
        assert_eq!(
            Step::steps_between(&VirtAddr(0), &VirtAddr(1)),
            (1, Some(1))
        );
        assert_eq!(Step::steps_between(&VirtAddr(1), &VirtAddr(0)), (0, None));
        assert_eq!(
            Step::steps_between(
                &VirtAddr(0x7fff_ffff_ffff),
                &VirtAddr(0xffff_8000_0000_0000)
            ),
            (1, Some(1))
        );
        assert_eq!(
            Step::steps_between(
                &VirtAddr(0xffff_8000_0000_0000),
                &VirtAddr(0x7fff_ffff_ffff)
            ),
            (0, None)
        );
        assert_eq!(
            Step::steps_between(
                &VirtAddr(0xffff_8000_0000_0000),
                &VirtAddr(0xffff_8000_0000_0000)
            ),
            (0, Some(0))
        );
        assert_eq!(
            Step::steps_between(
                &VirtAddr(0xffff_8000_0000_0000),
                &VirtAddr(0xffff_8000_0000_0001)
            ),
            (1, Some(1))
        );
        assert_eq!(
            Step::steps_between(
                &VirtAddr(0xffff_8000_0000_0001),
                &VirtAddr(0xffff_8000_0000_0000)
            ),
            (0, None)
        );
        // Make sure that we handle `steps > u32::MAX` correctly on 32-bit
        // targets. On 64-bit targets, `0x1_0000_0000` fits into `usize`, so we
        // can return exact lower and upper bounds. On 32-bit targets,
        // `0x1_0000_0000` doesn't fit into `usize`, so we only return an lower
        // bound of `usize::MAX` and don't return an upper bound.
        #[cfg(target_pointer_width = "64")]
        assert_eq!(
            Step::steps_between(&VirtAddr(0), &VirtAddr(0x1_0000_0000)),
            (0x1_0000_0000, Some(0x1_0000_0000))
        );
        #[cfg(not(target_pointer_width = "64"))]
        assert_eq!(
            Step::steps_between(&VirtAddr(0), &VirtAddr(0x1_0000_0000)),
            (usize::MAX, None)
        );
    }

    #[test]
    #[cfg(feature = "step_trait")]
    fn virtaddr_step_overflowing() {
        assert_eq!(
            Step::forward_overflowing(VirtAddr(0x7fff_ffff_ffff), 1),
            (VirtAddr(0xffff_8000_0000_0000), false)
        );
        assert_eq!(
            Step::backward_overflowing(VirtAddr(0xffff_8000_0000_0000), 1),
            (VirtAddr(0x7fff_ffff_ffff), false)
        );
        assert_eq!(
            Step::forward_overflowing(VirtAddr(0), 0),
            (VirtAddr(0), false)
        );

        assert!(Step::forward_overflowing(VirtAddr(0xffff_ffff_ffff_ffff), 1).1);
        assert!(Step::backward_overflowing(VirtAddr(0), 1).1);
    }

    #[test]
    pub fn test_align_up() {
        // align 1
        assert_eq!(align_up(0, 1), 0);
        assert_eq!(align_up(1234, 1), 1234);
        assert_eq!(align_up(0xffff_ffff_ffff_ffff, 1), 0xffff_ffff_ffff_ffff);
        // align 2
        assert_eq!(align_up(0, 2), 0);
        assert_eq!(align_up(1233, 2), 1234);
        assert_eq!(align_up(0xffff_ffff_ffff_fffe, 2), 0xffff_ffff_ffff_fffe);
        // address 0
        assert_eq!(align_up(0, 128), 0);
        assert_eq!(align_up(0, 1), 0);
        assert_eq!(align_up(0, 2), 0);
        assert_eq!(align_up(0, 0x8000_0000_0000_0000), 0);
    }

    #[test]
    fn test_virt_addr_align_up() {
        // Make sure the 47th bit is extended.
        assert_eq!(
            VirtAddr48::new(0x7fff_ffff_ffff).align_up(2u64),
            VirtAddr48::new(0xffff_8000_0000_0000)
        );
    }

    #[test]
    fn test_virt_addr_align_down() {
        // Make sure the 47th bit is extended.
        assert_eq!(
            VirtAddr48::new(0xffff_8000_0000_0000).align_down(1u64 << 48),
            VirtAddr48::new(0)
        );
    }

    #[test]
    #[should_panic]
    fn test_virt_addr_align_up_overflow() {
        VirtAddr48::new(0xffff_ffff_ffff_ffff).align_up(2u64);
    }

    #[test]
    #[should_panic]
    fn test_phys_addr_align_up_overflow() {
        PhysAddr::new(0x000f_ffff_ffff_ffff).align_up(2u64);
    }

    #[test]
    #[cfg(target_pointer_width = "64")]
    fn test_from_ptr_array() {
        let slice = &[1, 2, 3, 4, 5];
        // Make sure that from_ptr(slice) is the address of the first element
        assert_eq!(
            VirtAddr48::from_ptr(slice.as_slice()),
            VirtAddr48::from_ptr(&slice[0])
        );
    }
}

#[cfg(kani)]
mod proofs {
    use super::*;

    // The next two proof harnesses prove the correctness of the `forward`
    // implementation of VirtAddr.

    // This harness proves that our implementation can correctly take 0 or 1
    // step starting from any address.
    #[kani::proof]
    fn forward_base_case() {
        let start = kani::any::<VirtAddr>();
        let start_raw = start.as_u64();

        // Adding 0 to any address should always yield the same address.
        let same = Step::forward(start, 0);
        assert!(start == same);

        // Manually calculate the expected address after stepping once.
        let expected = match start_raw {
            // Adding 1 to addresses in this range don't require gap jumps, so
            // we can just add 1.
            0x0000_0000_0000_0000..=0x0000_7fff_ffff_fffe => Some(start_raw + 1),
            // Adding 1 to this address jumps the gap.
            0x0000_7fff_ffff_ffff => Some(0xffff_8000_0000_0000),
            // The range of non-canonical addresses.
            0x0000_8000_0000_0000..=0xffff_7fff_ffff_ffff => unreachable!(),
            // Adding 1 to addresses in this range don't require gap jumps, so
            // we can just add 1.
            0xffff_8000_0000_0000..=0xffff_ffff_ffff_fffe => Some(start_raw + 1),
            // Adding 1 to this address causes an overflow.
            0xffff_ffff_ffff_ffff => None,
        };
        if let Some(expected) = expected {
            // Verify that `expected` is a valid address.
            assert!(VirtAddr48::try_new(expected).is_ok());
        }
        // Verify `forward_checked`.
        let next = Step::forward_checked(start, 1);
        assert!(next.map(VirtAddr::as_u64) == expected);
    }

    // This harness proves that the result of taking two small steps is the
    // same as taking one combined large step.
    #[kani::proof]
    fn forward_induction_step() {
        let start = kani::any::<VirtAddr>();

        let count1: usize = kani::any();
        let count2: usize = kani::any();
        // If we can take two small steps...
        let Some(next1) = Step::forward_checked(start, count1) else {
            return;
        };
        let Some(next2) = Step::forward_checked(next1, count2) else {
            return;
        };

        // ...then we can also take one combined large step.
        let count_both = count1 + count2;
        let next_both = Step::forward(start, count_both);
        assert!(next2 == next_both);
    }

    // The next two proof harnesses prove the correctness of the `backward`
    // implementation of VirtAddr using the `forward` implementation which
    // we've already proven to be correct.
    // They do this by proving the symmetry between those two functions.

    // This harness proves the correctness of the implementation of `backward`
    // for all inputs for which `forward_checked` succeeds.
    #[kani::proof]
    fn forward_implies_backward() {
        let start = kani::any::<VirtAddr>();
        let count: usize = kani::any();

        // If `forward_checked` succeeds...
        let Some(end) = Step::forward_checked(start, count) else {
            return;
        };

        // ...then `backward` succeeds as well.
        let start2 = Step::backward(end, count);
        assert!(start == start2);
    }

    // This harness proves that for all inputs for which `backward_checked`
    // succeeds, `forward` succeeds as well.
    #[kani::proof]
    fn backward_implies_forward() {
        let end = kani::any::<VirtAddr>();
        let count: usize = kani::any();

        // If `backward_checked` succeeds...
        let Some(start) = Step::backward_checked(end, count) else {
            return;
        };

        // ...then `forward` succeeds as well.
        let end2 = Step::forward(start, count);
        assert!(end == end2);
    }

    // The next two proof harnesses prove the correctness of the
    // `steps_between` implementation of VirtAddr using the `forward`
    // implementation which we've already proven to be correct.
    // They do this by proving the symmetry between those two functions.

    // This harness proves the correctness of the implementation of
    // `steps_between` for all inputs for which `forward_checked` succeeds.
    #[kani::proof]
    fn forward_implies_steps_between() {
        let start = kani::any::<VirtAddr>();
        let count: usize = kani::any();

        // If `forward_checked` succeeds...
        let Some(end) = Step::forward_checked(start, count) else {
            return;
        };

        // ...then `steps_between` succeeds as well.
        assert!(Step::steps_between(&start, &end) == (count, Some(count)));
    }

    // This harness proves that for all inputs for which `steps_between`
    // succeeds, `forward` succeeds as well.
    #[kani::proof]
    fn steps_between_implies_forward() {
        let start = kani::any::<VirtAddr>();
        let end = kani::any::<VirtAddr>();

        // If `steps_between` succeeds...
        let Some(count) = Step::steps_between(&start, &end).1 else {
            return;
        };

        // ...then `forward` succeeds as well.
        assert!(Step::forward(start, count) == end);
    }
}
