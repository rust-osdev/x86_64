#![allow(non_upper_case_globals)]

pub mod irq;
pub mod task;

pub use shared::Flags;

#[inline(always)]
pub fn get_flags() -> Flags {
    unsafe {
        let r: usize;
        asm!("pushfd; pop $0" : "=r"(r) ::: "intel");
        Flags::from_bits_truncate(r)
    }
}

#[inline(always)]
pub unsafe fn set_flags(val: Flags) {
    asm!("push $0; popfd" :: "r"(val.bits()) : "flags" : "volatile", "intel");
}

#[inline(always)]
pub unsafe fn stack_jmp(stack: *mut (), ip: *const ()) -> ! {
    asm!("mov esp, $0; jmp $1" :: "rg"(stack), "r"(ip) :: "volatile", "intel");
    loop { }
}
