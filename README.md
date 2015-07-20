# x86 / amd64 library [![Build Status](https://travis-ci.org/gz/rust-x86.svg)](https://travis-ci.org/gz/rust-x86) [![Crates.io](https://img.shields.io/crates/v/x86.svg)](https://crates.io/crates/x86)

Library to program x86 (amd64) hardware. Contains x86 specific data structure descriptions, data-tables, as well as convenience function to call assembly instructions typically not exposed in higher level languages.

Currently supports
  * I/O registers
  * Control registers
  * MSR registers
  * Segmentation
  * Descriptor-tables (GDT, LDT, IDT)
  * IA32-e page table layout
  * Interrupts
  * Task state
  * Querying CPUID (uses [raw_cpuid](https://github.com/gz/rust-cpuid) library)

This library depends on libcore so it can be used in kernel level code.

## Documentation
 * [API Documentation](http://gz.github.io/rust-x86/x86/)