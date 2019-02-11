- Add `RdRand::get_u{16, 32, 64}` methods
- Deprecate `RdRand::get` because it does not check for failure
- Make `RdRand` Copy

# 0.4.1

- Add support for the RdRand instruction (random number generation)

# 0.4.0

## Breaking

- Make `Mapper::map_to` and `Mapper::identity_map` unsafe because it is possible to break memory safety by passing invalid arguments.
- Rename `FrameAllocator::alloc` to `allocate_frame` and `FrameDeallocator::dealloc` to `deallocate_frame`.
- Remove `From<os_bootinfo::FrameRange>` implementation for `PhysFrameRange`
  - The `os_bootinfo` crate is no longer used by the `bootloader` crate.
  - It is not possible to provide an implementation for all `os_bootinfo` versions.

## Other

- Update to 2018 edition

# 0.3.6

- Add a `SIZE` constant to the `Page` type
- Add two interrupt tests to the `testing` sub-crate
