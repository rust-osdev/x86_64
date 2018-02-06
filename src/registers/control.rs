//! Functions to read and write control registers.

pub use super::model_specific::Efer;

use PhysAddr;
use structures::paging::PhysFrame;

/// Various control flags modifying the basic operation of the CPU.
pub struct Cr0;

impl Cr0 {
    /// Read the current set of CR0 flags.
    pub fn read() -> Cr0Flags {
        let value: u64;
        unsafe {
            asm!("mov %cr0, $0" : "=r" (value));
        }
        Cr0Flags::from_bits_truncate(value)
    }

    /// Write CR0 flags.
    ///
    /// Preserves the value of reserved fields. Unsafe because it's possible to violate memory
    /// safety by e.g. disabling paging.
    pub unsafe fn write(flags: Cr0Flags) {
        let mut value = Self::read();
        value |= flags;
        let value = value.bits();

        asm!("mov $0, %cr0" :: "r" (value) : "memory")
    }
}

bitflags! {
    pub struct Cr0Flags: u64 {
        const PROTECTED_MODE_ENABLE = 1 << 0;
        const MONITOR_COPROCESSOR = 1 << 1;
        const EMULATION = 1 << 2;
        const TASK_SWITCHED = 1 << 3;
        const EXTENSION_TYPE = 1 << 4;
        const NUMERIC_ERROR = 1 << 5;
        const WRITE_PROTECT = 1 << 16;
        const ALIGNMENT_MASK = 1 << 18;
        const NON_WRITE_THROUGH = 1 << 29;
        const CACHE_DISABLE = 1 << 30;
        const PAGING = 1 << 31;
    }
}


/// Contains the physical address of the level 4 page table.
pub struct Cr3;

impl Cr3 {
    /// Read the current P4 table address from the CR3 register.
    pub fn read() -> (PhysFrame, Cr3Flags) {
        let value: u64;
        unsafe {
            asm!("mov %cr3, $0" : "=r" (value));
        }
        let flags = Cr3Flags::from_bits_truncate(value);
        let addr = PhysAddr::new(value & 0x_000f_ffff_ffff_f000);
        let frame = PhysFrame::containing_address(addr);
        (frame, flags)
    }

    /// Write a new P4 table address into the CR3 register.
    ///
    /// ## Safety
    /// Changing the level 4 page table is unsafe, because it's possible to violate memory safety by
    /// changing the page mapping.
    pub unsafe fn write(frame: PhysFrame, flags: Cr3Flags) {
        let addr = frame.start_address();
        let value = addr.as_u64() | flags.bits();
        asm!("mov $0, %cr3" :: "r" (value) : "memory")
    }
}

bitflags! {
    /// Controls cache settings for the level 4 page table.
    pub struct Cr3Flags: u64 {
        /// Use a writethrough cache policy for the P4 table (else a writeback policy is used).
        const PAGE_LEVEL_WRITETHROUGH = 1 << 3;
        /// Disable caching for the P4 table.
        const PAGE_LEVEL_CACHE_DISABLE = 1 << 4;
    }
}
