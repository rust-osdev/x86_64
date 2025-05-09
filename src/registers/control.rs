//! Functions to read and write control registers.

pub use super::model_specific::{Efer, EferFlags};
use bitflags::bitflags;

/// Various control flags modifying the basic operation of the CPU.
#[derive(Debug)]
pub struct Cr0;

bitflags! {
    /// Configuration flags of the [`Cr0`] register.
    #[repr(transparent)]
    #[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Clone, Copy)]
    pub struct Cr0Flags: u64 {
        /// Enables protected mode.
        const PROTECTED_MODE_ENABLE = 1;
        /// Enables monitoring of the coprocessor, typical for x87 instructions.
        ///
        /// Controls (together with the [`TASK_SWITCHED`](Cr0Flags::TASK_SWITCHED)
        /// flag) whether a `wait` or `fwait` instruction should cause an `#NE` exception.
        const MONITOR_COPROCESSOR = 1 << 1;
        /// Force all x87 and MMX instructions to cause an `#NE` exception.
        const EMULATE_COPROCESSOR = 1 << 2;
        /// Automatically set to 1 on _hardware_ task switch.
        ///
        /// This flags allows lazily saving x87/MMX/SSE instructions on hardware context switches.
        const TASK_SWITCHED = 1 << 3;
        /// Indicates support of 387DX math coprocessor instructions.
        ///
        /// Always set on all recent x86 processors, cannot be cleared.
        const EXTENSION_TYPE = 1 << 4;
        /// Enables the native (internal) error reporting mechanism for x87 FPU errors.
        const NUMERIC_ERROR = 1 << 5;
        /// Controls whether supervisor-level writes to read-only pages are inhibited.
        ///
        /// When set, it is not possible to write to read-only pages from ring 0.
        const WRITE_PROTECT = 1 << 16;
        /// Enables automatic usermode alignment checking if [`RFlags::ALIGNMENT_CHECK`] is also set.
        const ALIGNMENT_MASK = 1 << 18;
        /// Ignored, should always be unset.
        ///
        /// Must be unset if [`CACHE_DISABLE`](Cr0Flags::CACHE_DISABLE) is unset.
        /// Older CPUs used this to control write-back/write-through cache strategy.
        const NOT_WRITE_THROUGH = 1 << 29;
        /// Disables some processor caches, specifics are model-dependent.
        const CACHE_DISABLE = 1 << 30;
        /// Enables paging.
        ///
        /// If this bit is set, [`PROTECTED_MODE_ENABLE`](Cr0Flags::PROTECTED_MODE_ENABLE) must be set.
        const PAGING = 1 << 31;
    }
}

/// Contains the Page Fault Linear Address (PFLA).
///
/// When a page fault occurs, the CPU sets this register to the faulting virtual address.
#[derive(Debug)]
pub struct Cr2;

/// Contains the physical address of the highest-level page table.
#[derive(Debug)]
pub struct Cr3;

bitflags! {
    /// Controls cache settings for the highest-level page table.
    ///
    /// Unused if paging is disabled or if [`PCID`](Cr4Flags::PCID) is enabled.
    #[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Clone, Copy)]
    pub struct Cr3Flags: u64 {
        /// Use a writethrough cache policy for the table (otherwise a writeback policy is used).
        const PAGE_LEVEL_WRITETHROUGH = 1 << 3;
        /// Disable caching for the table.
        const PAGE_LEVEL_CACHE_DISABLE = 1 << 4;
    }
}

/// Contains various control flags that enable architectural extensions, and
/// indicate support for specific processor capabilities.
#[derive(Debug)]
pub struct Cr4;

bitflags! {
    /// Configuration flags of the [`Cr4`] register.
    #[repr(transparent)]
    #[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Clone, Copy)]
    pub struct Cr4Flags: u64 {
        /// Enables hardware-supported performance enhancements for software running in
        /// virtual-8086 mode.
        const VIRTUAL_8086_MODE_EXTENSIONS = 1;
        /// Enables support for protected-mode virtual interrupts.
        const PROTECTED_MODE_VIRTUAL_INTERRUPTS = 1 << 1;
        /// When set, only privilege-level 0 can execute the `RDTSC` or `RDTSCP` instructions.
        const TIMESTAMP_DISABLE = 1 << 2;
        /// Enables I/O breakpoint capability and enforces treatment of `DR4` and `DR5` registers
        /// as reserved.
        const DEBUGGING_EXTENSIONS = 1 << 3;
        /// Enables the use of 4MB physical frames; ignored if
        /// [`PHYSICAL_ADDRESS_EXTENSION`](Cr4Flags::PHYSICAL_ADDRESS_EXTENSION)
        /// is set (so always ignored in long mode).
        const PAGE_SIZE_EXTENSION = 1 << 4;
        /// Enables physical address extensions and 2MB physical frames. Required in long mode.
        const PHYSICAL_ADDRESS_EXTENSION = 1 << 5;
        /// Enables the machine-check exception mechanism.
        const MACHINE_CHECK_EXCEPTION = 1 << 6;
        /// Enables the global page feature, allowing some page translations to
        /// be marked as global (see [`PageTableFlags::GLOBAL`]).
        const PAGE_GLOBAL = 1 << 7;
        /// Allows software running at any privilege level to use the `RDPMC` instruction.
        const PERFORMANCE_MONITOR_COUNTER = 1 << 8;
        /// Enables the use of legacy SSE instructions; allows using `FXSAVE`/`FXRSTOR` for saving
        /// processor state of 128-bit media instructions.
        const OSFXSR = 1 << 9;
        /// Enables the SIMD floating-point exception (`#XF`) for handling unmasked 256-bit and
        /// 128-bit media floating-point errors.
        const OSXMMEXCPT_ENABLE = 1 << 10;
        /// Prevents the execution of the `SGDT`, `SIDT`, `SLDT`, `SMSW`, and `STR` instructions by
        /// user-mode software.
        const USER_MODE_INSTRUCTION_PREVENTION = 1 << 11;
        /// Enables 5-level paging on supported CPUs (Intel Only).
        const L5_PAGING = 1 << 12;
        /// Enables VMX instructions (Intel Only).
        const VIRTUAL_MACHINE_EXTENSIONS = 1 << 13;
        /// Enables SMX instructions (Intel Only).
        const SAFER_MODE_EXTENSIONS = 1 << 14;
        /// Enables software running in 64-bit mode at any privilege level to read and write
        /// the FS.base and GS.base hidden segment register state.
        const FSGSBASE = 1 << 16;
        /// Enables process-context identifiers (PCIDs).
        const PCID = 1 << 17;
        /// Enables extended processor state management instructions, including `XGETBV` and `XSAVE`.
        const OSXSAVE = 1 << 18;
        /// Enables the Key Locker feature (Intel Only).
        ///
        /// This enables creation and use of opaque AES key handles; see the
        /// [Intel Key Locker Specification](https://software.intel.com/content/www/us/en/develop/download/intel-key-locker-specification.html)
        /// for more information.
        const KEY_LOCKER = 1 << 19;
        /// Prevents the execution of instructions that reside in pages accessible by user-mode
        /// software when the processor is in supervisor-mode.
        const SUPERVISOR_MODE_EXECUTION_PROTECTION = 1 << 20;
        /// Enables restrictions for supervisor-mode software when reading data from user-mode
        /// pages.
        const SUPERVISOR_MODE_ACCESS_PREVENTION = 1 << 21;
        /// Enables protection keys for user-mode pages.
        ///
        /// Also enables access to the PKRU register (via the `RDPKRU`/`WRPKRU`
        /// instructions) to set user-mode protection key access controls.
        const PROTECTION_KEY_USER = 1 << 22;
        /// Enables Control-flow Enforcement Technology (CET)
        ///
        /// This enables the shadow stack feature, ensuring return addresses read
        /// via `RET` and `IRET` have not been corrupted.
        const CONTROL_FLOW_ENFORCEMENT = 1 << 23;
        /// Enables protection keys for supervisor-mode pages (Intel Only).
        ///
        /// Also enables the `IA32_PKRS` MSR to set supervisor-mode protection
        /// key access controls.
        const PROTECTION_KEY_SUPERVISOR = 1 << 24;
    }
}

/// Contains the task priority.
#[derive(Debug)]
pub struct Cr8;

/// A priority class for an interrupt. Loading CR8 with a priority class blocks
/// all interrupts of that class or lower. Note that 0 is not a priority class,
/// if CR8 contains 0, all interrupts are enabled regardless of their priority
/// class.
#[derive(Debug)]
pub enum PriorityClass {
    // 0 is not a valid priority class, 1 is the first valid class
    /// Priority class 1
    PriorityClass1 = 1,
    /// Priority class 2
    PriorityClass2,
    /// Priority class 3
    PriorityClass3,
    /// Priority class 4
    PriorityClass4,
    /// Priority class 5
    PriorityClass5,
    /// Priority class 6
    PriorityClass6,
    /// Priority class 7
    PriorityClass7,
    /// Priority class 8
    PriorityClass8,
    /// Priority class 9
    PriorityClass9,
    /// Priority class 10
    PriorityClass10,
    /// Priority class 11
    PriorityClass11,
    /// Priority class 12
    PriorityClass12,
    /// Priority class 13
    PriorityClass13,
    /// Priority class 14
    PriorityClass14,
    /// Priority class 15
    PriorityClass15,
}

impl PriorityClass {
    /// Convert a number into a priority class
    pub const fn new(priority_class: u8) -> Option<Self> {
        Some(match priority_class {
            1 => Self::PriorityClass1,
            2 => Self::PriorityClass2,
            3 => Self::PriorityClass3,
            4 => Self::PriorityClass4,
            5 => Self::PriorityClass5,
            6 => Self::PriorityClass6,
            7 => Self::PriorityClass7,
            8 => Self::PriorityClass8,
            9 => Self::PriorityClass9,
            10 => Self::PriorityClass10,
            11 => Self::PriorityClass11,
            12 => Self::PriorityClass12,
            13 => Self::PriorityClass13,
            14 => Self::PriorityClass14,
            15 => Self::PriorityClass15,
            _ => return None,
        })
    }
}

#[cfg(all(feature = "instructions", target_arch = "x86_64"))]
mod x86_64 {
    use super::*;
    use crate::{
        addr::VirtAddrNotValid, instructions::tlb::Pcid, structures::paging::PhysFrame, PhysAddr,
        VirtAddr,
    };
    use core::arch::asm;

    impl Cr0 {
        /// Read the current set of CR0 flags.
        #[inline]
        pub fn read() -> Cr0Flags {
            Cr0Flags::from_bits_truncate(Self::read_raw())
        }

        /// Read the current raw CR0 value.
        #[inline]
        pub fn read_raw() -> u64 {
            let value: u64;

            unsafe {
                asm!("mov {}, cr0", out(reg) value, options(nomem, nostack, preserves_flags));
            }

            value
        }

        /// Write CR0 flags.
        ///
        /// Preserves the value of reserved fields.
        ///
        /// ## Safety
        ///
        /// This function is unsafe because it's possible to violate memory
        /// safety through it, e.g. by disabling paging.
        #[inline]
        pub unsafe fn write(flags: Cr0Flags) {
            let old_value = Self::read_raw();
            let reserved = old_value & !(Cr0Flags::all().bits());
            let new_value = reserved | flags.bits();

            unsafe {
                Self::write_raw(new_value);
            }
        }

        /// Write raw CR0 flags.
        ///
        /// Does _not_ preserve any values, including reserved fields.
        ///
        /// ## Safety
        ///
        /// This function is unsafe because it's possible to violate memory
        /// safety through it, e.g. by disabling paging.
        #[inline]
        pub unsafe fn write_raw(value: u64) {
            unsafe {
                asm!("mov cr0, {}", in(reg) value, options(nostack, preserves_flags));
            }
        }

        /// Updates CR0 flags.
        ///
        /// Preserves the value of reserved fields.
        ///
        /// ## Safety
        ///
        /// This function is unsafe because it's possible to violate memory
        /// safety through it, e.g. by disabling paging.
        #[inline]
        pub unsafe fn update<F>(f: F)
        where
            F: FnOnce(&mut Cr0Flags),
        {
            let mut flags = Self::read();
            f(&mut flags);
            unsafe {
                Self::write(flags);
            }
        }
    }

    impl Cr2 {
        /// Read the current page fault linear address from the CR2 register.
        ///
        /// # Errors
        ///
        /// This method returns a [`VirtAddrNotValid`] error if the CR2 register contains a
        /// non-canonical address. Call [`Cr2::read_raw`] to handle such cases.
        #[inline]
        pub fn read() -> Result<VirtAddr, VirtAddrNotValid> {
            VirtAddr::try_new(Self::read_raw())
        }

        /// Read the current page fault linear address from the CR2 register as a raw `u64`.
        #[inline]
        pub fn read_raw() -> u64 {
            let value: u64;

            unsafe {
                asm!("mov {}, cr2", out(reg) value, options(nomem, nostack, preserves_flags));
            }

            value
        }
    }

    impl Cr3 {
        /// Read the current P4 table address from the CR3 register.
        #[inline]
        pub fn read() -> (PhysFrame, Cr3Flags) {
            let (frame, value) = Cr3::read_raw();
            let flags = Cr3Flags::from_bits_truncate(value.into());
            (frame, flags)
        }

        /// Read the current P4 table address from the CR3 register
        #[inline]
        pub fn read_raw() -> (PhysFrame, u16) {
            let value: u64;

            unsafe {
                asm!("mov {}, cr3", out(reg) value, options(nomem, nostack, preserves_flags));
            }

            let addr = PhysAddr::new(value & 0x_000f_ffff_ffff_f000);
            let frame = PhysFrame::containing_address(addr);
            (frame, (value & 0xFFF) as u16)
        }

        /// Read the current P4 table address from the CR3 register along with PCID.
        /// The correct functioning of this requires CR4.PCIDE = 1.
        /// See [`Cr4Flags::PCID`]
        #[inline]
        pub fn read_pcid() -> (PhysFrame, Pcid) {
            let (frame, value) = Cr3::read_raw();
            (frame, Pcid::new(value).unwrap())
        }

        /// Write a new P4 table address into the CR3 register.
        ///
        /// ## Safety
        ///
        /// Changing the level 4 page table is unsafe, because it's possible to violate memory safety by
        /// changing the page mapping.
        #[inline]
        pub unsafe fn write(frame: PhysFrame, flags: Cr3Flags) {
            unsafe {
                Cr3::write_raw_impl(false, frame, flags.bits() as u16);
            }
        }

        /// Write a new P4 table address into the CR3 register.
        ///
        /// ## Safety
        ///
        /// Changing the level 4 page table is unsafe, because it's possible to violate memory safety by
        /// changing the page mapping.
        /// [`Cr4Flags::PCID`] must be set before calling this method.
        #[inline]
        pub unsafe fn write_pcid(frame: PhysFrame, pcid: Pcid) {
            unsafe {
                Cr3::write_raw_impl(false, frame, pcid.value());
            }
        }

        /// Write a new P4 table address into the CR3 register without flushing existing TLB entries for
        /// the PCID.
        ///
        /// ## Safety
        ///
        /// Changing the level 4 page table is unsafe, because it's possible to violate memory safety by
        /// changing the page mapping.
        /// [`Cr4Flags::PCID`] must be set before calling this method.
        #[inline]
        pub unsafe fn write_pcid_no_flush(frame: PhysFrame, pcid: Pcid) {
            unsafe {
                Cr3::write_raw_impl(true, frame, pcid.value());
            }
        }

        /// Write a new P4 table address into the CR3 register.
        ///
        /// ## Safety
        ///
        /// Changing the level 4 page table is unsafe, because it's possible to violate memory safety by
        /// changing the page mapping.
        #[inline]
        pub unsafe fn write_raw(frame: PhysFrame, val: u16) {
            unsafe { Self::write_raw_impl(false, frame, val) }
        }

        #[inline]
        unsafe fn write_raw_impl(top_bit: bool, frame: PhysFrame, val: u16) {
            let addr = frame.start_address();
            let value = ((top_bit as u64) << 63) | addr.as_u64() | val as u64;

            unsafe {
                asm!("mov cr3, {}", in(reg) value, options(nostack, preserves_flags));
            }
        }

        /// Update the P4 table address in the CR3 register.
        ///
        /// ## Safety
        ///
        /// Changing the level 4 page table is unsafe, because it's possible to violate memory safety by
        /// changing the page mapping.
        #[inline]
        pub unsafe fn update<F>(f: F)
        where
            F: FnOnce(&mut PhysFrame, &mut Cr3Flags),
        {
            let (mut frame, mut flags) = Self::read();
            f(&mut frame, &mut flags);
            unsafe {
                Self::write(frame, flags);
            }
        }

        /// Updates the P4 table address in the CR3 register.
        ///
        /// ## Safety
        ///
        /// Changing the level 4 page table is unsafe, because it's possible to violate memory safety by
        /// changing the page mapping.
        /// [`Cr4Flags::PCID`] must be set before calling this method.
        #[inline]
        pub unsafe fn update_pcid<F>(f: F)
        where
            F: FnOnce(&mut PhysFrame, &mut Pcid),
        {
            let (mut frame, mut pcid) = Self::read_pcid();
            f(&mut frame, &mut pcid);
            unsafe {
                Self::write_pcid(frame, pcid);
            }
        }

        /// Updates the P4 table address in the CR3 register without flushing existing TLB entries for
        /// the PCID.
        ///
        /// ## Safety
        ///
        /// Changing the level 4 page table is unsafe, because it's possible to violate memory safety by
        /// changing the page mapping.
        /// [`Cr4Flags::PCID`] must be set before calling this method.
        #[inline]
        pub unsafe fn update_pcid_no_flush<F>(f: F)
        where
            F: FnOnce(&mut PhysFrame, &mut Pcid),
        {
            let (mut frame, mut pcid) = Self::read_pcid();
            f(&mut frame, &mut pcid);
            unsafe {
                Self::write_pcid_no_flush(frame, pcid);
            }
        }
    }

    impl Cr4 {
        /// Read the current set of CR4 flags.
        #[inline]
        pub fn read() -> Cr4Flags {
            Cr4Flags::from_bits_truncate(Self::read_raw())
        }

        /// Read the current raw CR4 value.
        #[inline]
        pub fn read_raw() -> u64 {
            let value: u64;

            unsafe {
                asm!("mov {}, cr4", out(reg) value, options(nomem, nostack, preserves_flags));
            }

            value
        }

        /// Write CR4 flags.
        ///
        /// Preserves the value of reserved fields.
        ///
        /// ## Safety
        ///
        /// This function is unsafe because it's possible to violate memory
        /// safety through it, e.g. by overwriting the physical address extension
        /// flag.
        #[inline]
        pub unsafe fn write(flags: Cr4Flags) {
            let old_value = Self::read_raw();
            let reserved = old_value & !(Cr4Flags::all().bits());
            let new_value = reserved | flags.bits();

            unsafe {
                Self::write_raw(new_value);
            }
        }

        /// Write raw CR4 flags.
        ///
        /// Does _not_ preserve any values, including reserved fields.
        ///
        /// ## Safety
        ///
        /// This function is unsafe because it's possible to violate memory
        /// safety through it, e.g. by overwriting the physical address extension
        /// flag.
        #[inline]
        pub unsafe fn write_raw(value: u64) {
            unsafe {
                asm!("mov cr4, {}", in(reg) value, options(nostack, preserves_flags));
            }
        }

        /// Updates CR4 flags.
        ///
        /// Preserves the value of reserved fields.
        ///
        /// ## Safety
        ///
        /// This function is unsafe because it's possible to violate memory
        /// safety through it, e.g. by overwriting the physical address extension
        /// flag.
        #[inline]
        pub unsafe fn update<F>(f: F)
        where
            F: FnOnce(&mut Cr4Flags),
        {
            let mut flags = Self::read();
            f(&mut flags);
            unsafe {
                Self::write(flags);
            }
        }
    }

    impl Cr8 {
        /// Read the current priority class in CR8.
        #[inline]
        pub fn read() -> Option<PriorityClass> {
            PriorityClass::new(Self::read_raw() as u8)
        }

        /// Read the current raw CR8 value.
        #[inline]
        pub fn read_raw() -> u64 {
            let value: u64;

            unsafe {
                asm!("mov {}, cr8", out(reg) value, options(nomem, nostack, preserves_flags));
            }

            value
        }

        /// Write the priority class to CR8.
        #[inline]
        pub fn write(priority_class: Option<PriorityClass>) {
            let value = priority_class.map_or(0, |pc| pc as u64);
            Self::write_raw(value);
        }

        /// Write to CR8.
        #[inline]
        pub fn write_raw(value: u64) {
            unsafe {
                asm!("mov cr8, {}", in(reg) value, options(nomem, nostack, preserves_flags));
            }
        }

        /// Updates the priority class in CR8.
        #[inline]
        pub fn update<F>(f: F)
        where
            F: FnOnce(&mut Option<PriorityClass>),
        {
            let mut priority_class = Self::read();
            f(&mut priority_class);
            Self::write(priority_class);
        }
    }
}
