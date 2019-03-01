## Breaking

- The `random` module is now a submodule of the `instructions` module.
- The `structures::paging` module was split into several submodules:
    - The `NotGiantPageSize`, `PageRange`, and `PageRangeInclusive` types were moved to a new `page` submodule.
    - The `PhysFrameRange` and `PhysFrameRangeInclusive` types were moved to a new `frame` submodule.
    - The `FrameError` and `PageTableEntry` types were moved to a new `page_table` submodule.
    - The `MapperFlush`, `MapToError`, `UnmapError`, and `FlagUpdateError` types were moved to a new `mapper` submodule.
- The `structures::paging` module received the following changes:
    - The `Mapper::translate_page` function now returns a `Result` with a new `TranslateError` error type.
    - The `NotRecursivelyMapped` error type was removed.
- The `instructions::int3` function was moved into the `instructions::interrupts` module.
- Removed some old deprecated functions.

## Other

- Added a new `structures::paging::MappedPageTable` type that implements the `Mapper` trait.
- Added a `software_interrupt` macro to invoke arbitrary `int x` instructions.
- Renamed the `ExceptionStackFrame` type to `InterruptStackFrame`.

# 0.4.2

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
