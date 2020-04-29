//! Access to various system and model specific registers.

pub mod control;
pub mod model_specific;
pub mod rflags;

/// Gets the current instruction pointer. Note that this is only approximate as it requires a few
/// instructions to execute.
#[cfg(feature = "llvm_asm")]
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

/// Struct to store data from all registers in, in event of context switch
#[derive(Debug)]
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
    /// Copy data from all registers, including stack and instruction pointers, into RegData instance
    #[cfg(feature = "llvm_asm")]
    pub fn backup() -> RegData {

        let reg_rax: usize;
        unsafe {
            llvm_asm!("
                mov (%rax), $0"
                :"=i"(reg_rax) ::: "volatile"
            );
        }

        let reg_rbx: usize;
        unsafe {
            llvm_asm!("
                mov (%rbx), $0"
                :"=i"(reg_rbx) ::: "volatile"
            );
        }

        let reg_rcx: usize;
        unsafe {
            llvm_asm!("
                mov (%rcx), $0"
                :"=i"(reg_rcx) ::: "volatile"
            );
        }

        let reg_rdx: usize;
        unsafe {
            llvm_asm!("
                mov (%rdx), $0"
                :"=i"(reg_rdx) ::: "volatile"
            );
        }

        let reg_rdi: usize;
        unsafe {
            llvm_asm!("
                mov (%rdi), $0"
                :"=i"(reg_rdi) ::: "volatile"
            );
        }

        let reg_rsi: usize;
        unsafe {
            llvm_asm!("
                mov (%rsi), $0"
                :"=i"(reg_rsi) ::: "volatile"
            );
        }

        let reg_rbp: usize;
        unsafe {
            llvm_asm!("
                mov (%rbp), $0"
                :"=i"(reg_rbp) ::: "volatile"
            );
        }

        let reg_rsp: usize;
        unsafe {
            llvm_asm!("
                mov (%rsp), $0"
                :"=i"(reg_rsp) ::: "volatile"
            );
        }

        let reg_r8: usize;
        unsafe {
            llvm_asm!("
                mov (%r8), $0"
                :"=i"(reg_r8) ::: "volatile"
            );
        }

        let reg_r9: usize;
        unsafe {
            llvm_asm!("
                mov (%r9), $0"
                :"=i"(reg_r9) ::: "volatile"
            );
        }

        let reg_r10: usize;
        unsafe {
            llvm_asm!("
                mov (%r10), $0"
                :"=i"(reg_r10) ::: "volatile"
            );
        }

        let reg_r11: usize;
        unsafe {
            llvm_asm!("
                mov (%r11), $0"
                :"=i"(reg_r11) ::: "volatile"
            );
        }

        let reg_r12: usize;
        unsafe {
            llvm_asm!("
                mov (%r12), $0"
                :"=i"(reg_r12) ::: "volatile"
            );
        }

        let reg_r13: usize;
        unsafe {
            llvm_asm!("
                mov (%r13), $0"
                :"=i"(reg_r13) ::: "volatile"
            );
        }

        let reg_r14: usize;
        unsafe {
            llvm_asm!("
                mov (%r14), $0"
                :"=i"(reg_r14) ::: "volatile"
            );
        }

        let reg_r15: usize;
        unsafe {
            llvm_asm!("
                mov (%r15), $0"
                :"=i"(reg_r15) ::: "volatile"
            );
        }

        let reg_rip: usize;
        unsafe {
            llvm_asm!("
                mov (%rip), $0"
                :"=i"(reg_rip) ::: "volatile"
            );
        }

        let reg_rflags: usize;
        unsafe {
            llvm_asm!("
                mov (%rflags), $0"
                :"=i"(reg_rflags) ::: "volatile"
            );
        }

        let reg_cs: usize;
        unsafe {
            llvm_asm!("
                mov (%cs), $0"
                :"=i"(reg_cs) ::: "volatile"
            );
        }

        let reg_fs: usize;
        unsafe {
            llvm_asm!("
                mov (%fs), $0"
                :"=i"(reg_fs) ::: "volatile"
            );
        }

        let reg_gs: usize;
        unsafe {
            llvm_asm!("
                mov (%gs), $0"
                :"=i"(reg_gs) ::: "volatile"
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

    /// Copy data from RegData instance provided as method argument into all registers, including stack and instruction pointers
    #[cfg(feature = "llvm_asm")]
    pub fn restore(data: RegData) {

        unsafe {
            llvm_asm!("
                mov $0, (%rax)"
                :
                : "{rax}"(data.rax) ::: "volatile"
            );
        }
        unsafe {
            llvm_asm!("
                mov $0, (%rbx)"
                :
                : "{rbx}"(data.rbx) ::: "volatile"
            );
        }
        unsafe {
            llvm_asm!("
                mov $0, (%rcx)"
                :
                : "{rcx}"(data.rcx) ::: "volatile"
            );
        }
        unsafe {
            llvm_asm!("
                mov $0, (%rdx)"
                :
                : "{rdx}"(data.rdx) ::: "volatile"
            );
        }
        unsafe {
            llvm_asm!("
                mov $0, (%rdi)"
                :
                : "{rdi}"(data.rdi) ::: "volatile"
            );
        }
        unsafe {
            llvm_asm!("
                mov $0, (%rsi)"
                :
                : "{rsi}"(data.rsi) ::: "volatile"
            );
        }
        unsafe {
            llvm_asm!("
                mov $0, (%rbp)"
                :
                : "{rbp}"(data.rbp) ::: "volatile"
            );
        }
        unsafe {
            llvm_asm!("
                mov $0, (%rsp)"
                :
                : "{rsp}"(data.rsp) ::: "volatile"
            );
        }
        unsafe {
            llvm_asm!("
                mov $0, (%r8)"
                :
                : "{r8}"(data.r8) ::: "volatile"
            );
        }
        unsafe {
            llvm_asm!("
                mov $0, (%r9)"
                :
                : "{r9}"(data.r9) ::: "volatile"
            );
        }
        unsafe {
            llvm_asm!("
                mov $0, (%r10)"
                :
                : "{r10}"(data.r10) ::: "volatile"
            );
        }
        unsafe {
            llvm_asm!("
                mov $0, (%r11)"
                :
                : "{r11}"(data.r11) ::: "volatile"
            );
        }
        unsafe {
            llvm_asm!("
                mov $0, (%r12)"
                :
                : "{r12}"(data.r12) ::: "volatile"
            );
        }
        unsafe {
            llvm_asm!("
                mov $0, (%r13)"
                :
                : "{r13}"(data.r13) ::: "volatile"
            );
        }
        unsafe {
            llvm_asm!("
                mov $0, (%r14)"
                :
                : "{r14}"(data.r14) ::: "volatile"
            );
        }
        unsafe {
            llvm_asm!("
                mov $0, (%r15)"
                :
                : "{r15}"(data.r15) ::: "volatile"
            );
        }
        unsafe {
            llvm_asm!("
                mov $0, (%rip)"
                :
                : "{rip}"(data.rip) ::: "volatile"
            );
        }
        unsafe {
            llvm_asm!("
                mov $0, (%rflags)"
                :
                : "{rflags}"(data.rflags) ::: "volatile"
            );
        }
        unsafe {
            llvm_asm!("
                mov $0, (%cs)"
                :
                : "{cs}"(data.cs) ::: "volatile"
            );
        }
        unsafe {
            llvm_asm!("
                mov $0, (%fs)"
                :
                : "{fs}"(data.fs) ::: "volatile"
            );
        }
        unsafe {
            llvm_asm!("
                mov $0, (%gs)"
                :
                : "{gs}"(data.gs) ::: "volatile"
            );
        }
    }
}
