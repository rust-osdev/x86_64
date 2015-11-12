#![feature(lang_items, start, no_std, core, libc)]
#![no_std]

extern crate libc;
extern crate x86;

use core::prelude::*;
use core::mem;

#[start]
fn start(_argc: isize, _argv: *const *const u8) -> isize {
    0
}

#[lang = "eh_personality"] extern fn eh_personality() {}
#[lang = "panic_fmt"] fn panic_fmt() -> ! { loop {} }
