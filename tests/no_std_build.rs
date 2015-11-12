// Verify that we can be linked against an appliction which only uses
// libcore, which is common in kernel space.

#![feature(no_std, lang_items)]
#![no_std]

extern crate x86;

fn main() {
}

// We want to supply these definitions ourselves, and not have them
// accidentally pulled in via the x86 crate.
#[lang = "eh_personality"]
extern "C" fn eh_personality() {
}

#[lang = "panic_fmt"]
extern "C" fn panic_fmt(
    args: ::core::fmt::Arguments, file: &str, line: usize)
    -> !
{
    loop {}
}
