//! Abstractions for default-sized and huge physical memory frames.

use super::page::AddressNotAligned;
use crate::structures::paging::page::{PageSize, Size4KiB};
use crate::PhysAddr;
use core::convert::TryFrom;
use core::fmt;
use core::marker::PhantomData;
use core::ops::{Add, AddAssign, Sub, SubAssign};

/// A physical memory frame.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(C)]
pub struct PhysFrame<S: PageSize = Size4KiB> {
    // TODO: Make private when our minimum supported stable Rust version is 1.61
    pub(crate) start_address: PhysAddr,
    size: PhantomData<S>,
}

impl<S: PageSize> PhysFrame<S> {
    /// Returns the frame that starts at the given virtual address.
    ///
    /// Returns an error if the address is not correctly aligned (i.e. is not a valid frame start).
    #[inline]
    #[rustversion::attr(since(1.61), const)]
    pub fn from_start_address(address: PhysAddr) -> Result<Self, AddressNotAligned> {
        if !address.is_aligned_u64(S::SIZE) {
            return Err(AddressNotAligned);
        }

        // SAFETY: correct address alignment is checked above
        Ok(unsafe { PhysFrame::from_start_address_unchecked(address) })
    }

    /// Returns the frame that starts at the given virtual address.
    ///
    /// ## Safety
    ///
    /// The address must be correctly aligned.
    #[inline]
    #[rustversion::attr(since(1.61), const)]
    pub unsafe fn from_start_address_unchecked(start_address: PhysAddr) -> Self {
        PhysFrame {
            start_address,
            size: PhantomData,
        }
    }

    /// Returns the frame by a physical frame number.
    ///
    /// ```
    /// use x86_64::{PhysAddr, structures::paging::{PhysFrame, Size4KiB}};
    ///
    /// assert_eq!(PhysFrame::<Size4KiB>::from_pfn(0x123), PhysFrame::<Size4KiB>::containing_address(PhysAddr::new(0x123000)));
    /// ```
    ///
    /// # Panics
    ///
    /// This function will panic if the resulting address is not valid.
    #[inline]
    #[rustversion::attr(
        since(1.61),
        dep_const_fn::const_fn(cfg(not(feature = "memory_encryption")))
    )]
    pub fn from_pfn(pfn: u64) -> Self {
        match Self::try_from_pfn(pfn) {
            Ok(frame) => frame,
            Err(_) => panic!("PFNs must not have any bits in the range 40 to 64 set"),
        }
    }

    /// Returns the frame by a physical frame number.
    ///
    /// ```
    /// use x86_64::{PhysAddr, structures::paging::{PhysFrame, Size4KiB}};
    ///
    /// assert_eq!(PhysFrame::<Size4KiB>::try_from_pfn(0x123), Ok(PhysFrame::<Size4KiB>::containing_address(PhysAddr::new(0x123000))));
    /// ```
    ///
    /// # Error
    ///
    /// This function will return an error if the resulting address is not valid.
    #[inline]
    #[rustversion::attr(
        since(1.61),
        dep_const_fn::const_fn(cfg(not(feature = "memory_encryption")))
    )]
    pub fn try_from_pfn(pfn: u64) -> Result<Self, PfnNotValid> {
        let addr_raw = if let Some(addr_raw) = pfn.checked_mul(S::SIZE) {
            addr_raw
        } else {
            return Err(PfnNotValid(pfn));
        };
        let addr = if let Ok(addr) = PhysAddr::try_new(addr_raw) {
            addr
        } else {
            return Err(PfnNotValid(pfn));
        };
        Ok(PhysFrame {
            start_address: addr,
            size: PhantomData,
        })
    }

    /// Returns the frame by a physical frame number.
    ///
    /// # Safety
    ///
    /// The resulting address must be valid.
    #[inline]
    #[rustversion::attr(since(1.61), const)]
    pub unsafe fn from_pfn_unchecked(pfn: u64) -> Self {
        PhysFrame {
            start_address: unsafe { PhysAddr::new_unsafe(pfn * S::SIZE) },
            size: PhantomData,
        }
    }

    /// Returns the frame that contains the given physical address.
    #[inline]
    #[rustversion::attr(since(1.61), const)]
    pub fn containing_address(address: PhysAddr) -> Self {
        PhysFrame {
            start_address: address.align_down_u64(S::SIZE),
            size: PhantomData,
        }
    }

    /// Returns the start address of the frame.
    #[inline]
    #[rustversion::attr(since(1.61), const)]
    pub fn start_address(self) -> PhysAddr {
        self.start_address
    }

    /// Returns the size the frame (4KB, 2MB or 1GB).
    #[inline]
    #[rustversion::attr(since(1.61), const)]
    pub fn size(self) -> u64 {
        S::SIZE
    }

    /// Returns the PFN of the current frame.
    ///
    /// The PFN is defined to be the address divided by the page size.
    ///
    /// ```
    /// use x86_64::{PhysAddr, structures::paging::{PhysFrame, Size1GiB, Size2MiB, Size4KiB}};
    ///
    /// assert_eq!(PhysFrame::<Size4KiB>::containing_address(PhysAddr::new(0x123000)).pfn(), 0x123);
    ///
    /// // Note that this means that the PFN for the same address will be
    /// // different for different page sizes.
    /// assert_eq!(PhysFrame::<Size4KiB>::containing_address(PhysAddr::new(0xC000_0000)).pfn(), 0xC0000);
    /// assert_eq!(PhysFrame::<Size2MiB>::containing_address(PhysAddr::new(0xC000_0000)).pfn(), 0x600);
    /// assert_eq!(PhysFrame::<Size1GiB>::containing_address(PhysAddr::new(0xC000_0000)).pfn(), 0x3);
    /// ```
    #[inline]
    #[rustversion::attr(since(1.61), const)]
    pub fn pfn(self) -> u64 {
        self.start_address.as_u64() / S::SIZE
    }

    /// Returns a range of frames, exclusive `end`.
    #[inline]
    #[rustversion::attr(since(1.61), const)]
    pub fn range(start: PhysFrame<S>, end: PhysFrame<S>) -> PhysFrameRange<S> {
        PhysFrameRange { start, end }
    }

    /// Returns a range of frames, inclusive `end`.
    #[inline]
    #[rustversion::attr(since(1.61), const)]
    pub fn range_inclusive(start: PhysFrame<S>, end: PhysFrame<S>) -> PhysFrameRangeInclusive<S> {
        PhysFrameRangeInclusive { start, end }
    }
}

impl<S: PageSize> fmt::Debug for PhysFrame<S> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_fmt(format_args!(
            "PhysFrame[{}]({:#x})",
            S::DEBUG_STR,
            self.start_address().as_u64()
        ))
    }
}

impl<S: PageSize> Add<u64> for PhysFrame<S> {
    type Output = Self;
    #[inline]
    fn add(self, rhs: u64) -> Self::Output {
        PhysFrame::containing_address(self.start_address() + rhs * S::SIZE)
    }
}

impl<S: PageSize> AddAssign<u64> for PhysFrame<S> {
    #[inline]
    fn add_assign(&mut self, rhs: u64) {
        *self = *self + rhs;
    }
}

impl<S: PageSize> Sub<u64> for PhysFrame<S> {
    type Output = Self;
    #[inline]
    fn sub(self, rhs: u64) -> Self::Output {
        PhysFrame::containing_address(self.start_address() - rhs * S::SIZE)
    }
}

impl<S: PageSize> SubAssign<u64> for PhysFrame<S> {
    #[inline]
    fn sub_assign(&mut self, rhs: u64) {
        *self = *self - rhs;
    }
}

impl<S: PageSize> Sub<PhysFrame<S>> for PhysFrame<S> {
    type Output = u64;
    #[inline]
    fn sub(self, rhs: PhysFrame<S>) -> Self::Output {
        (self.start_address - rhs.start_address) / S::SIZE
    }
}

/// An range of physical memory frames, exclusive the upper bound.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
#[repr(C)]
pub struct PhysFrameRange<S: PageSize = Size4KiB> {
    /// The start of the range, inclusive.
    pub start: PhysFrame<S>,
    /// The end of the range, exclusive.
    pub end: PhysFrame<S>,
}

impl<S: PageSize> PhysFrameRange<S> {
    /// Returns whether the range contains no frames.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.start >= self.end
    }

    /// Returns the number of frames in the range.
    #[inline]
    pub fn len(&self) -> u64 {
        if !self.is_empty() {
            self.end - self.start
        } else {
            0
        }
    }

    /// Returns the size in bytes of all frames within the range.
    #[inline]
    pub fn size(&self) -> u64 {
        S::SIZE * self.len()
    }
}

impl<S: PageSize> Iterator for PhysFrameRange<S> {
    type Item = PhysFrame<S>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.start < self.end {
            let frame = self.start;
            self.start += 1;
            Some(frame)
        } else {
            None
        }
    }

    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        if self.is_empty() {
            return None;
        }

        // Convert to `u64`. If the value doesn't fit just use `u64::MAX`.
        // `self.len()` is guaranteed to be smaller than the real value and
        // `u64::MAX` anyway, so it doesn't make a difference.
        let n = u64::try_from(n).unwrap_or(u64::MAX);

        // Handling `n >= self.len()` is a bit more complicated because we
        // can't just add `n` to `self.start` (it might overflow). Handle this
        // by doing two steps, `self.len()-1` and `1`. This should return
        // `None`.
        if n >= self.len() {
            self.nth(self.len() as usize - 1)?;
            return self.next();
        }

        self.start += n;
        self.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.len();
        usize::try_from(len)
            .map(|len| (len, Some(len)))
            .unwrap_or((usize::MAX, None))
    }
}

impl<S: PageSize> DoubleEndedIterator for PhysFrameRange<S> {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.start < self.end {
            self.end -= 1;
            Some(self.end)
        } else {
            None
        }
    }

    fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
        if self.is_empty() {
            return None;
        }

        // Convert to `u64`. If the value doesn't fit just use `u64::MAX`.
        // `self.len()` is guaranteed to be smaller than the real value and
        // `u64::MAX` anyway, so it doesn't make a difference.
        let n = u64::try_from(n).unwrap_or(u64::MAX);

        // Handling `n >= self.len()` is a bit more complicated because we
        // can't just subtract `n` to `self.end` (it might overflow). Handle
        // this by doing two steps, `self.len()-1` and `1`. This should return
        // `None`.
        if n >= self.len() {
            self.nth_back(self.len() as usize - 1)?;
            return self.next_back();
        }

        self.end -= n;
        self.next_back()
    }
}

impl<S: PageSize> fmt::Debug for PhysFrameRange<S> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("PhysFrameRange")
            .field("start", &self.start)
            .field("end", &self.end)
            .finish()
    }
}

/// A passed `u64` was not a valid physical address.
///
/// This means that bits 40 to 64 were not all null.
///
/// Contains the invalid PFN.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct PfnNotValid(pub u64);

// Implementation of display
impl fmt::Display for PfnNotValid {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_tuple("PhysAddrNotValid")
            .field(&format_args!("{:#x}", self.0))
            .finish()
    }
}

/// An range of physical memory frames, inclusive the upper bound.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
#[repr(C)]
pub struct PhysFrameRangeInclusive<S: PageSize = Size4KiB> {
    /// The start of the range, inclusive.
    pub start: PhysFrame<S>,
    /// The start of the range, inclusive.
    pub end: PhysFrame<S>,
}

impl<S: PageSize> PhysFrameRangeInclusive<S> {
    /// Returns whether the range contains no frames.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.start > self.end
    }

    /// Returns the number of frames in the range.
    #[inline]
    pub fn len(&self) -> u64 {
        if !self.is_empty() {
            self.end - self.start + 1
        } else {
            0
        }
    }

    /// Returns the size in bytes of all frames within the range.
    #[inline]
    pub fn size(&self) -> u64 {
        S::SIZE * self.len()
    }
}

impl<S: PageSize> Iterator for PhysFrameRangeInclusive<S> {
    type Item = PhysFrame<S>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.start <= self.end {
            let frame = self.start;

            // If the end of the inclusive range is the maximum page possible for size S,
            // incrementing start until it is greater than the end will cause an integer overflow.
            // So instead, in that case we decrement end rather than incrementing start.
            let max_frame_addr = PhysAddr::new_truncate(u64::MAX) - (S::SIZE - 1);
            if self.start.start_address() < max_frame_addr {
                self.start += 1;
            } else {
                self.end -= 1;
            }
            Some(frame)
        } else {
            None
        }
    }

    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        if self.is_empty() {
            return None;
        }

        // Convert to `u64`. If the value doesn't fit just use `u64::MAX`.
        // `self.len()` is guaranteed to be smaller than the real value and
        // `u64::MAX` anyway, so it doesn't make a difference.
        let n = u64::try_from(n).unwrap_or(u64::MAX);

        // Handling `n >= self.len()` is a bit more complicated because we
        // can't just add `n` to `self.start` (it might overflow). Handle this
        // by doing two steps, `self.len()-1` and `1`. This should return
        // `None`.
        if n >= self.len() {
            self.nth(self.len() as usize - 1)?;
            return self.next();
        }

        self.start += n;
        self.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.len();
        usize::try_from(len)
            .map(|len| (len, Some(len)))
            .unwrap_or((usize::MAX, None))
    }
}

impl<S: PageSize> DoubleEndedIterator for PhysFrameRangeInclusive<S> {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.start <= self.end {
            let frame = self.end;

            // If the start of the inclusive range is 0, decrementing end until
            // it is smaller than the start will cause an integer underflow.
            // So instead, in that case we increment start rather than decrementing end.
            if self.end.start_address().as_u64() != 0 {
                self.end -= 1;
            } else {
                self.start += 1;
            }
            Some(frame)
        } else {
            None
        }
    }

    fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
        if self.is_empty() {
            return None;
        }

        // Convert to `u64`. If the value doesn't fit just use `u64::MAX`.
        // `self.len()` is guaranteed to be smaller than the real value and
        // `u64::MAX` anyway, so it doesn't make a difference.
        let n = u64::try_from(n).unwrap_or(u64::MAX);

        // Handling `n >= self.len()` is a bit more complicated because we
        // can't just subtract `n` to `self.end` (it might overflow). Handle
        // this by doing two steps, `self.len()-1` and `1`. This should return
        // `None`.
        if n >= self.len() {
            self.nth_back(self.len() as usize - 1)?;
            return self.next_back();
        }

        self.end -= n;
        self.next_back()
    }
}

impl<S: PageSize> fmt::Debug for PhysFrameRangeInclusive<S> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("PhysFrameRangeInclusive")
            .field("start", &self.start)
            .field("end", &self.end)
            .finish()
    }
}

#[cfg(kani)]
impl<S: PageSize> kani::Arbitrary for PhysFrame<S> {
    fn any() -> Self {
        Self::containing_address(kani::any())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    pub fn test_frame_range_len() {
        let start_addr = PhysAddr::new(0xdead_beaf);
        let start = PhysFrame::<Size4KiB>::containing_address(start_addr);
        let end = start + 50;

        let range = PhysFrameRange { start, end };
        assert_eq!(range.len(), 50);

        let range_inclusive = PhysFrameRangeInclusive { start, end };
        assert_eq!(range_inclusive.len(), 51);
    }
}

#[cfg(kani)]
mod proofs {
    use super::*;

    #[kani::proof]
    fn phys_frame_range_next() {
        let start = kani::any::<PhysFrame>();
        let end = kani::any::<PhysFrame>();
        let mut range = PhysFrame::range(start, end);

        // Test that calling `next` twice works.
        let difference = end
            .start_address()
            .as_u64()
            .checked_sub(start.start_address().as_u64());
        let expected_result = difference.is_some_and(|d| d >= 0x1000).then(|| start);
        assert_eq!(range.next(), expected_result);
        let expected_result = difference.is_some_and(|d| d >= 0x2000).then(|| start + 1);
        assert_eq!(range.next(), expected_result);
    }

    #[kani::proof]
    fn phys_frame_range_inclusive_next() {
        let start = kani::any::<PhysFrame>();
        let end = kani::any::<PhysFrame>();
        let mut range = PhysFrame::range_inclusive(start, end);

        // Test that calling `next` twice works.
        let difference = end
            .start_address()
            .as_u64()
            .checked_sub(start.start_address().as_u64());
        let expected_result = difference.is_some().then(|| start);
        assert_eq!(range.next(), expected_result);
        let expected_result = difference.is_some_and(|d| d >= 0x1000).then(|| start + 1);
        assert_eq!(range.next(), expected_result);
    }

    #[kani::proof]
    fn phys_frame_range_next_back() {
        let start = kani::any::<PhysFrame>();
        let end = kani::any::<PhysFrame>();
        let mut range = PhysFrame::range(start, end);

        // Test that calling `next_back` twice works.
        let difference = end
            .start_address()
            .as_u64()
            .checked_sub(start.start_address().as_u64());
        let expected_result = difference.is_some_and(|d| d >= 0x1000).then(|| end - 1);
        assert_eq!(range.next_back(), expected_result);
        let expected_result = difference.is_some_and(|d| d >= 0x2000).then(|| end - 2);
        assert_eq!(range.next_back(), expected_result);
    }

    #[kani::proof]
    fn phys_frame_range_inclusive_next_back() {
        let start = kani::any::<PhysFrame>();
        let end = kani::any::<PhysFrame>();
        let mut range = PhysFrame::range_inclusive(start, end);

        // Test that calling `next_back` twice works.
        let difference = end
            .start_address()
            .as_u64()
            .checked_sub(start.start_address().as_u64());
        let expected_result = difference.is_some().then(|| end);
        assert_eq!(range.next_back(), expected_result);
        let expected_result = difference.is_some_and(|d| d >= 0x1000).then(|| end - 1);
        assert_eq!(range.next_back(), expected_result);
    }

    #[kani::proof]
    #[kani::unwind(1)]
    fn phys_frame_range_nth_0() {
        let start = kani::any::<PhysFrame>();
        let end = kani::any::<PhysFrame>();
        let mut range = PhysFrame::range(start, end);
        let mut range2 = PhysFrame::range(start, end);

        // Test that nth(0) behaves like next().
        assert_eq!(range.next(), range2.nth(0));
        assert_eq!(range.next(), range2.nth(0));
    }

    #[kani::proof]
    #[kani::unwind(1)]
    fn phys_frame_range_nth() {
        let start = kani::any::<PhysFrame>();
        let end = kani::any::<PhysFrame>();
        let m = kani::any::<usize>();
        let n = kani::any::<usize>();
        let sum = m.saturating_add(n).saturating_add(1);
        let mut range = PhysFrame::range(start, end);
        let mut range2 = PhysFrame::range(start, end);

        // Test that doing steps of size m and n is equivalent to a single step
        // of size m+n+1.
        range.nth(m);
        assert_eq!(range.nth(n), range2.nth(sum));
    }

    #[kani::proof]
    #[kani::unwind(1)]
    fn phys_frame_range_inclusive_nth_0() {
        let start = kani::any::<PhysFrame>();
        let end = kani::any::<PhysFrame>();
        let mut range = PhysFrame::range_inclusive(start, end);
        let mut range2 = PhysFrame::range_inclusive(start, end);

        // Test that nth(0) behaves like next().
        assert_eq!(range.next(), range2.nth(0));
        assert_eq!(range.next(), range2.nth(0));
    }

    #[kani::proof]
    #[kani::unwind(1)]
    fn phys_frame_range_inclusive_nth() {
        let start = kani::any::<PhysFrame>();
        let end = kani::any::<PhysFrame>();
        let m = kani::any::<usize>();
        let n = kani::any::<usize>();
        let sum = m.saturating_add(n).saturating_add(1);
        let mut range = PhysFrame::range_inclusive(start, end);
        let mut range2 = PhysFrame::range_inclusive(start, end);

        // Test that doing steps of size m and n is equivalent to a single step
        // of size m+n+1.
        range.nth(m);
        assert_eq!(range.nth(n), range2.nth(sum));
    }

    #[kani::proof]
    #[kani::unwind(1)]
    fn phys_frame_range_nth_back_0() {
        let start = kani::any::<PhysFrame>();
        let end = kani::any::<PhysFrame>();
        let mut range = PhysFrame::range(start, end);
        let mut range2 = PhysFrame::range(start, end);

        // Test that nth_back(0) behaves like next_back().
        assert_eq!(range.next_back(), range2.nth_back(0));
        assert_eq!(range.next_back(), range2.nth_back(0));
    }

    #[kani::proof]
    #[kani::unwind(1)]
    fn phys_frame_range_nth_back() {
        let start = kani::any::<PhysFrame>();
        let end = kani::any::<PhysFrame>();
        let m = kani::any::<usize>();
        let n = kani::any::<usize>();
        let sum = m.saturating_add(n).saturating_add(1);
        let mut range = PhysFrame::range(start, end);
        let mut range2 = PhysFrame::range(start, end);

        // Test that doing steps of size m and n is equivalent to a single step
        // of size m+n+1.
        range.nth_back(m);
        assert_eq!(range.nth_back(n), range2.nth_back(sum));
    }

    #[kani::proof]
    #[kani::unwind(1)]
    fn phys_frame_range_inclusive_nth_back_0() {
        let start = kani::any::<PhysFrame>();
        let end = kani::any::<PhysFrame>();
        let mut range = PhysFrame::range_inclusive(start, end);
        let mut range2 = PhysFrame::range_inclusive(start, end);

        // Test that nth_back(0) behaves like next_back().
        assert_eq!(range.next_back(), range2.nth_back(0));
        assert_eq!(range.next_back(), range2.nth_back(0));
    }

    #[kani::proof]
    #[kani::unwind(1)]
    fn phys_frame_range_inclusive_nth_back() {
        let start = kani::any::<PhysFrame>();
        let end = kani::any::<PhysFrame>();
        let m = kani::any::<usize>();
        let n = kani::any::<usize>();
        let sum = m.saturating_add(n).saturating_add(1);
        let mut range = PhysFrame::range_inclusive(start, end);
        let mut range2 = PhysFrame::range_inclusive(start, end);

        // Test that doing steps of size m and n is equivalent to a single step
        // of size m+n+1.
        range.nth_back(m);
        assert_eq!(range.nth_back(n), range2.nth_back(sum));
    }
}
