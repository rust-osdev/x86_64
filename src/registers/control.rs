//! Functions to read and write control registers.
//! See AMD64 Vol. 2 Section 3.1.1

use {VirtualAddress, PhysicalAddress};

bitflags! {
    pub flags Cr0: usize {
        const ENABLE_PAGING = 1 << 31,
        const CACHE_DISABLE = 1 << 30,
        const NOT_WRITE_THROUGH = 1 << 29,
        const ALIGNMENT_MASK = 1 << 18,
        const WRITE_PROTECT = 1 << 16,
        const NUMERIC_ERROR = 1 << 5,
        const EXTENSION_TYPE = 1 << 4,
        const TASK_SWITCHED = 1 << 3,
        const EMULATE_COPROCESSOR = 1 << 2,
        const MONITOR_COPROCESSOR = 1 << 1,
        const PROTECTED_MODE = 1 << 0,
    }
}

bitflags! {
    pub flags Cr4: usize {
        const ENABLE_SMAP = 1 << 21,
        const ENABLE_SMEP = 1 << 20,
        const ENABLE_OS_XSAVE = 1 << 18,
        const ENABLE_PCID = 1 << 17,
        const ENABLE_SMX = 1 << 14,
        const ENABLE_VMX = 1 << 13,
        const UNMASKED_SSE = 1 << 10,
        const ENABLE_SSE = 1 << 9,
        const ENABLE_PPMC = 1 << 8,
        const ENABLE_GLOBAL_PAGES = 1 << 7,
        const ENABLE_MACHINE_CHECK = 1 << 6,
        const ENABLE_PAE = 1 << 5,
        const ENABLE_PSE = 1 << 4,
        const DEBUGGING_EXTENSIONS = 1 << 3,
        const TIME_STAMP_DISABLE = 1 << 2,
        const VIRTUAL_INTERRUPTS = 1 << 1,
        const ENABLE_VME = 1 << 0,
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
    VirtualAddress(ret)
}

/// Contains page-table root pointer.
pub fn cr3() -> PhysicalAddress {
    let ret: u64;
    unsafe { asm!("mov %cr3, $0" : "=r" (ret)) };
    PhysicalAddress(ret)
}

/// Switch page-table PML4 pointer (level 4 page table).
///
/// # Safety
/// Changing the level 4 page table is unsafe, because it's possible to violate memory safety by
/// changing the page mapping.
pub unsafe fn cr3_write(val: PhysicalAddress) {
    asm!("mov $0, %cr3" :: "r" (val.0) : "memory");
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
