# x86_64 library

[![Crates.io](https://img.shields.io/crates/v/x86_64)](https://crates.io/crates/x86_64)
[![Build Status](https://github.com/rust-osdev/x86_64/workflows/Build/badge.svg)](https://github.com/rust-osdev/x86_64/actions?query=workflow%3ABuild) [![docs.rs](https://img.shields.io/badge/docs.rs-documentation-green.svg)](https://docs.rs/x86_64)

Support for x86_64 specific instructions (e.g. TLB flush), registers (e.g. control registers), and structures (e.g. page tables).

## Crate Feature Flags

* `nightly`: Enables features only available on nightly Rust; enabled by default.
* `instructions`: Enabled by default, turns on x86\_64 specific instructions, and dependent features. Only available for x86\_64 targets.

## Minimum Supported Rust Version (MSRV)

If no features are enabled (`--no-default-features`), Rust 1.57.0 is required.

If only the `instructions` feature is enabled (`--no-default-features --features instructions`), Rust 1.59.0 is required.

If the `nightly` feature or any of its sub-features is enabled, a recent nightly is required.

## Other OS development crates

This crate does not attempt to handle every facet of OS development. Other
useful crates in this space include:
  - [`raw-cpuid`](https://crates.io/crates/raw-cpuid): safe wrappers around the
  [`cpuid` instruction](https://en.wikipedia.org/wiki/CPUID)
    - Provides parsed versions of the CPUID data, rather than just raw binary values.
    - Support for AMD and Intel specific values.
    - Works on x86 and x86_64 systems, in both user and kernel mode.
  - [`uefi`](https://crates.io/crates/uefi): abstractions for
  [UEFI](https://en.wikipedia.org/wiki/Unified_Extensible_Firmware_Interface)
  (the successor to BIOS)
    - Provides UEFI tables, functions, and types.
    - Useful for writing UEFI applications, or calling UEFI functions from your OS.
    - Works on a variety of modern platforms, not just x86_64.
  - [`volatile`](https://crates.io/crates/volatile): interface to
  [`read_volatile`](https://doc.rust-lang.org/std/ptr/fn.read_volatile.html) and
  [`write_volatile`](https://doc.rust-lang.org/std/ptr/fn.write_volatile.html)
    - Makes it easier to program [MMIO](https://en.wikipedia.org/wiki/Memory-mapped_I/O) interfaces and devices.
    - Works on any Rust target.