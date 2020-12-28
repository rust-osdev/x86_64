# Unreleased

# 0.12.3 – 2020-10-31

- Use `asm!` instead of perma-unstable `llvm_asm!` macro ([#165](https://github.com/rust-osdev/x86_64/pull/165))
- Make `GlobalDescriptorTable::add_entry` a const fn ([#191](https://github.com/rust-osdev/x86_64/pull/191))
- Rename `enable_interrupts_and_hlt` to `enable_and_hlt` ([#206](https://github.com/rust-osdev/x86_64/pull/206))
- Provide functions for accessing the underlying L4 table for mapper types ([#184](https://github.com/rust-osdev/x86_64/pull/184))
- Remove Trait constraint for `Port::new()` ([#188](https://github.com/rust-osdev/x86_64/pull/188))

# 0.12.2 – 2020-09-29

- Add additional `DescriptorFlags` and aliases compatible with `syscall`/`sysenter` ([#181](https://github.com/rust-osdev/x86_64/pull/181))
- Fix (another) build error on latest nightly ([#186](https://github.com/rust-osdev/x86_64/pull/186))

# 0.12.1 – 2020-09-24

- Fix build error on latest nightly ([#182](https://github.com/rust-osdev/x86_64/pull/182))

# 0.12.0 – 2020-09-23

- **Breaking**: Decouple instructions into a separate feature flag ([#179](https://github.com/rust-osdev/x86_64/pull/179))
  - Gates the `instructions` module by a new `instructions` feature (enabled by default).
  - Rename the `stable` feature to `external_asm`
  - `PageTable::new` is no longer a `const fn` on stable (i.e. without the `nightly` feature)

# 0.11.8 – 2020-09-23

- Add `VirtAddr::is_null` ([#180](https://github.com/rust-osdev/x86_64/pull/180))

# 0.11.7 – 2020-09-11

- Fix const_item_mutation warnings added in latest Rust nightly ([#178](https://github.com/rust-osdev/x86_64/pull/178))

# 0.11.6 – 2020-09-11 (yanked)

- (accidental empty release)

# 0.11.5 – 2020-09-03

- Don't rely on promotion of `PageTableEntry::new` inside a `const fn` ([#175](https://github.com/rust-osdev/x86_64/pull/175))

# 0.11.4 – 2020-09-01

- Add a function for the `nop` instruction ([#174](https://github.com/rust-osdev/x86_64/pull/174))

# ~~0.11.3 – 2020-09-01~~

- (accidental release, yanked)

# 0.11.2 – 2020-08-13

- Add rdfsbase, rdgsbase, wrfsbase, wrgsbase ([#172](https://github.com/rust-osdev/x86_64/pull/172))

# 0.11.1

- Export `PhysAddrNotValid` and `VirtAddrNotValid` error types ([#163](https://github.com/rust-osdev/x86_64/pull/163))

# 0.11.0

- **Breaking**: Handle parent table flags in Mapper methods ([#114](https://github.com/rust-osdev/x86_64/pull/114))

# 0.10.3

- Fix: Inclusive ranges is_empty() comparison ([#156](https://github.com/rust-osdev/x86_64/pull/156))

# 0.10.2

- **Nightly Breakage**: Use `llvm_asm!` instead of deprecated `asm!` macro ([#151](https://github.com/rust-osdev/x86_64/pull/151))
- Return the correct RPL from GDT::add_entry() ([#153](https://github.com/rust-osdev/x86_64/pull/153))

# 0.10.1

- Add InterruptDescriptorTable::load_unsafe ([#137](https://github.com/rust-osdev/x86_64/pull/137))

# 0.10.0

- **Breaking**: Make `map_to` and `update_flags` unsafe ([#135](https://github.com/rust-osdev/x86_64/pull/135))
- **Breaking**: Make `FrameDeallocator::deallocate_frame` unsafe ([#146](https://github.com/rust-osdev/x86_64/pull/146))
- **Breaking**: Don't pass small trivially copyable types by reference ([#147](https://github.com/rust-osdev/x86_64/pull/147))
- Various improvements to VirtAddr and PhysAddr ([#141](https://github.com/rust-osdev/x86_64/pull/141))
  - Among other things, this renamed the `VirtAddr::new_unchecked` function to `new_truncate`.
- Add `const_fn!{}` macro to make functions const without duplication ([#144](https://github.com/rust-osdev/x86_64/pull/144))
  - Also makes some more functions `const`.
- Add `{PhysFrame,Page}::from_start_address_unchecked` ([#142](https://github.com/rust-osdev/x86_64/pull/142))
- Use `#[inline]` everywhere ([#145](https://github.com/rust-osdev/x86_64/pull/145))
- In `VirtAddr::new_truncate`, use shift instead of mul and div ([#143](https://github.com/rust-osdev/x86_64/pull/143))
- Use `Self::new()` in `InterruptDescriptorTable::reset()` ([#148](https://github.com/rust-osdev/x86_64/pull/148))

# 0.9.6

- Add an enable_interrupts_and_hlt function that executes `sti; hlt` ([#138](https://github.com/rust-osdev/x86_64/pull/138))
- Fix some clippy warnings ([#130](https://github.com/rust-osdev/x86_64/pull/130))
- Resolve remaining clippy warnings and add clippy job to CI ([#132](https://github.com/rust-osdev/x86_64/pull/132))

# 0.9.5

- Add `#[inline]` attribute to small functions ([#129](https://github.com/rust-osdev/x86_64/pull/129))

# 0.9.4

- asm: add target_env = "musl" to pickup the underscore asm names ([#128](https://github.com/rust-osdev/x86_64/pull/128))

# 0.9.3

- Enable usage with non-nightly rust ([#127](https://github.com/rust-osdev/x86_64/pull/127))

# 0.9.2

- Remove the `cast` dependency ([#124](https://github.com/rust-osdev/x86_64/pull/124))

# 0.9.1

- Improve PageTableIndex and PageOffset ([#122](https://github.com/rust-osdev/x86_64/pull/122))

# 0.9.0

- **Breaking:** Return the UnusedPhysFrame on MapToError::PageAlreadyMapped ([#118](https://github.com/rust-osdev/x86_64/pull/118))
- Add User Mode registers ([#119](https://github.com/rust-osdev/x86_64/pull/119))

# 0.8.3

- Allow immediate port version of in/out instructions ([#115](https://github.com/rust-osdev/x86_64/pull/115))
- Make more functions const ([#116](https://github.com/rust-osdev/x86_64/pull/116))

# 0.8.2

- Add support for cr4 control register ([#111](https://github.com/rust-osdev/x86_64/pull/111))

# 0.8.1

- Fix: Add required reexport for new UnusedPhysFrame type ([#110](https://github.com/rust-osdev/x86_64/pull/110))

# 0.8.0

- **Breaking:** Replace `ux` dependency with custom wrapper structs ([#91](https://github.com/rust-osdev/x86_64/pull/91))
- **Breaking:** Add new UnsafePhysFrame type and use it in Mapper::map_to ([#89](https://github.com/rust-osdev/x86_64/pull/89))
- **Breaking:** Rename divide_by_zero field of interrupt descriptor table to divide_error ([#108](https://github.com/rust-osdev/x86_64/pull/108))
- **Breaking:** Introduce new diverging handler functions for double faults and machine check exceptions ([#109](https://github.com/rust-osdev/x86_64/pull/109))
- _Possibly Breaking:_ Make Mapper trait object safe by adding `Self: Sized` bounds on generic functions ([#84](https://github.com/rust-osdev/x86_64/pull/84))


# 0.7.7

- Add `slice` and `slice_mut` methods to IDT ([#95](https://github.com/rust-osdev/x86_64/pull/95))

# 0.7.6

- Use repr C to suppress not-ffi-safe when used with extern handler functions ([#94](https://github.com/rust-osdev/x86_64/pull/94))

# 0.7.5

- Add FsBase and GsBase register support ([#87](https://github.com/rust-osdev/x86_64/pull/87))

# 0.7.4

- Remove raw-cpuid dependency and use rdrand intrinsics ([#85](https://github.com/rust-osdev/x86_64/pull/85))
- Update integration tests to use new testing framework ([#86](https://github.com/rust-osdev/x86_64/pull/86))

# 0.7.3

- Add a new `OffsetPageTable` mapper type ([#83](https://github.com/rust-osdev/x86_64/pull/83))

# 0.7.2

- Add `instructions::bochs_breakpoint` and `registers::read_rip` functions ([#79](https://github.com/rust-osdev/x86_64/pull/79))
- Mark all single instruction functions as `#[inline]` ([#79](https://github.com/rust-osdev/x86_64/pull/79))
- Update GDT docs, add user_data_segment function and WRITABLE flag ([#78](https://github.com/rust-osdev/x86_64/pull/78))
- Reexport MappedPageTable on non-x86_64 platforms too ([#82](https://github.com/rust-osdev/x86_64/pull/82))

# 0.7.1

- Add ring-3 flag to GDT descriptor ([#77](https://github.com/rust-osdev/x86_64/pull/77))

# 0.7.0

- **Breaking**: `Port::read` and `PortReadOnly::read` now take `&mut self` instead of `&self` ([#76](https://github.com/rust-osdev/x86_64/pull/76)).

# 0.6.0

- **Breaking**: Make the `FrameAllocator` unsafe to implement. This way, we can force the implementer to guarantee that all frame allocators are valid. See [#69](https://github.com/rust-osdev/x86_64/issues/69) for more information.

# 0.5.5

- Use [`cast`](https://docs.rs/cast/0.2.2/cast/) crate instead of less general `usize_conversions` crate.

# 0.5.4

- Update dependencies to latest versions (fix [#67](https://github.com/rust-osdev/x86_64/issues/67))

# 0.5.3

- Add `PortReadOnly` and `PortWriteOnly` types in `instructions::port` module ([#66](https://github.com/rust-osdev/x86_64/pull/66)).

# 0.5.2

- Update documentation of `MappedPageTable`: Require that passed `level_4_table` is valid.

# 0.5.1

- Add `PageTable::{iter, iter_mut}` functions to iterate over page table entries.

# 0.5.0

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
- Made modifications of the interrupt stack frame unsafe by introducing a new wrapper type and an unsafe `as_mut` method.

## Other

- Added a new `structures::paging::MapperAllSizes` trait with generic translation methods and implement it for `MappedPageTable` and `RecursivePageTable`.
- Added a new `structures::paging::MappedPageTable` type that implements the `Mapper` and `MapperAllSizes` traits.
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
