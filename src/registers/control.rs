//! Functions to read and write control registers.

pub use super::model_specific::{Efer, EferFlags};

use bitflags::bitflags;

/// Various control flags modifying the basic operation of the CPU.
#[derive(Debug)]
pub struct Cr0;

bitflags! {
    /// Configuration flags of the Cr0 register.
    pub struct Cr0Flags: u64 {
        /// Enables protected mode.
        const PROTECTED_MODE_ENABLE = 1 << 0;
        /// Enables monitoring of the coprocessor, typical for x87 instructions.
        ///
        /// Controls together with the `TASK_SWITCHED` flag whether a `wait` or `fwait`
        /// instruction should cause a device-not-available exception.
        const MONITOR_COPROCESSOR = 1 << 1;
        /// Force all x87 and MMX instructions to cause an exception.
        const EMULATE_COPROCESSOR = 1 << 2;
        /// Automatically set to 1 on _hardware_ task switch.
        ///
        /// This flags allows lazily saving x87/MMX/SSE instructions on hardware context switches.
        const TASK_SWITCHED = 1 << 3;
        /// Enables the native error reporting mechanism for x87 FPU errors.
        const NUMERIC_ERROR = 1 << 5;
        /// Controls whether supervisor-level writes to read-only pages are inhibited.
        ///
        /// When set, it is not possible to write to read-only pages from ring 0.
        const WRITE_PROTECT = 1 << 16;
        /// Enables automatic alignment checking.
        const ALIGNMENT_MASK = 1 << 18;
        /// Ignored. Used to control write-back/write-through cache strategy on older CPUs.
        const NOT_WRITE_THROUGH = 1 << 29;
        /// Disables internal caches (only for some cases).
        const CACHE_DISABLE = 1 << 30;
        /// Enables page translation.
        const PAGING = 1 << 31;
    }
}

/// Contains the Page Fault Linear Address (PFLA).
///
/// When page fault occurs, the CPU sets this register to the accessed address.
#[derive(Debug)]
pub struct Cr2;

/// Contains the physical address of the level 4 page table.
#[derive(Debug)]
pub struct Cr3;

bitflags! {
    /// Controls cache settings for the level 4 page table.
    pub struct Cr3Flags: u64 {
        /// Use a writethrough cache policy for the P4 table (else a writeback policy is used).
        const PAGE_LEVEL_WRITETHROUGH = 1 << 3;
        /// Disable caching for the P4 table.
        const PAGE_LEVEL_CACHE_DISABLE = 1 << 4;
    }
}

/// Contains various flags for controling extended processor settings.
/// These settings include PCIDE, VME, etc.
#[derive(Debug)]
pub struct Cr4;

bitflags! {
pub struct Cr4Flags: u64 {
const VIRTUAL_8086_MODE_EXTENSIONS = 1<<0;
const PROTECTED_MODE_VIRTUAL_INTERRUPTS = 1<<1;
const TIME_STAMP_RING0_ONLY = 1<<2;
const DEBUGGING_EXTENSIONS = 1<<3;
const PAGE_SIZE_EXTENSION = 1 << 4;
const PHYSICAL_ADDRESS_EXTENSION = 1 << 5;
const MACHINE_CHECK_EXCEPTION = 1 << 6;
const PAGE_GLOBAL_ENABLE = 1 << 7;
const PERFORMANCE_MONITORING_COUNTER_ENABLE = 1 << 8;
const OS_SUPPORTS_FXSAVE_FXSTOR = 1 << 9;
const OS_SUPPORTS_UNMASKED_SIMD_FP_EXCEPTIONS = 1 << 10;
const USER_MODE_INSTRUCTION_PREVENTION = 1 << 11;
const VIRTUAL_MACHINE_EXTENSIONS_ENABLE = 1 << 13;
const SAFER_MODE_EXTENSIONS_ENABLE = 1 << 14;
const PCID_ENABLE = 1 << 17;
const XSAVE_AND_PROCESSOR_EXTENDED_STATES_ENABLE = 1 << 18;
const SUPERVISOR_MODE_EXECUTIONS_PROTECTION_ENABLE = 1 << 20;
const SUPERVISOR_MODE_ACCESS_PROTECTION_ENABLE = 1 << 21;
}

#[cfg(target_arch = "x86_64")]
mod x86_64 {
    use super::*;
    use crate::structures::paging::PhysFrame;
    use crate::{PhysAddr, VirtAddr};

    impl Cr0 {
        /// Read the current set of CR0 flags.
        pub fn read() -> Cr0Flags {
            Cr0Flags::from_bits_truncate(Self::read_raw())
        }

        /// Read the current raw CR0 value.
        pub fn read_raw() -> u64 {
            let value: u64;
            unsafe {
                asm!("mov %cr0, $0" : "=r" (value));
            }
            value
        }

        /// Write CR0 flags.
        ///
        /// Preserves the value of reserved fields. Unsafe because it's possible to violate memory
        /// safety by e.g. disabling paging.
        pub unsafe fn write(flags: Cr0Flags) {
            let old_value = Self::read_raw();
            let reserved = old_value & !(Cr0Flags::all().bits());
            let new_value = reserved | flags.bits();

            Self::write_raw(new_value);
        }

        /// Write raw CR0 flags.
        ///
        /// Does _not_ preserve any values, including reserved fields. Unsafe because it's possible to violate memory
        /// safety by e.g. disabling paging.
        pub unsafe fn write_raw(value: u64) {
            asm!("mov $0, %cr0" :: "r" (value) : "memory")
        }

        /// Updates CR0 flags.
        ///
        /// Preserves the value of reserved fields. Unsafe because it's possible to violate memory
        /// safety by e.g. disabling paging.
        pub unsafe fn update<F>(f: F)
        where
            F: FnOnce(&mut Cr0Flags),
        {
            let mut flags = Self::read();
            f(&mut flags);
            Self::write(flags);
        }
    }

    impl Cr2 {
        /// Read the current page fault linear address from the CR3 register.
        pub fn read() -> VirtAddr {
            let value: u64;
            unsafe {
                asm!("mov %cr2, $0" : "=r" (value));
            }
            VirtAddr::new(value)
        }
    }

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

    impl Cr4 {
        /// Read the current set of CR4 flags.
        pub fn read() -> Cr4Flags {
            Cr4Flags::from_bits_truncate(Self::read_raw())
        }

        /// Read the current raw CR4 value.
        pub fn read_raw() -> u64 {
            let value: u64;
            unsafe {
                asm!("mov %cr4, $0" : "=r" (value));
            }
            value
        }

        /// Write CR4 flags.
        ///
        /// Preserves the value of reserved fields. Unsafe because it's possible to violate memory
        /// safety by e.g. disabling PAE.
        pub unsafe fn write(flags: Cr4Flags) {
            let old_value = Self::read_raw();
            let reserved = old_value & !(Cr4Flags::all().bits());
            let new_value = reserved | flags.bits();

            Self::write_raw(new_value);
        }

        /// Write raw CR4 flags.
        ///
        /// Does _not_ preserve any values, including reserved fields. Unsafe because it's possible to violate memory
        /// safety by e.g. disabling PAE.
        pub unsafe fn write_raw(value: u64) {
            asm!("mov $0, %cr4" :: "r" (value) : "memory")
        }

        /// Updates CR4 flags.
        ///
        /// Preserves the value of reserved fields. Unsafe because it's possible to violate memory
        /// safety by e.g. disabling PAE.
        pub unsafe fn update<F>(f: F)
        where
            F: FnOnce(&mut Cr4Flags),
        {
            let mut flags = Self::read();
            f(&mut flags);
            Self::write(flags);
        }
    }
}
