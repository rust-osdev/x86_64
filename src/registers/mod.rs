//! Access to various system and model specific registers.

pub mod control;
pub mod model_specific;
pub mod rflags;

/// Gets the current instruction pointer. Note that this is only approximate as it requires a few
/// instructions to execute.
#[cfg(feature = "inline_asm")]
#[inline(always)]
pub fn read_rip() -> u64 {
    let rip: u64;
    unsafe {
        llvm_asm!(
            "lea (%rip), $0"
            : "=r"(rip) ::: "volatile"
        );
    }
    rip
}

pub struct RegData {
    rax: usize,
    rbx: usize,
    rcx: usize,
    rdx: usize,
    rdi: usize,
    rsi: usize,
    rbp: usize,
    rsp: usize,
    r8: usize,
    r9: usize,
    r10: usize,
    r11: usize,
    r12: usize,
    r13: usize,
    r14: usize,
    r15: usize,
    rip: usize,
    rflags: usize,
    cs: usize,
    fs: usize,
    gs: usize,
}

impl RegData {
    pub fn backup() -> RegData {

        let rax: usize;
        unsafe {
            llvm_asm!("
                mov %rax, $0"
                :"=0"(rax)
            );
        }

        let rbx: usize;
        unsafe {
            llvm_asm!("
                mov %rbx, $0"
                :"=0"(rbx)
            );
        }

        let rcx: usize;
        unsafe {
            llvm_asm!("
                mov %rcx, $0"
                :"=0"(rcx)
            );
        }

        let rdx: usize;
        unsafe {
            llvm_asm!("
                mov %rdx, $0"
                :"=0"(rdx)
            );
        }

        let rbp: usize;
        unsafe {
            llvm_asm!("
                mov %rbp, $0"
                :"=0"(rbp)
            );
        }

        let rsp: usize;
        unsafe {
            llvm_asm!("
                mov %rsp, $0"
                :"=0"(rsp)
            );usize
        }
        let r8: usize;
        unsafe {
            llvm_asm!("
                mov %r8, $0"
                :"=0"(r8)
            );
        }
        let r9: usize;
        unsafe {
            llvm_asm!("
                mov %r9, $0"
                :"=0"(r9)
            );
        }
        let r10: usize;
        unsafe {
            llvm_asm!("
                mov %r10, $0"
                :"=0"(r10)
            );
        }
        let r11: usize;
        unsafe {
            llvm_asm!("
                mov %r11, $0"
                :"=0"(r11)
            );
        }
        let r12: usize;
        unsafe {
            llvm_asm!("
                mov %r12, $0"
                :"=0"(r12)
            );
        }
        let r13: usize;
        unsafe {
            llvm_asm!("
                mov %r13, $0"
                :"=0"(r13)
            );
        }
        let r14: usize;
        unsafe {
            llvm_asm!("
                mov %r14, $0"
                :"=0"(r14)
            );
        }
        let r15: usize;
        unsafe {
            llvm_asm!("
                mov %r15, $0"
                :"=0"(r15)
            );
        }
        let rip: usize;
        unsafe {
            llvm_asm!("
                mov %rip, $0"
                :"=0"(rip)
            );
        }
        let rflags: usize;
        unsafe {
            llvm_asm!("
                mov %rflags, $0"
                :"=0"(rflags)
            );
        }
        let cs: usize;
        unsafe {
            llvm_asm!("
                mov %cs, $0"
                :"=0"(cs)
            );
        }
        let fs: usize;
        unsafe {
            llvm_asm!("
                mov %fs, $0"
                :"=0"(fs)
            );
        }
        let gs: usize;
        unsafe {
            llvm_asm!("
                mov %gs, $0"
                :"=0"(gs)
            );
        }

        RegData {
            rax: rax,
            rbx: rbx,
            rcx: rcx,
            rdx: rdx,
            rdi: rdi,
            rsi: rsi,
            rbp: rbp,
            rsp: rsp,
            r8: r8,
            r9: r9,
            r10: r10,
            r11: r11,
            r12: r12,
            r13: r13,
            r14: r14,
            r15: r15,
            rip: rip,
            rflags: rflags,
            cs: cs,
            fs: fs,
            gs: gs,
        }
    }

    pub fn restore(data: RegData) {

        unsafe {
            llvm_asm!("
                mov $0, %rax"
                :
                : "{rax}"(data.rax)
            );
        }
        unsafe {
            llvm_asm!("
                mov $0, %rbx"
                :
                : "{rbx}"(data.rbx)
            );
        }
        unsafe {
            llvm_asm!("
                mov $0, %rcx"
                :
                : "{rcx}"(data.rcx)
            );
        }
        unsafe {
            llvm_asm!("
                mov $0, %rdx"
                :
                : "{rdx}"(data.rdx)
            );
        }
        unsafe {
            llvm_asm!("
                mov $0, %rdi"
                :
                : "{rdi}"(data.rdi)
            );
        }
        unsafe {
            llvm_asm!("
                mov $0, %rsi"
                :
                : "{rsi}"(data.rsi)
            );
        }
        unsafe {
            llvm_asm!("
                mov $0, %rbp"
                :
                : "{rbp}"(data.rbp)
            );
        }
        unsafe {
            llvm_asm!("
                mov $0, %rsp"
                :
                : "{rsp}"(data.rsp)
            );
        }
        unsafe {
            llvm_asm!("
                mov $0, %r8"
                :
                : "{r8}"(data.r8)
            );
        }
        unsafe {
            llvm_asm!("
                mov $0, %r9"
                :
                : "{r9}"(data.r9)
            );
        }
        unsafe {
            llvm_asm!("
                mov $0, %r10"
                :
                : "{r10}"(data.r10)
            );
        }
        unsafe {
            llvm_asm!("
                mov $0, %r11"
                :
                : "{r11}"(data.r11)
            );
        }
        unsafe {
            llvm_asm!("
                mov $0, %r12"
                :
                : "{r12}"(data.r12)
            );
        }
        unsafe {
            llvm_asm!("
                mov $0, %r13"
                :
                : "{r13}"(data.r13)
            );
        }
        unsafe {
            llvm_asm!("
                mov $0, %r14"
                :
                : "{r14}"(data.r14)
            );
        }
        unsafe {
            llvm_asm!("
                mov $0, %r15"
                :
                : "{r15}"(data.r15)
            );
        }
        unsafe {
            llvm_asm!("
                mov $0, %rip"
                :
                : "{rip}"(data.rip)
            );
        }
        unsafe {
            llvm_asm!("
                mov $0, %rflags"
                :
                : "{rflags}"(data.rflags)
            );
        }
        unsafe {
            llvm_asm!("
                mov $0, %cs"
                :
                : "{cs}"(data.cs)
            );
        }
        unsafe {
            llvm_asm!("
                mov $0, %fs"
                :
                : "{fs}"(data.fs)
            );
        }
        unsafe {
            llvm_asm!("
                mov $0, %gs"
                :
                : "{gs}"(data.gs)
            );
        }
    }
}
