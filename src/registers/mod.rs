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

/// Stores all data from all registers, in event of e.g. a context switch
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
    
    #[cfg(feature = "llvm_asm")]
    pub fn backup() -> RegData {
        let reg_rax: usize;
        let reg_rbx: usize;
        let reg_rcx: usize;
        let reg_rdx: usize;
        let reg_rdi: usize;
        let reg_rsi: usize;
        let reg_rbp: usize;
        let reg_rsp: usize;
        let reg_r8: usize;
        let reg_r9: usize;
        let reg_r10: usize;
        let reg_r11: usize;
        let reg_r12: usize;
        let reg_r13: usize;
        let reg_r14: usize;
        let reg_r15: usize;
        let reg_rip: usize;
        let reg_rflags: usize;
        let reg_cs: usize;
        let reg_fs: usize;
        let reg_gs: usize;
        
        unsafe {
            llvm_asm!("
                mov (%rax), $0"
                :"=i"(reg_rax) ::: "volatile"
            );
            
            llvm_asm!("
                mov (%rbx), $0"
                :"=i"(reg_rbx) ::: "volatile"
            );
            
            llvm_asm!("
                mov (%rcx), $0"
                :"=i"(reg_rcx) ::: "volatile"
            );
            
            llvm_asm!("
                mov (%rdx), $0"
                :"=i"(reg_rdx) ::: "volatile"
            );
            
            llvm_asm!("
                mov (%rdi), $0"
                :"=i"(reg_rdi) ::: "volatile"
            );
            
            llvm_asm!("
                mov (%rsi), $0"
                :"=i"(reg_rsi) ::: "volatile"
            );
            
            llvm_asm!("
                mov (%rbp), $0"
                :"=i"(reg_rbp) ::: "volatile"
            );
            
            llvm_asm!("
                mov (%rsp), $0"
                :"=i"(reg_rsp) ::: "volatile"
            );
            
            llvm_asm!("
                mov (%r8), $0"
                :"=i"(reg_r8) ::: "volatile"
            );
            
            llvm_asm!("
                mov (%r9), $0"
                :"=i"(reg_r9) ::: "volatile"
            );
            
            llvm_asm!("
                mov (%r10), $0"
                :"=i"(reg_r10) ::: "volatile"
            );
            
            llvm_asm!("
                mov (%r11), $0"
                :"=i"(reg_r11) ::: "volatile"
            );
            
            llvm_asm!("
                mov (%r12), $0"
                :"=i"(reg_r12) ::: "volatile"
            );
            
            llvm_asm!("
                mov (%r13), $0"
                :"=i"(reg_r13) ::: "volatile"
            );
            
            llvm_asm!("
                mov (%r14), $0"
                :"=i"(reg_r14) ::: "volatile"
            );
            
            llvm_asm!("
                mov (%r15), $0"
                :"=i"(reg_r15) ::: "volatile"
            );
            llvm_asm!("
                mov (%rip), $0"
                :"=i"(reg_rip) ::: "volatile"
            );
            llvm_asm!("
                mov (%rflags), $0"
                :"=i"(reg_rflags) ::: "volatile"
            );
            llvm_asm!("
                mov (%cs), $0"
                :"=i"(reg_cs) ::: "volatile"
            );
            llvm_asm!("
                mov (%fs), $0"
                :"=i"(reg_fs) ::: "volatile"
            );
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

    #[cfg(feature = "llvm_asm")]
    pub fn restore(data: RegData) {
        unsafe {
            llvm_asm!("
                mov $0, (%rax)"
                :
                : "{rax}"(data.rax) ::: "volatile"
            );
            
            llvm_asm!("
                mov $0, (%rbx)"
                :
                : "{rbx}"(data.rbx) ::: "volatile"
            );
            
            llvm_asm!("
                mov $0, (%rcx)"
                :
                : "{rcx}"(data.rcx) ::: "volatile"
            );
            
            llvm_asm!("
                mov $0, (%rdx)"
                :
                : "{rdx}"(data.rdx) ::: "volatile"
            );
            
            llvm_asm!("
                mov $0, (%rdi)"
                :
                : "{rdi}"(data.rdi) ::: "volatile"
            );
            
            llvm_asm!("
                mov $0, (%rsi)"
                :
                : "{rsi}"(data.rsi) ::: "volatile"
            );
            
            llvm_asm!("
                mov $0, (%rbp)"
                :
                : "{rbp}"(data.rbp) ::: "volatile"
            );
            
            llvm_asm!("
                mov $0, (%rsp)"
                :
                : "{rsp}"(data.rsp) ::: "volatile"
            );
            
            llvm_asm!("
                mov $0, (%r8)"
                :
                : "{r8}"(data.r8) ::: "volatile"
            );
            
            llvm_asm!("
                mov $0, (%r9)"
                :
                : "{r9}"(data.r9) ::: "volatile"
            );
            
            llvm_asm!("
                mov $0, (%r10)"
                :
                : "{r10}"(data.r10) ::: "volatile"
            );
            
            llvm_asm!("
                mov $0, (%r11)"
                :
                : "{r11}"(data.r11) ::: "volatile"
            );
            
            llvm_asm!("
                mov $0, (%r12)"
                :
                : "{r12}"(data.r12) ::: "volatile"
            );
            
            llvm_asm!("
                mov $0, (%r13)"
                :
                : "{r13}"(data.r13) ::: "volatile"
            );
            
            llvm_asm!("
                mov $0, (%r14)"
                :
                : "{r14}"(data.r14) ::: "volatile"
            );
            
            llvm_asm!("
                mov $0, (%r15)"
                :
                : "{r15}"(data.r15) ::: "volatile"
            );
            
            llvm_asm!("
                mov $0, (%rip)"
                :
                : "{rip}"(data.rip) ::: "volatile"
            );
            
            llvm_asm!("
                mov $0, (%rflags)"
                :
                : "{rflags}"(data.rflags) ::: "volatile"
            );

            llvm_asm!("
                mov $0, (%cs)"
                :
                : "{cs}"(data.cs) ::: "volatile"
            );
            
            llvm_asm!("
                mov $0, (%fs)"
                :
                : "{fs}"(data.fs) ::: "volatile"
            );
            
            llvm_asm!("
                mov $0, (%gs)"
                :
                : "{gs}"(data.gs) ::: "volatile"
            );
        }
    }
}
