//! Functions to read and write control registers.
//! See Intel Vol. 3a Section 2.5, especially Figure 2.6.

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
        const CR4_ENABLE_SMAP = 1 << 21,
        const CR4_ENABLE_SMEP = 1 << 20,
        const CR4_ENABLE_OS_XSAVE = 1 << 18,
        const CR4_ENABLE_PCID = 1 << 17,
        const CR4_ENABLE_SMX = 1 << 14,
        const CR4_ENABLE_VMX = 1 << 13,
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
pub unsafe fn cr3() -> usize {
    let ret: usize;
    asm!("mov %cr3, $0" : "=r" (ret));
    ret
}

/// Switch page-table PML4 pointer.
pub unsafe fn cr3_write(val: usize) {
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
