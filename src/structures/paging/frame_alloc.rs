//! Traits for abstracting away frame allocation and deallocation.

use crate::structures::paging::{PageSize, PhysFrame, Size4KiB};
use core::ops::{Deref, DerefMut};

/// A trait for types that can allocate a frame of memory.
///
/// This trait is unsafe to implement because the implementer must guarantee that
/// the `allocate_frame` method returns only unique unused frames.
pub unsafe trait FrameAllocator<S: PageSize> {
    /// Allocate a frame of the appropriate size and return it if possible.
    fn allocate_frame(&mut self) -> Option<UnusedPhysFrame<S>>;
}

/// A trait for types that can deallocate a frame of memory.
pub trait FrameDeallocator<S: PageSize> {
    /// Deallocate the given frame of memory.
    fn deallocate_frame(&mut self, frame: UnusedPhysFrame<S>);
}

/// Represents a physical frame that is not used for any mapping.
#[derive(Debug)]
pub struct UnusedPhysFrame<S: PageSize = Size4KiB>(PhysFrame<S>);

impl<S: PageSize> UnusedPhysFrame<S> {
    /// Creates a new UnusedPhysFrame from the given frame.
    ///
    /// This method is unsafe because the caller must guarantee
    /// that the given frame is unused.
    pub unsafe fn new(frame: PhysFrame<S>) -> Self {
        Self(frame)
    }

    pub fn frame(self) -> PhysFrame<S> {
        self.0
    }
}

impl<S: PageSize> Deref for UnusedPhysFrame<S> {
    type Target = PhysFrame<S>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<S: PageSize> DerefMut for UnusedPhysFrame<S> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
