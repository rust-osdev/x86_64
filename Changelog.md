## Breaking

- Make `Mapper::map_to` and `Mapper::identity_map` unsafe because it is possible to break memory safety by passing invalid arguments.
- Rename `FrameAllocator::alloc` to `allocate_frame` and `FrameDeallocator::dealloc` to `deallocate_frame`.

# 0.3.6

- Add a `SIZE` constant to the `Page` type
- Add two interrupt tests to the `testing` sub-crate
