//! Functions to read and write control registers.
//! See AMD64 Vol. 2 Section 3.1.1

use {VirtualAddress, PhysicalAddress};

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

/// Read CR0
pub fn cr0() -> Cr0 {
    let ret: usize;
    unsafe { asm!("mov %cr0, $0" : "=r" (ret)) };
    Cr0::from_bits_truncate(ret)
}

/// Write CR0.
///
/// # Safety
/// Changing the CR0 register is unsafe, because e.g. disabling paging would violate memory safety.
pub unsafe fn cr0_write(val: Cr0) {
    asm!("mov $0, %cr0" :: "r" (val.bits()) : "memory");
}

/// Update CR0.
///
/// # Safety
/// Changing the CR0 register is unsafe, because e.g. disabling paging would violate memory safety.
pub unsafe fn cr0_update<F>(f: F)
    where F: FnOnce(&mut Cr0)
{
    let mut value = cr0();
    f(&mut value);
    cr0_write(value);
}

/// Contains page-fault virtual address.
pub fn cr2() -> VirtualAddress {
    let ret: usize;
    unsafe { asm!("mov %cr2, $0" : "=r" (ret)) };
    VirtualAddress::from(ret)
}

/// Contains page-table root pointer.
pub fn cr3() -> PhysicalAddress {
    let ret: u64;
    unsafe { asm!("mov %cr3, $0" : "=r" (ret)) };
    PhysicalAddress::from(ret)
}

/// Switch page-table PML4 pointer (level 4 page table).
///
/// # Safety
/// Changing the level 4 page table is unsafe, because it's possible to violate memory safety by
/// changing the page mapping.
pub unsafe fn cr3_write(val: PhysicalAddress) {
    asm!("mov $0, %cr3" :: "r" (val.as_u64()) : "memory");
}

/// Contains various flags to control operations in protected mode.
pub fn cr4() -> Cr4 {
    let ret: usize;
    unsafe { asm!("mov %cr4, $0" : "=r" (ret)) };
    Cr4::from_bits_truncate(ret)
}

/// Write cr4.
///
/// # Safety
/// It's not clear if it's always memory safe to change the CR4 register.
pub unsafe fn cr4_write(val: Cr4) {
    asm!("mov $0, %cr4" :: "r" (val.bits) : "memory");
}
