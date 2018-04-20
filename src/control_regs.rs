//! Functions to read and write control registers.
//! See Intel Vol. 3a Section 2.5, especially Figure 2-7.

bitflags! {
    pub flags Cr0: usize {
        const CR0_ENABLE_PAGING = 1 << 31,
        const CR0_CACHE_DISABLE = 1 << 30,
        const CR0_NOT_WRITE_THROUGH = 1 << 29,
        const CR0_ALIGNMENT_MASK = 1 << 18,
        const CR0_WRITE_PROTECT = 1 << 16,
        const CR0_NUMERIC_ERROR = 1 << 5,
        const CR0_EXTENSION_TYPE = 1 << 4,
        const CR0_TASK_SWITCHED = 1 << 3,
        const CR0_EMULATE_COPROCESSOR = 1 << 2,
        const CR0_MONITOR_COPROCESSOR = 1 << 1,
        const CR0_PROTECTED_MODE = 1 << 0,
    }
}

bitflags! {
    pub flags Cr4: usize {
        const CR4_ENABLE_PROTECTION_KEY = 1 << 22,
        const CR4_ENABLE_SMAP = 1 << 21,
        const CR4_ENABLE_SMEP = 1 << 20,
        const CR4_ENABLE_OS_XSAVE = 1 << 18,
        const CR4_ENABLE_PCID = 1 << 17,
        const CR4_ENABLE_FSGSBASE = 1 << 16,
        const CR4_ENABLE_SMX = 1 << 14,
        const CR4_ENABLE_VMX = 1 << 13,
        const CR4_ENABLE_UMIP = 1 << 11,
        const CR4_UNMASKED_SSE = 1 << 10,
        const CR4_ENABLE_SSE = 1 << 9,
        const CR4_ENABLE_PPMC = 1 << 8,
        const CR4_ENABLE_GLOBAL_PAGES = 1 << 7,
        const CR4_ENABLE_MACHINE_CHECK = 1 << 6,
        const CR4_ENABLE_PAE = 1 << 5,
        const CR4_ENABLE_PSE = 1 << 4,
        const CR4_DEBUGGING_EXTENSIONS = 1 << 3,
        const CR4_TIME_STAMP_DISABLE = 1 << 2,
        const CR4_VIRTUAL_INTERRUPTS = 1 << 1,
        const CR4_ENABLE_VME = 1 << 0,
    }
}

bitflags! {
    pub flags Xcr0: u64 {
        const XCR0_PKRU_STATE = 1 << 9,
        const XCR0_HI16_ZMM_STATE = 1 << 7,
        const XCR0_ZMM_HI256_STATE = 1 << 6,
        const XCR0_OPMASK_STATE = 1 << 5,
        const XCR0_BNDCSR_STATE = 1 << 4,
        const XCR0_BNDREG_STATE = 1 << 3,
        const XCR0_AVX_STATE = 1 << 2,
        const XCR0_SSE_STATE = 1 << 1,
        const XCR0_FPU_MMX_STATE = 1 << 0,
    }
}


/// Read cr0
pub unsafe fn cr0() -> Cr0 {
    let ret: usize;
    asm!("mov %cr0, $0" : "=r" (ret));
    Cr0::from_bits_truncate(ret)
}

/// Write cr0.
pub unsafe fn cr0_write(val: Cr0) {
    asm!("mov $0, %cr0" :: "r" (val.bits) : "memory");
}

/// Contains page-fault linear address.
pub unsafe fn cr2() -> usize {
    let ret: usize;
    asm!("mov %cr2, $0" : "=r" (ret));
    ret
}

/// Contains page-table root pointer.
pub unsafe fn cr3() -> u64 {
    let ret: u64;
    asm!("mov %cr3, $0" : "=r" (ret));
    ret
}

/// Switch page-table PML4 pointer.
pub unsafe fn cr3_write(val: u64) {
    asm!("mov $0, %cr3" :: "r" (val) : "memory");
}

/// Contains various flags to control operations in protected mode.
pub unsafe fn cr4() -> Cr4 {
    let ret: usize;
    asm!("mov %cr4, $0" : "=r" (ret));
    Cr4::from_bits_truncate(ret)
}

/// Write cr4.
pub unsafe fn cr4_write(val: Cr4) {
    asm!("mov $0, %cr4" :: "r" (val.bits) : "memory");
}

/// Read Extended Control Register XCR0.
/// Only supported if CR4_ENABLE_OS_XSAVE is set.
pub unsafe fn xcr0() -> Xcr0 {
    let high: u32;
    let low: u32;
    asm!("xgetbv" : "={eax}"(low), "={edx}"(high) : "{ecx}"(0));
    Xcr0::from_bits_truncate((high as u64) << 32 | low as u64)
}

/// Write to Extended Control Register XCR0.
/// Only supported if CR4_ENABLE_OS_XSAVE is set.
pub unsafe fn xcr0_write(val: Xcr0) {
    let high: u32 = (val.bits >> 32) as u32;
    let low: u32 = val.bits as u32;
    asm!("xsetbv" :: "{eax}"(low), "{ecx}"(0), "{edx}"(high));
}
