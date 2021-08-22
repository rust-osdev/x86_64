//! Abstractions for default-sized and huge physical memory frames.

use super::page::AddressNotAligned;
use crate::ops::{CheckedAdd, CheckedSub};
use crate::structures::paging::page::{PageSize, Size4KiB};
use crate::PhysAddr;
use core::fmt;
use core::marker::PhantomData;
use core::ops::{Add, AddAssign, Sub, SubAssign};

/// A physical memory frame.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(C)]
pub struct PhysFrame<S: PageSize = Size4KiB> {
    pub(crate) start_address: PhysAddr, // TODO: remove when start_address() is const
    size: PhantomData<S>,
}

impl<S: PageSize> PhysFrame<S> {
    /// Returns the frame that starts at the given physical address.
    ///
    /// Returns an error if the address is not correctly aligned (i.e. is not a valid frame start).
    #[inline]
    pub fn from_start_address(address: PhysAddr) -> Result<Self, AddressNotAligned> {
        if !address.is_aligned(S::SIZE) {
            return Err(AddressNotAligned);
        }

        // SAFETY: correct address alignment is checked above
        Ok(unsafe { PhysFrame::from_start_address_unchecked(address) })
    }

    const_fn! {
        /// Returns the frame that starts at the given physical address.
        ///
        /// ## Safety
        ///
        /// The address must be correctly aligned.
        #[inline]
        pub unsafe fn from_start_address_unchecked(start_address: PhysAddr) -> Self {
            PhysFrame {
                start_address,
                size: PhantomData,
            }
        }
    }

    /// Returns the frame that contains the given physical address.
    #[inline]
    pub fn containing_address(address: PhysAddr) -> Self {
        PhysFrame {
            start_address: address.align_down(S::SIZE),
            size: PhantomData,
        }
    }

    const_fn! {
        /// Returns the start address of the frame.
        #[inline]
        pub fn start_address(self) -> PhysAddr {
            self.start_address
        }
    }

    const_fn! {
        /// Returns the size the frame (4KB, 2MB or 1GB).
        #[inline]
        pub fn size(self) -> u64 {
            S::SIZE
        }
    }

    const_fn! {
        /// Returns a range of frames, exclusive `end`.
        #[inline]
        pub fn range(start: PhysFrame<S>, end: PhysFrame<S>) -> PhysFrameRange<S> {
            PhysFrameRange { start, end }
        }
    }

    const_fn! {
        /// Returns a range of frames, inclusive `end`.
        #[inline]
        pub fn range_inclusive(start: PhysFrame<S>, end: PhysFrame<S>) -> PhysFrameRangeInclusive<S> {
            PhysFrameRangeInclusive { start, end }
        }
    }
}

impl<S: PageSize> fmt::Debug for PhysFrame<S> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_fmt(format_args!(
            "PhysFrame[{}]({:#x})",
            S::SIZE_AS_DEBUG_STR,
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

impl<S: PageSize> CheckedAdd<u64> for PhysFrame<S> {
    type Output = Self;
    #[inline]
    fn checked_add(self, rhs: u64) -> Option<Self::Output> {
        let phys_addr_rhs = rhs
            .checked_mul(S::SIZE)
            .and_then(|addr| PhysAddr::try_new(addr).ok())?;

        self.start_address()
            .checked_add(phys_addr_rhs.as_u64())
            .and_then(|addr| PhysFrame::from_start_address(addr).ok())
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

impl<S: PageSize> CheckedSub<u64> for PhysFrame<S> {
    type Output = Self;
    #[inline]
    fn checked_sub(self, rhs: u64) -> Option<Self::Output> {
        let phys_addr_rhs = rhs
            .checked_mul(S::SIZE)
            .and_then(|addr| PhysAddr::try_new(addr).ok())?;

        self.start_address()
            .checked_sub(phys_addr_rhs.as_u64())
            .and_then(|addr| PhysFrame::from_start_address(addr).ok())
    }
}

impl<S: PageSize> Sub<PhysFrame<S>> for PhysFrame<S> {
    type Output = u64;
    #[inline]
    fn sub(self, rhs: PhysFrame<S>) -> Self::Output {
        (self.start_address - rhs.start_address) / S::SIZE
    }
}

impl<S: PageSize> CheckedSub<PhysFrame<S>> for PhysFrame<S> {
    type Output = u64;
    #[inline]
    fn checked_sub(self, rhs: PhysFrame<S>) -> Option<Self::Output> {
        self.start_address()
            .checked_sub(rhs.start_address())
            .map(PhysAddr::new)
            .and_then(|addr| PhysFrame::<S>::from_start_address(addr).ok())
            .map(|phys_frame| phys_frame.start_address().as_u64())
    }
}

/// An range of physical memory frames, exclusive the upper bound.
#[derive(Clone, Copy, PartialEq, Eq)]
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
}

impl<S: PageSize> fmt::Debug for PhysFrameRange<S> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("PhysFrameRange")
            .field("start", &self.start)
            .field("end", &self.end)
            .finish()
    }
}

/// An range of physical memory frames, inclusive the upper bound.
#[derive(Clone, Copy, PartialEq, Eq)]
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
}

impl<S: PageSize> Iterator for PhysFrameRangeInclusive<S> {
    type Item = PhysFrame<S>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.start <= self.end {
            let frame = self.start;
            self.start += 1;
            Some(frame)
        } else {
            None
        }
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
