/// Contains various flags to control operations.
pub unsafe fn cr0() -> u64
{
    let ret: u64;
    asm!("mov %cr0, $0" : "=r" (ret) : );
    ret
}

/// Write cr0.
pub unsafe fn cr0_write(val: u64)
{
    asm!("mov $0, %cr0" :: "r" (val));
}

/// Contains page-fault linear address.
pub unsafe fn cr2() -> u64 {
    let ret: u64;
    asm!("mov %cr2, $0" : "=r" (ret) :);
    ret
}

/// Contains page-table root pointer.
pub unsafe fn cr3() -> u64 {
    let ret: u64;
    asm!("mov %cr3, $0" : "=r" (ret) :);
    ret
}

/// Switch page-table PML4 pointer.
pub unsafe fn cr3_write(val: u64) {
    asm!("mov $0, %cr3" :: "r" (val));
}

/// Contains various flags to control operations in protected mode.
pub unsafe fn cr4() -> u64 {
    let ret: u64;
    asm!("mov %cr4, $0" : "=r" (ret) :);
    ret
}

/// Write cr4.
pub unsafe fn cr4_write(val: u64) {
    asm!("mov $0, %cr4" :: "r" (val));
}
