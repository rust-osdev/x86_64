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

        let reg_rax: usize;
        unsafe {
            llvm_asm!("
                mov %rax, $0"
                :"=0"(reg_rax)
            );
        }

        let reg_rbx: usize;
        unsafe {
            llvm_asm!("
                mov %rbx, $0"
                :"=0"(reg_rbx)
            );
        }

        let reg_rcx: usize;
        unsafe {
            llvm_asm!("
                mov %rcx, $0"
                :"=0"(reg_rcx)
            );
        }

        let reg_rdx: usize;
        unsafe {
            llvm_asm!("
                mov %rdx, $0"
                :"=0"(reg_rdx)
            );
        }
        
        let reg_rdi: usize;
        unsafe {
            llvm_asm!("
                mov %rdi, $0"
                :"=0"(reg_rdi)
            );
        }
        
        let reg_rsi: usize;
        unsafe {
            llvm_asm!("
                mov %rsi, $0"
                :"=0"(reg_rsi)
            );
        }

        let reg_rbp: usize;
        unsafe {
            llvm_asm!("
                mov %rbp, $0"
                :"=0"(reg_rbp)
            );
        }

        let reg_rsp: usize;
        unsafe {
            llvm_asm!("
                mov %rsp, $0"
                :"=0"(reg_rsp)
            );
        }
        let reg_r8: usize;
        unsafe {
            llvm_asm!("
                mov %r8, $0"
                :"=0"(reg_r8)
            );
        }
        let reg_r9: usize;
        unsafe {
            llvm_asm!("
                mov %r9, $0"
                :"=0"(reg_r9)
            );
        }
        let reg_r10: usize;
        unsafe {
            llvm_asm!("
                mov %r10, $0"
                :"=0"(reg_r10)
            );
        }
        let reg_r11: usize;
        unsafe {
            llvm_asm!("
                mov %r11, $0"
                :"=0"(reg_r11)
            );
        }
        let reg_r12: usize;
        unsafe {
            llvm_asm!("
                mov %r12, $0"
                :"=0"(reg_r12)
            );
        }
        let reg_r13: usize;
        unsafe {
            llvm_asm!("
                mov %r13, $0"
                :"=0"(reg_r13)
            );
        }
        let reg_r14: usize;
        unsafe {
            llvm_asm!("
                mov %r14, $0"
                :"=0"(reg_r14)
            );
        }
        let reg_r15: usize;
        unsafe {
            llvm_asm!("
                mov %r15, $0"
                :"=0"(reg_r15)
            );
        }
        let reg_rip: usize;
        unsafe {
            llvm_asm!("
                mov %rip, $0"
                :"=0"(reg_rip)
            );
        }
        let reg_rflags: usize;
        unsafe {
            llvm_asm!("
                mov %rflags, $0"
                :"=0"(reg_rflags)
            );
        }
        let reg_cs: usize;
        unsafe {
            llvm_asm!("
                mov %cs, $0"
                :"=0"(reg_cs)
            );
        }
        let reg_fs: usize;
        unsafe {
            llvm_asm!("
                mov %fs, $0"
                :"=0"(reg_fs)
            );
        }
        let reg_gs: usize;
        unsafe {
            llvm_asm!("
                mov %gs, $0"
                :"=0"(reg_gs)
            );
        }

        RegData {
            rax: reg_rax,
            rbx: reg_rbx,
            rcx: reg_rcx,
            rdx: reg_rdx,
            rdi: reg_rdi,
            rsi: reg_rsi,
            rbp: reg_rbp,
            rsp: reg_rsp,
            r8: reg_r8,
            r9: reg_r9,
            r10: reg_r10,
            r11: reg_r11,
            r12: reg_r12,
            r13: reg_r13,
            r14: reg_r14,
            r15: reg_r15,
            rip: reg_rip,
            rflags: reg_rflags,
            cs: reg_cs,
            fs: reg_fs,
            gs: reg_gs,
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
