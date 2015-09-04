#![allow(non_upper_case_globals)]

pub use self::x86_shared::*;

mod x86_shared;

#[inline(always)]
pub fn get_flags() -> Flags {
	unsafe {
		let r: usize;
		asm!("pushfq; pop $0" : "=r"(r) ::: "intel");
		Flags::from_bits_truncate(r)
	}
}

#[inline(always)]
pub unsafe fn set_flags(val: Flags) {
	asm!("push $0; popfq" :: "r"(val.bits()) : "flags" : "volatile", "intel");
}
