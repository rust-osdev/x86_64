# x86 / amd64 library

[![Build Status](https://travis-ci.org/gz/rust-x86.svg)](https://travis-ci.org/gz/rust-x86)

This is a low level library that provides only the most basic wrapper functions
for assembly instructions, defines etc. for x86 hardware.

Currently supports
  * I/O registers
  * Control registers
  * MSR registers
  * Segmentation
  * Descriptor-tables (GDT, LDT, IDT)
  * IA32-e page table layout
  * Interrupts
  * Task state

This only depends on libcore so it can be used in kernel level code.
