//! Processor state stored in the RFLAGS register.

pub use x86_64_types::registers::RFlags;

impl super::RegReader for RFlags {
    fn read() -> Self {
        let r: Self;
        unsafe { asm!("pushfq; popq $0" : "=r"(r) :: "memory") };
        r
    }
}

impl super::RegWriter for RFlags {
    unsafe fn write(self) {
        asm!("pushq $0; popfq" :: "r"(self) : "memory" "flags");
    }
}
