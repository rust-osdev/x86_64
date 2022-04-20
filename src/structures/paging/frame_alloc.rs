//! Traits for abstracting away frame allocation and deallocation.

use crate::structures::paging::{PageSize, PhysFrame};

/// A trait for types that can allocate a frame of memory.
///
/// # Safety
///
/// The implementer of this trait must guarantee that the `allocate_frame`
/// method returns only unique unused frames. Otherwise, undefined behavior
/// may result from two callers modifying or deallocating the same frame.
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
