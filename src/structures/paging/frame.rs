//! Abstractions for default-sized and huge physical memory frames.

use super::page::AddressNotAligned;
use crate::structures::paging::page::{PageSize, Size4KiB};
use crate::PhysAddr;
use core::fmt;
use core::marker::PhantomData;
use core::ops::{Add, AddAssign, Range, RangeInclusive, Sub, SubAssign};

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

/// Helper trait to get the number of frames in the range.
#[allow(clippy::len_without_is_empty)]
pub trait PhysFrameRangeLen {
    /// Returns the number of frames in the range.
    fn len(&self) -> u64;
}

impl<S: PageSize> PhysFrameRangeLen for Range<PhysFrame<S>> {
    #[inline]
    fn len(&self) -> u64 {
        if !self.is_empty() {
            self.end - self.start
        } else {
            0
        }
    }
}

impl<S: PageSize> PhysFrameRangeLen for RangeInclusive<PhysFrame<S>> {
    #[inline]
    fn len(&self) -> u64 {
        if !self.is_empty() {
            *self.end() - *self.start() + 1
        } else {
            0
        }
    }
}

/// Helper trait to get the size in bytes of all frames within the range.
pub trait PhysFrameRangeSize {
    /// Returns the size in bytes of all frames within the range.
    fn size(&self) -> u64;
}

impl<S: PageSize> PhysFrameRangeSize for Range<PhysFrame<S>> {
    #[inline]
    fn size(&self) -> u64 {
        S::SIZE * self.len()
    }
}

impl<S: PageSize> PhysFrameRangeSize for RangeInclusive<PhysFrame<S>> {
    #[inline]
    fn size(&self) -> u64 {
        S::SIZE * self.len()
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

        let range = start..end;
        assert_eq!(range.len(), 50);

        let range_inclusive = start..=end;
        assert_eq!(range_inclusive.len(), 51);
    }
}
