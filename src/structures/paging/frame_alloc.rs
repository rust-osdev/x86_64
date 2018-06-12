//! Traits for abstracting away frame allocation and deallocation.

use structures::paging::{PageSize, PhysFrame};

/// A trait for types that can allocate a frame of memory.
pub trait FrameAllocator {
    /// Allocate a frame of the appropriate size and return it if possible.
    fn alloc<S: PageSize>(&mut self) -> Option<PhysFrame<S>>;
}

/// A trait for types that can deallocate a frame of memory.
pub trait FrameDeallocator {
    /// Deallocate the given frame of memory.
    fn dealloc<S: PageSize>(&mut self, frame: PhysFrame<S>);
}
