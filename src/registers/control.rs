//! Functions to read and write control registers.

pub use super::model_specific::{Efer, EferFlags};
pub use x86_64_types::registers::{Cr0 as Cr0Flags, Cr3 as Cr3Flags};

/// Various control flags modifying the basic operation of the CPU.
#[derive(Debug)]
pub struct Cr0;

/// Contains the Page Fault Linear Address (PFLA).
///
/// When page fault occurs, the CPU sets this register to the accessed address.
#[derive(Debug)]
pub struct Cr2;

/// Contains the physical address of the level 4 page table.
#[derive(Debug)]
pub struct Cr3;

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
}
