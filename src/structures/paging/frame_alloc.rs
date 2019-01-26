//! Traits for abstracting away frame allocation and deallocation.

use structures::paging::{PageSize, PhysFrame};

/// A trait for types that can allocate a frame of memory.
pub trait FrameAllocator<S: PageSize> {
    /// Allocate a frame of the appropriate size and return it if possible.
    fn allocate_frame(&mut self) -> Option<PhysFrame<S>>;
}

/// A trait for types that can deallocate a frame of memory.
pub trait FrameDeallocator<S: PageSize> {
    /// Deallocate the given frame of memory.
    fn deallocate_frame(&mut self, frame: PhysFrame<S>);
}
