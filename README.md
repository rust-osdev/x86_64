# x86_64 library

[![docs.rs](https://img.shields.io/badge/docs.rs-documentation-green.svg)](https://docs.rs/x86_64)

Fork of [gz/rust-x86](https://github.com/gz/rust-x86).

Library to program x86_64 hardware. Contains x86_64 specific data structure descriptions, data-tables, as well as convenience function to call assembly instructions typically not exposed in higher level languages.

Currently supports
  * I/O registers
  * Control registers
  * MSR registers
  * Interrupts
  * Task state
  * Querying CPUID (uses [raw_cpuid](https://github.com/gz/rust-cpuid) library)

This library depends on libcore so it can be used in kernel level code.
