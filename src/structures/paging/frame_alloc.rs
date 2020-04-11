//! Traits for abstracting away frame allocation and deallocation.

use crate::structures::paging::{PageSize, PhysFrame, Size4KiB};
use core::ops::{Deref, DerefMut};

/// A trait for types that can allocate a frame of memory.
///
/// This trait is unsafe to implement because the implementer must guarantee that
/// the `allocate_frame` method returns only unique unused frames.
pub unsafe trait FrameAllocator<S: PageSize> {
    /// Allocate a frame of the appropriate size and return it if possible.
    fn allocate_frame(&mut self) -> Option<PhysFrame<S>>;
}

/// A trait for types that can deallocate a frame of memory.
pub trait FrameDeallocator<S: PageSize> {
    /// Deallocate the given unused frame.
    ///
    /// ## Safety
    ///
    /// The caller must ensure that the passed frame is unused.
    unsafe fn deallocate_frame(&mut self, frame: PhysFrame<S>);
}

/// Represents a physical frame that is not used for any mapping.
#[deprecated(note = "This wrapper type is no longer used. Use `PhysFrame` instead.")]
#[derive(Debug)]
pub struct UnusedPhysFrame<S: PageSize = Size4KiB>(PhysFrame<S>);

#[allow(deprecated)]
impl<S: PageSize> UnusedPhysFrame<S> {
    /// Creates a new UnusedPhysFrame from the given frame.
    ///
    /// ## Safety
    ///
    /// This method is unsafe because the caller must guarantee
    /// that the given frame is unused.
    #[inline]
    pub unsafe fn new(frame: PhysFrame<S>) -> Self {
        Self(frame)
    }

    /// Returns the physical frame as `PhysFrame` type.
    #[inline]
    pub fn frame(self) -> PhysFrame<S> {
        self.0
    }
}

#[allow(deprecated)]
impl<S: PageSize> Deref for UnusedPhysFrame<S> {
    type Target = PhysFrame<S>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[allow(deprecated)]
impl<S: PageSize> DerefMut for UnusedPhysFrame<S> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
