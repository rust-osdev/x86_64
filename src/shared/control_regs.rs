//! Functions to read and write control registers.
//! See Intel Vol. 3a Section 2.5, especially Figure 2.6.

bitflags! {
    pub flags Cr0: usize {
        const EnablePaging = 1 << 31,
        const CacheDisable = 1 << 30,
        const NotWriteThrough = 1 << 29,
        const AlignmentMask = 1 << 18,
        const WriteProtect = 1 << 16,
        const NumericError = 1 << 5,
        const ExtensionType = 1 << 4,
        const TaskSwitched = 1 << 3,
        const EmulateCoprocessor = 1 << 2,
        const MonitorCoprocessor = 1 << 1,
        const ProtectedMode = 1 << 0,
    }
}

bitflags! {
    pub flags Cr4: usize {
        const EnableSmap = 1 << 21,
        const EnableSmep = 1 << 20,
        const EnableOsXSave = 1 << 18,
        const EnablePcid = 1 << 17,
        const EnableSmx = 1 << 14,
        const EnableVmx = 1 << 13,
        const UnmaskedSse = 1 << 10,
        const EnableSse = 1 << 9,
        const EnablePpmc = 1 << 8,
        const EnableGlobalPages = 1 << 7,
        const EnableMachineCheck = 1 << 6,
        const EnablePae = 1 << 5,
        const EnablePse = 1 << 4,
        const DebuggingExtensions = 1 << 3,
        const TimeStampDisable = 1 << 2,
        const VirtualInterrupts = 1 << 1,
        const EnableVme = 1 << 0,
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
