//! Physical and virtual addresses manipulation

use core::convert::TryFrom;
use core::fmt;
#[cfg(feature = "step_trait")]
use core::iter::Step;
use core::ops::{Add, AddAssign, Sub, SubAssign};
#[cfg(feature = "memory_encryption")]
use core::sync::atomic::Ordering;

#[cfg(feature = "memory_encryption")]
use crate::structures::mem_encrypt::ENC_BIT_MASK;
use crate::structures::paging::page_table::PageTableLevel;
use crate::structures::paging::{PageOffset, PageTableIndex};

use bit_field::BitField;
use dep_const_fn::const_fn;

const ADDRESS_SPACE_SIZE: u64 = 0x1_0000_0000_0000;

/// A canonical 64-bit virtual memory address.
///
/// This is a wrapper type around an `u64`, so it is always 8 bytes, even when compiled
/// on non 64-bit systems. The
/// [`TryFrom`](https://doc.rust-lang.org/std/convert/trait.TryFrom.html) trait can be used for performing conversions
/// between `u64` and `usize`.
///
/// On `x86_64`, only the 48 lower bits of a virtual address can be used. The top 16 bits need
/// to be copies of bit 47, i.e. the most significant bit. Addresses that fulfil this criterion
/// are called “canonical”. This type guarantees that it always represents a canonical address.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct VirtAddr(u64);

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
/// This means that bits 48 to 64 are not
/// a valid sign extension and are not null either. So automatic sign extension would have
/// overwritten possibly meaningful bits. This likely indicates a bug, for example an invalid
/// address calculation.
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

impl VirtAddr {
    /// Creates a new canonical virtual address.
    ///
    /// The provided address should already be canonical. If you want to check
    /// whether an address is canonical, use [`try_new`](Self::try_new).
    ///
    /// ## Panics
    ///
    /// This function panics if the bits in the range 48 to 64 are invalid
    /// (i.e. are not a proper sign extension of bit 47).
    #[inline]
    pub const fn new(addr: u64) -> VirtAddr {
        // TODO: Replace with .ok().expect(msg) when that works on stable.
        match Self::try_new(addr) {
            Ok(v) => v,
            Err(_) => panic!("virtual address must be sign extended in bits 48 to 64"),
        }
    }

    /// Tries to create a new canonical virtual address.
    ///
    /// This function checks wether the given address is canonical
    /// and returns an error otherwise. An address is canonical
    /// if bits 48 to 64 are a correct sign
    /// extension (i.e. copies of bit 47).
    #[inline]
    pub const fn try_new(addr: u64) -> Result<VirtAddr, VirtAddrNotValid> {
        let v = Self::new_truncate(addr);
        if v.0 == addr {
            Ok(v)
        } else {
            Err(VirtAddrNotValid(addr))
        }
    }

    /// Creates a new canonical virtual address, throwing out bits 48..64.
    ///
    /// This function performs sign extension of bit 47 to make the address
    /// canonical, overwriting bits 48 to 64. If you want to check whether an
    /// address is canonical, use [`new`](Self::new) or [`try_new`](Self::try_new).
    #[inline]
    pub const fn new_truncate(addr: u64) -> VirtAddr {
        // By doing the right shift as a signed operation (on a i64), it will
        // sign extend the value, repeating the leftmost bit.
        VirtAddr(((addr << 16) as i64 >> 16) as u64)
    }

    /// Creates a new virtual address, without any checks.
    ///
    /// ## Safety
    ///
    /// You must make sure bits 48..64 are equal to bit 47. This is not checked.
    #[inline]
    pub const unsafe fn new_unsafe(addr: u64) -> VirtAddr {
        VirtAddr(addr)
    }

    /// Creates a virtual address that points to `0`.
    #[inline]
    pub const fn zero() -> VirtAddr {
        VirtAddr(0)
    }

    /// Converts the address to an `u64`.
    #[inline]
    pub const fn as_u64(self) -> u64 {
        self.0
    }

    /// Creates a virtual address from the given pointer
    #[cfg(target_pointer_width = "64")]
    #[inline]
    pub fn from_ptr<T: ?Sized>(ptr: *const T) -> Self {
        Self::new(ptr as *const () as u64)
    }

    /// Converts the address to a raw pointer.
    #[cfg(target_pointer_width = "64")]
    #[inline]
    pub const fn as_ptr<T>(self) -> *const T {
        self.as_u64() as *const T
    }

    /// Converts the address to a mutable raw pointer.
    #[cfg(target_pointer_width = "64")]
    #[inline]
    pub const fn as_mut_ptr<T>(self) -> *mut T {
        self.as_ptr::<T>() as *mut T
    }

    /// Convenience method for checking if a virtual address is null.
    #[inline]
    pub const fn is_null(self) -> bool {
        self.0 == 0
    }

    /// Aligns the virtual address upwards to the given alignment.
    ///
    /// See the `align_up` function for more information.
    ///
    /// # Panics
    ///
    /// This function panics if the resulting address is higher than
    /// `0xffff_ffff_ffff_ffff`.
    #[inline]
    pub fn align_up<U>(self, align: U) -> Self
    where
        U: Into<u64>,
    {
        VirtAddr::new_truncate(align_up(self.0, align.into()))
    }

    /// Aligns the virtual address downwards to the given alignment.
    ///
    /// See the `align_down` function for more information.
    #[inline]
    pub fn align_down<U>(self, align: U) -> Self
    where
        U: Into<u64>,
    {
        self.align_down_u64(align.into())
    }

    /// Aligns the virtual address downwards to the given alignment.
    ///
    /// See the `align_down` function for more information.
    #[inline]
    pub(crate) const fn align_down_u64(self, align: u64) -> Self {
        VirtAddr::new_truncate(align_down(self.0, align))
    }

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
    pub(crate) const fn is_aligned_u64(self, align: u64) -> bool {
        self.align_down_u64(align).as_u64() == self.as_u64()
    }

    /// Returns the 12-bit page offset of this virtual address.
    #[inline]
    pub const fn page_offset(self) -> PageOffset {
        PageOffset::new_truncate(self.0 as u16)
    }

    /// Returns the 9-bit level 1 page table index.
    #[inline]
    pub const fn p1_index(self) -> PageTableIndex {
        PageTableIndex::new_truncate((self.0 >> 12) as u16)
    }

    /// Returns the 9-bit level 2 page table index.
    #[inline]
    pub const fn p2_index(self) -> PageTableIndex {
        PageTableIndex::new_truncate((self.0 >> 12 >> 9) as u16)
    }

    /// Returns the 9-bit level 3 page table index.
    #[inline]
    pub const fn p3_index(self) -> PageTableIndex {
        PageTableIndex::new_truncate((self.0 >> 12 >> 9 >> 9) as u16)
    }

    /// Returns the 9-bit level 4 page table index.
    #[inline]
    pub const fn p4_index(self) -> PageTableIndex {
        PageTableIndex::new_truncate((self.0 >> 12 >> 9 >> 9 >> 9) as u16)
    }

    /// Returns the 9-bit level page table index.
    #[inline]
    pub const fn page_table_index(self, level: PageTableLevel) -> PageTableIndex {
        PageTableIndex::new_truncate((self.0 >> 12 >> ((level as u8 - 1) * 9)) as u16)
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
    #[cfg(any(feature = "instructions", feature = "step_trait"))]
    pub(crate) fn steps_between_u64(start: &Self, end: &Self) -> Option<u64> {
        let mut steps = end.0.checked_sub(start.0)?;

        // Mask away extra bits that appear while jumping the gap.
        steps &= 0xffff_ffff_ffff;

        Some(steps)
    }

    // FIXME: Move this into the `Step` impl, once `Step` is stabilized.
    #[inline]
    pub(crate) fn forward_checked_impl(start: Self, count: usize) -> Option<Self> {
        Self::forward_checked_u64(start, u64::try_from(count).ok()?)
    }

    /// An implementation of forward_checked that takes u64 instead of usize.
    #[inline]
    pub(crate) fn forward_checked_u64(start: Self, count: u64) -> Option<Self> {
        if count > ADDRESS_SPACE_SIZE {
            return None;
        }

        let mut addr = start.0.checked_add(count)?;

        match addr.get_bits(47..) {
            0x1 => {
                // Jump the gap by sign extending the 47th bit.
                addr.set_bits(47.., 0x1ffff);
            }
            0x2 => {
                // Address overflow
                return None;
            }
            _ => {}
        }

        Some(unsafe { Self::new_unsafe(addr) })
    }

    /// An implementation of backward_checked that takes u64 instead of usize.
    #[cfg(feature = "step_trait")]
    #[inline]
    pub(crate) fn backward_checked_u64(start: Self, count: u64) -> Option<Self> {
        if count > ADDRESS_SPACE_SIZE {
            return None;
        }

        let mut addr = start.0.checked_sub(count)?;

        match addr.get_bits(47..) {
            0x1fffe => {
                // Jump the gap by sign extending the 47th bit.
                addr.set_bits(47.., 0);
            }
            0x1fffd => {
                // Address underflow
                return None;
            }
            _ => {}
        }

        Some(unsafe { Self::new_unsafe(addr) })
    }
}

impl fmt::Debug for VirtAddr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_tuple("VirtAddr")
            .field(&format_args!("{:#x}", self.0))
            .finish()
    }
}

impl fmt::Binary for VirtAddr {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Binary::fmt(&self.0, f)
    }
}

impl fmt::LowerHex for VirtAddr {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::LowerHex::fmt(&self.0, f)
    }
}

impl fmt::Octal for VirtAddr {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Octal::fmt(&self.0, f)
    }
}

impl fmt::UpperHex for VirtAddr {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::UpperHex::fmt(&self.0, f)
    }
}

impl fmt::Pointer for VirtAddr {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Pointer::fmt(&(self.0 as *const ()), f)
    }
}

impl Add<u64> for VirtAddr {
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
        VirtAddr::try_new(
            self.0
                .checked_add(rhs)
                .expect("attempt to add with overflow"),
        )
        .expect("attempt to add resulted in non-canonical virtual address")
    }
}

impl AddAssign<u64> for VirtAddr {
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

impl Sub<u64> for VirtAddr {
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
        VirtAddr::try_new(
            self.0
                .checked_sub(rhs)
                .expect("attempt to subtract with overflow"),
        )
        .expect("attempt to subtract resulted in non-canonical virtual address")
    }
}

impl SubAssign<u64> for VirtAddr {
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

impl Sub<VirtAddr> for VirtAddr {
    type Output = u64;

    /// Returns the difference between two addresses.
    ///
    /// # Panics
    ///
    /// This function will panic on overflow.
    #[inline]
    fn sub(self, rhs: VirtAddr) -> Self::Output {
        self.as_u64()
            .checked_sub(rhs.as_u64())
            .expect("attempt to subtract with overflow")
    }
}

#[cfg(feature = "step_trait")]
impl Step for VirtAddr {
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
    // If the `memory_encryption` feature has been enabled and an encryption bit has been
    // configured, this also panics if the encryption bit is manually set in the address.
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

    #[test]
    #[should_panic]
    pub fn add_overflow_virtaddr() {
        let _ = VirtAddr::new(0xffff_ffff_ffff_ffff) + 1;
    }

    #[test]
    #[should_panic]
    pub fn add_overflow_physaddr() {
        let _ = PhysAddr::new(0x000f_ffff_ffff_ffff) + 0xffff_0000_0000_0000;
    }

    #[test]
    #[should_panic]
    pub fn sub_underflow_virtaddr() {
        let _ = VirtAddr::new(0) - 1;
    }

    #[test]
    #[should_panic]
    pub fn sub_overflow_physaddr() {
        let _ = PhysAddr::new(0) - 1;
    }

    #[test]
    pub fn virtaddr_new_truncate() {
        assert_eq!(VirtAddr::new_truncate(0), VirtAddr(0));
        assert_eq!(VirtAddr::new_truncate(1 << 47), VirtAddr(0xfffff << 47));
        assert_eq!(VirtAddr::new_truncate(123), VirtAddr(123));
        assert_eq!(VirtAddr::new_truncate(123 << 47), VirtAddr(0xfffff << 47));
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
            VirtAddr::new(0x7fff_ffff_ffff).align_up(2u64),
            VirtAddr::new(0xffff_8000_0000_0000)
        );
    }

    #[test]
    fn test_virt_addr_align_down() {
        // Make sure the 47th bit is extended.
        assert_eq!(
            VirtAddr::new(0xffff_8000_0000_0000).align_down(1u64 << 48),
            VirtAddr::new(0)
        );
    }

    #[test]
    #[should_panic]
    fn test_virt_addr_align_up_overflow() {
        VirtAddr::new(0xffff_ffff_ffff_ffff).align_up(2u64);
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
            VirtAddr::from_ptr(slice.as_slice()),
            VirtAddr::from_ptr(&slice[0])
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
        let start_raw: u64 = kani::any();
        let Ok(start) = VirtAddr::try_new(start_raw) else {
            return;
        };

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
            assert!(VirtAddr::try_new(expected).is_ok());
        }
        // Verify `forward_checked`.
        let next = Step::forward_checked(start, 1);
        assert!(next.map(VirtAddr::as_u64) == expected);
    }

    // This harness proves that the result of taking two small steps is the
    // same as taking one combined large step.
    #[kani::proof]
    fn forward_induction_step() {
        let start_raw: u64 = kani::any();
        let Ok(start) = VirtAddr::try_new(start_raw) else {
            return;
        };

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
        let start_raw: u64 = kani::any();
        let Ok(start) = VirtAddr::try_new(start_raw) else {
            return;
        };
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
        let end_raw: u64 = kani::any();
        let Ok(end) = VirtAddr::try_new(end_raw) else {
            return;
        };
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
        let start: u64 = kani::any();
        let Ok(start) = VirtAddr::try_new(start) else {
            return;
        };
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
        let start: u64 = kani::any();
        let Ok(start) = VirtAddr::try_new(start) else {
            return;
        };
        let end: u64 = kani::any();
        let Ok(end) = VirtAddr::try_new(end) else {
            return;
        };

        // If `steps_between` succeeds...
        let Some(count) = Step::steps_between(&start, &end).1 else {
            return;
        };

        // ...then `forward` succeeds as well.
        assert!(Step::forward(start, count) == end);
    }
}
