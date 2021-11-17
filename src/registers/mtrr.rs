//! Functions to read and write memory typing registers.

use crate::addr::PhysAddr;
use crate::registers::model_specific::Msr;
use crate::structures::paging::frame::PhysFrame;
use crate::structures::paging::frame::PhysFrameRange;
use bitflags::bitflags;
use core::convert::TryFrom;
use core::convert::TryInto;

/// Read only register describing the level of MTRR support
#[derive(Debug)]
pub struct MTRRcap;

#[allow(dead_code)]
/// Fixed range MTRR address with memory type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FixMemRange {
    /// Address range being mem typed
    pub range: PhysFrameRange,
    /// Memory type
    pub memory_type: MTRRtype,
}

impl FixMemRange {
    /// Creates a new mem range struct describing memory typing
    pub fn new(start: u64, end: u64, memory_type: MTRRtype) -> Self {
        let start = PhysFrame::from_start_address(PhysAddr::new(start)).unwrap();
        let end = PhysFrame::from_start_address(PhysAddr::new(end + 1)).unwrap();
        Self {
            memory_type,
            range: PhysFrameRange {
                start,
                end,
            },
        }
    }
}

/// Return type for reading a fixed memory range MTRR
pub type FixMemRangeReg = [FixMemRange; 8];

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(u8)]
/// Memory types
pub enum MTRRtype {
    /// All accesses are uncacheable. Write combining is not allowed. Speculative accesses are not allowed.
    Uncachable = 0x0,
    /// All accesses are uncacheable. Write combining is allowed. Speculative reads are allowed.
    WriteCombining = 0x1,
    /// Reads allocate cache lines on a cache miss.
    /// Cache lines are not allocated on a write miss. Write hits update the cache and main memory.
    Writethrough = 0x4,
    /// Reads allocate cache lines on a cache miss.
    /// All writes update main memory. Cache lines are not allocated on a write miss. Write hits invalidate the cache and update main memory.
    WriteProtect = 0x5,
    /// Reads allocate cache lines on a cache miss,
    /// and can allocate to either the shared, exclusive, or modified state.
    /// Write allocate to the modified state on a cache miss.
    WriteBack = 0x6,
}

impl TryFrom<u8> for MTRRtype {
    type Error = u8;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0x0 => Ok(MTRRtype::Uncachable),
            0x1 => Ok(MTRRtype::WriteCombining),
            0x4 => Ok(MTRRtype::Writethrough),
            0x5 => Ok(MTRRtype::WriteProtect),
            0x6 => Ok(MTRRtype::WriteBack),
            _ => Err(value),
        }
    }
}

impl TryFrom<u64> for MTRRtype {
    type Error = u64;
    fn try_from(value: u64) -> Result<Self, Self::Error> {
        match value {
            0x0 => Ok(MTRRtype::Uncachable),
            0x1 => Ok(MTRRtype::WriteCombining),
            0x4 => Ok(MTRRtype::Writethrough),
            0x5 => Ok(MTRRtype::WriteProtect),
            0x6 => Ok(MTRRtype::WriteBack),
            _ => Err(value),
        }
    }
}

impl Into<u8> for MTRRtype {
    fn into(self) -> u8 {
        self as u8
    }
}

bitflags! {
    /// Flags for MTRR capabilities register
    pub struct MTRRcapFlags: u64 {
        /// Variable range register count
        const VARIABLE_RANGE_REGISTER_COUNT = 0xff;
        /// Fixed range registers
        const FIXED_RANGE_REGISTERS = 1 << 8;
        /// Write combining
        const WRITE_COMBINING = 1 << 10;
    }
}

bitflags! {
    /// Flags for default memory type register
    pub struct MTRRdefTypeFlags: u64 {
        /// Default memory type
        const TYPE = 0xff;
        /// Fixed range enable
        const FIXED_ENABLE = 1 << 10;
        /// MTRR enable bit if cleared the default memory type
        /// of fixed and variable range registers is uncachable!
        const MTRR_ENABLE = 1 << 11;
    }
}

bitflags! {
    /// Flags for the MTRRphysMask register
    pub struct MTRRphysMaskFlags: u64 {
        /// Indicates that the MTRR pair is valid (enalbed) when set to 1
        const VALID = 1 << 11;
        /// The mask value used to specify the memory range
        const PHYS_MASK = 0xffff_ffff_ff << 12;
    }
}

bitflags! {
    /// Flags for the MTRRphysBase[n] registers
    pub struct MTRRphysBaseFlags: u64 {
        /// The memory range base-address in physical-address space
        const PHYS_BASE = 0xffff_ffff_ff << 12;
        /// The memory type used to characterize the memory range
        const TYPE = 0xff;
    }
}

/// Specifies the memory-range base address of a
/// variable range memory region.
#[derive(Debug)]
pub struct MTRRphysBase0;
/// Specifies the memory-range base address of a
/// variable range memory region.
#[derive(Debug)]
pub struct MTRRphysBase1;
/// Specifies the memory-range base address of a
/// variable range memory region.
#[derive(Debug)]
pub struct MTRRphysBase2;
/// Specifies the memory-range base address of a
/// variable range memory region.
#[derive(Debug)]
pub struct MTRRphysBase3;
/// Specifies the memory-range base address of a
/// variable range memory region.
#[derive(Debug)]
pub struct MTRRphysBase4;
/// Specifies the memory-range base address of a
/// variable range memory region.
#[derive(Debug)]
pub struct MTRRphysBase5;
/// Specifies the memory-range base address of a
/// variable range memory region.
#[derive(Debug)]
pub struct MTRRphysBase6;
/// Specifies the memory-range base address of a
/// variable range memory region.
#[derive(Debug)]
pub struct MTRRphysBase7;
/// Specifies the size of a variable range memory region.
#[derive(Debug)]
pub struct MTRRphysMask0;
/// Specifies the size of a variable range memory region.
#[derive(Debug)]
pub struct MTRRphysMask1;
/// Specifies the size of a variable range memory region.
#[derive(Debug)]
pub struct MTRRphysMask2;
/// Specifies the size of a variable range memory region.
#[derive(Debug)]
pub struct MTRRphysMask3;
/// Specifies the size of a variable range memory region.
#[derive(Debug)]
pub struct MTRRphysMask4;
/// Specifies the size of a variable range memory region.
#[derive(Debug)]
pub struct MTRRphysMask5;
/// Specifies the size of a variable range memory region.
#[derive(Debug)]
pub struct MTRRphysMask6;
/// Specifies the size of a variable range memory region.
#[derive(Debug)]
pub struct MTRRphysMask7;
/// Fixed range MTRR used to characterize the first 1MB of physical memory
#[derive(Debug)]
pub struct MTRRfix64K00000;
/// Fixed range MTRR used to characterize the first 1MB of physical memory
#[derive(Debug)]
pub struct MTRRfix16K80000;
/// Fixed range MTRR used to characterize the first 1MB of physical memory
#[derive(Debug)]
pub struct MTRRfix16KA0000;
/// Fixed range MTRR used to characterize the first 1MB of physical memory
#[derive(Debug)]
pub struct MTRRfix4KC0000;
/// Fixed range MTRR used to characterize the first 1MB of physical memory
#[derive(Debug)]
pub struct MTRRfix4KC8000;
/// Fixed range MTRR used to characterize the first 1MB of physical memory
#[derive(Debug)]
pub struct MTRRfix4KD0000;
/// Fixed range MTRR used to characterize the first 1MB of physical memory
#[derive(Debug)]
pub struct MTRRfix4KD8000;
/// Fixed range MTRR used to characterize the first 1MB of physical memory
#[derive(Debug)]
pub struct MTRRfix4KE0000;
/// Fixed range MTRR used to characterize the first 1MB of physical memory
#[derive(Debug)]
pub struct MTRRfix4KE8000;
/// Fixed range MTRR used to characterize the first 1MB of physical memory
#[derive(Debug)]
pub struct MTRRfix4KF0000;
/// Fixed range MTRR used to characterize the first 1MB of physical memory
#[derive(Debug)]
pub struct MTRRfix4KF8000;
/// Sets the default memory type for physical addresses not within
/// ranges established by fixed range and variable range MTRRs.
#[derive(Debug)]
pub struct MTRRdefType;

impl MTRRdefType {
    /// The underlying model specific register.
    pub const MSR: Msr = Msr::new(0x02FF);
}

impl MTRRcap {
    /// The underlying model specific register.
    pub const MSR: Msr = Msr::new(0x00FE);
}

impl MTRRphysBase0 {
    /// THe underlying model specific register.
    pub const MSR: Msr = Msr::new(0x0200);
}

impl MTRRphysBase1 {
    /// The underlying model specific register
    pub const MSR: Msr = Msr::new(0x0202);
}

impl MTRRphysBase2 {
    /// The underlying model specific register
    pub const MSR: Msr = Msr::new(0x204);
}

impl MTRRphysBase3 {
    /// The underlying model specific register
    pub const MSR: Msr = Msr::new(0x206);
}

impl MTRRphysBase4 {
    /// The underlying model specific register
    pub const MSR: Msr = Msr::new(0x208);
}

impl MTRRphysBase5 {
    /// The underlying model specific register
    pub const MSR: Msr = Msr::new(0x20A);
}
impl MTRRphysBase6 {
    /// The underlying model specific register
    pub const MSR: Msr = Msr::new(0x20C);
}

impl MTRRphysBase7 {
    /// The underlying model specific register
    pub const MSR: Msr = Msr::new(0x20E);
}

impl MTRRphysMask0 {
    /// The underlying model specific register
    pub const MSR: Msr = Msr::new(0x201);
}

impl MTRRphysMask1 {
    /// The underlying model specific register
    pub const MSR: Msr = Msr::new(0x203);
}

impl MTRRphysMask2 {
    /// The underlying model specific register
    pub const MSR: Msr = Msr::new(0x205);
}

impl MTRRphysMask3 {
    /// The underlying model specific register
    pub const MSR: Msr = Msr::new(0x207);
}

impl MTRRphysMask4 {
    /// The underlying model specific register
    pub const MSR: Msr = Msr::new(0x209);
}

impl MTRRphysMask5 {
    /// The underlying model specific register
    pub const MSR: Msr = Msr::new(0x20B);
}

impl MTRRphysMask6 {
    /// The underlying model specific register
    pub const MSR: Msr = Msr::new(0x20D);
}

impl MTRRphysMask7 {
    /// The underlying model specific register
    pub const MSR: Msr = Msr::new(0x20F);
}

impl MTRRfix64K00000 {
    /// The underlying model specific register
    pub const MSR: Msr = Msr::new(0x250);
}

impl MTRRfix16K80000 {
    /// The underlying model specific register
    pub const MSR: Msr = Msr::new(0x258);
}

impl MTRRfix16KA0000 {
    /// The underlying model specific register
    pub const MSR: Msr = Msr::new(0x259);
}

impl MTRRfix4KC0000 {
    /// The underlying model specific register
    pub const MSR: Msr = Msr::new(0x268);
}

impl MTRRfix4KC8000 {
    /// The underlying model specific register
    pub const MSR: Msr = Msr::new(0x269);
}

impl MTRRfix4KD0000 {
    /// The underlying model specific register
    pub const MSR: Msr = Msr::new(0x26A);
}

impl MTRRfix4KD8000 {
    /// The underlying model specific register
    pub const MSR: Msr = Msr::new(0x26B);
}

impl MTRRfix4KE0000 {
    /// The underlying model specific register
    pub const MSR: Msr = Msr::new(0x26C);
}

impl MTRRfix4KE8000 {
    /// The underlying model specific register
    pub const MSR: Msr = Msr::new(0x26D);
}

impl MTRRfix4KF0000 {
    /// The underlying model specific register
    pub const MSR: Msr = Msr::new(0x26E);
}

impl MTRRfix4KF8000 {
    /// The underlying model specific register
    pub const MSR: Msr = Msr::new(0x26F);
}

#[cfg(feature = "instructions")]
mod x86_64 {

    use super::*;
    impl MTRRcap {
        /// Read the current raw MTRRcap flags.
        #[inline]
        pub fn read_raw() -> u64 {
            unsafe { Self::MSR.read() }
        }

        /// Write the MTRRcap flags.
        ///
        /// Does not preserve any bits, including reserved fields.
        ///
        /// ## Safety
        ///
        /// Unsafe because it's possible to
        /// break memory safety with wrong flags
        #[inline]
        pub unsafe fn write_raw(flags: u64) {
            let mut msr = Self::MSR;
            msr.write(flags);
        }

        /// Write the MTRRcap flags, preserving reserved values.
        ///
        /// Preserves the value of reserved fields.
        ///
        /// ## Safety
        ///
        /// Unsafe because it's possible to break memory
        /// safety with wrong flags
        #[inline]
        pub unsafe fn write(flags: MTRRcapFlags) {
            let old_value = Self::read_raw();
            let reserved = old_value & !(MTRRcapFlags::all().bits());
            let new_value = reserved | flags.bits();

            Self::write_raw(new_value);
        }

        /// Read the current MTRRcap flags.
        #[inline]
        pub fn read() -> MTRRcapFlags {
            MTRRcapFlags::from_bits_truncate(Self::read_raw())
        }
    }

    impl MTRRphysBase0 {
        /// Read the current raw MTRRphysBase flags.
        #[inline]
        pub fn read_raw() -> u64 {
            unsafe { Self::MSR.read() }
        }

        /// Write the MTRRphysBase flags.
        ///
        /// Does not preserve any bits, including reserved fields.
        ///
        /// ## Safety
        ///
        /// Unsafe because it's possible to
        /// break memory safety with wrong flags
        #[inline]
        pub unsafe fn write_raw(flags: u64) {
            let mut msr = Self::MSR;
            msr.write(flags);
        }

        /// Write the MTRRphysBase flags, preserving reserved values.
        ///
        /// Preserves the value of reserved fields.
        ///
        /// ## Safety
        ///
        /// Unsafe because it's possible to break memory
        /// safety with wrong flags
        #[inline]
        pub unsafe fn write(flags: MTRRphysBaseFlags) {
            let old_value = Self::read_raw();
            let reserved = old_value & !(MTRRphysBaseFlags::all().bits());
            let new_value = reserved | flags.bits();

            Self::write_raw(new_value);
        }

        /// Read the current MTRRphysBase flags.
        #[inline]
        pub fn read() -> MTRRphysBaseFlags {
            MTRRphysBaseFlags::from_bits_truncate(Self::read_raw())
        }
    }

    impl MTRRphysBase1 {
        /// Read the current raw MTRRphysBase flags.
        #[inline]
        pub fn read_raw() -> u64 {
            unsafe { Self::MSR.read() }
        }

        /// Write the MTRRphysBase flags.
        ///
        /// Does not preserve any bits, including reserved fields.
        ///
        /// ## Safety
        ///
        /// Unsafe because it's possible to
        /// break memory safety with wrong flags
        #[inline]
        pub unsafe fn write_raw(flags: u64) {
            let mut msr = Self::MSR;
            msr.write(flags);
        }

        /// Write the MTRRphysBase flags, preserving reserved values.
        ///
        /// Preserves the value of reserved fields.
        ///
        /// ## Safety
        ///
        /// Unsafe because it's possible to break memory
        /// safety with wrong flags
        #[inline]
        pub unsafe fn write(flags: MTRRphysBaseFlags) {
            let old_value = Self::read_raw();
            let reserved = old_value & !(MTRRphysBaseFlags::all().bits());
            let new_value = reserved | flags.bits();

            Self::write_raw(new_value);
        }

        /// Read the current MTRRphysBase flags.
        #[inline]
        pub fn read() -> MTRRphysBaseFlags {
            MTRRphysBaseFlags::from_bits_truncate(Self::read_raw())
        }
    }

    impl MTRRphysBase2 {
        /// Read the current raw MTRRphysBase flags.
        #[inline]
        pub fn read_raw() -> u64 {
            unsafe { Self::MSR.read() }
        }

        /// Write the MTRRphysBase flags.
        ///
        /// Does not preserve any bits, including reserved fields.
        ///
        /// ## Safety
        ///
        /// Unsafe because it's possible to
        /// break memory safety with wrong flags
        #[inline]
        pub unsafe fn write_raw(flags: u64) {
            let mut msr = Self::MSR;
            msr.write(flags);
        }

        /// Write the MTRRphysBase flags, preserving reserved values.
        ///
        /// Preserves the value of reserved fields.
        ///
        /// ## Safety
        ///
        /// Unsafe because it's possible to break memory
        /// safety with wrong flags
        #[inline]
        pub unsafe fn write(flags: MTRRphysBaseFlags) {
            let old_value = Self::read_raw();
            let reserved = old_value & !(MTRRphysBaseFlags::all().bits());
            let new_value = reserved | flags.bits();

            Self::write_raw(new_value);
        }

        /// Read the current MTRRphysBase flags.
        #[inline]
        pub fn read() -> MTRRphysBaseFlags {
            MTRRphysBaseFlags::from_bits_truncate(Self::read_raw())
        }
    }

    impl MTRRphysBase3 {
        /// Read the current raw MTRRphysBase flags.
        #[inline]
        pub fn read_raw() -> u64 {
            unsafe { Self::MSR.read() }
        }

        /// Write the MTRRphysBase flags.
        ///
        /// Does not preserve any bits, including reserved fields.
        ///
        /// ## Safety
        ///
        /// Unsafe because it's possible to
        /// break memory safety with wrong flags
        #[inline]
        pub unsafe fn write_raw(flags: u64) {
            let mut msr = Self::MSR;
            msr.write(flags);
        }

        /// Write the MTRRphysBase flags, preserving reserved values.
        ///
        /// Preserves the value of reserved fields.
        ///
        /// ## Safety
        ///
        /// Unsafe because it's possible to break memory
        /// safety with wrong flags
        #[inline]
        pub unsafe fn write(flags: MTRRphysBaseFlags) {
            let old_value = Self::read_raw();
            let reserved = old_value & !(MTRRphysBaseFlags::all().bits());
            let new_value = reserved | flags.bits();

            Self::write_raw(new_value);
        }

        /// Read the current MTRRphysBase flags.
        #[inline]
        pub fn read() -> MTRRphysBaseFlags {
            MTRRphysBaseFlags::from_bits_truncate(Self::read_raw())
        }
    }

    impl MTRRphysBase4 {
        /// Read the current raw MTRRphysBase flags.
        #[inline]
        pub fn read_raw() -> u64 {
            unsafe { Self::MSR.read() }
        }

        /// Write the MTRRphysBase flags.
        ///
        /// Does not preserve any bits, including reserved fields.
        ///
        /// ## Safety
        ///
        /// Unsafe because it's possible to
        /// break memory safety with wrong flags
        #[inline]
        pub unsafe fn write_raw(flags: u64) {
            let mut msr = Self::MSR;
            msr.write(flags);
        }

        /// Write the MTRRphysBase flags, preserving reserved values.
        ///
        /// Preserves the value of reserved fields.
        ///
        /// ## Safety
        ///
        /// Unsafe because it's possible to break memory
        /// safety with wrong flags
        #[inline]
        pub unsafe fn write(flags: MTRRphysBaseFlags) {
            let old_value = Self::read_raw();
            let reserved = old_value & !(MTRRphysBaseFlags::all().bits());
            let new_value = reserved | flags.bits();

            Self::write_raw(new_value);
        }

        /// Read the current MTRRphysBase flags.
        #[inline]
        pub fn read() -> MTRRphysBaseFlags {
            MTRRphysBaseFlags::from_bits_truncate(Self::read_raw())
        }
    }

    impl MTRRphysBase5 {
        /// Read the current raw MTRRphysBase flags.
        #[inline]
        pub fn read_raw() -> u64 {
            unsafe { Self::MSR.read() }
        }

        /// Write the MTRRphysBase flags.
        ///
        /// Does not preserve any bits, including reserved fields.
        ///
        /// ## Safety
        ///
        /// Unsafe because it's possible to
        /// break memory safety with wrong flags
        #[inline]
        pub unsafe fn write_raw(flags: u64) {
            let mut msr = Self::MSR;
            msr.write(flags);
        }

        /// Write the MTRRphysBase flags, preserving reserved values.
        ///
        /// Preserves the value of reserved fields.
        ///
        /// ## Safety
        ///
        /// Unsafe because it's possible to break memory
        /// safety with wrong flags
        #[inline]
        pub unsafe fn write(flags: MTRRphysBaseFlags) {
            let old_value = Self::read_raw();
            let reserved = old_value & !(MTRRphysBaseFlags::all().bits());
            let new_value = reserved | flags.bits();

            Self::write_raw(new_value);
        }

        /// Read the current MTRRphysBase flags.
        #[inline]
        pub fn read() -> MTRRphysBaseFlags {
            MTRRphysBaseFlags::from_bits_truncate(Self::read_raw())
        }
    }

    impl MTRRphysBase6 {
        /// Read the current raw MTRRphysBase flags.
        #[inline]
        pub fn read_raw() -> u64 {
            unsafe { Self::MSR.read() }
        }

        /// Write the MTRRphysBase flags.
        ///
        /// Does not preserve any bits, including reserved fields.
        ///
        /// ## Safety
        ///
        /// Unsafe because it's possible to
        /// break memory safety with wrong flags
        #[inline]
        pub unsafe fn write_raw(flags: u64) {
            let mut msr = Self::MSR;
            msr.write(flags);
        }

        /// Write the MTRRphysBase flags, preserving reserved values.
        ///
        /// Preserves the value of reserved fields.
        ///
        /// ## Safety
        ///
        /// Unsafe because it's possible to break memory
        /// safety with wrong flags
        #[inline]
        pub unsafe fn write(flags: MTRRphysBaseFlags) {
            let old_value = Self::read_raw();
            let reserved = old_value & !(MTRRphysBaseFlags::all().bits());
            let new_value = reserved | flags.bits();

            Self::write_raw(new_value);
        }

        /// Read the current MTRRphysBase flags.
        #[inline]
        pub fn read() -> MTRRphysBaseFlags {
            MTRRphysBaseFlags::from_bits_truncate(Self::read_raw())
        }
    }

    impl MTRRphysBase7 {
        /// Read the current raw MTRRphysBase flags.
        #[inline]
        pub fn read_raw() -> u64 {
            unsafe { Self::MSR.read() }
        }

        /// Write the MTRRphysBase flags.
        ///
        /// Does not preserve any bits, including reserved fields.
        ///
        /// ## Safety
        ///
        /// Unsafe because it's possible to
        /// break memory safety with wrong flags
        #[inline]
        pub unsafe fn write_raw(flags: u64) {
            let mut msr = Self::MSR;
            msr.write(flags);
        }

        /// Write the MTRRphysBase flags, preserving reserved values.
        ///
        /// Preserves the value of reserved fields.
        ///
        /// ## Safety
        ///
        /// Unsafe because it's possible to break memory
        /// safety with wrong flags
        #[inline]
        pub unsafe fn write(flags: MTRRphysBaseFlags) {
            let old_value = Self::read_raw();
            let reserved = old_value & !(MTRRphysBaseFlags::all().bits());
            let new_value = reserved | flags.bits();

            Self::write_raw(new_value);
        }

        /// Read the current MTRRphysBase flags.
        #[inline]
        pub fn read() -> MTRRphysBaseFlags {
            MTRRphysBaseFlags::from_bits_truncate(Self::read_raw())
        }
    }

    impl MTRRphysMask0 {
        /// Read the current raw MTRRphysMask flags.
        #[inline]
        pub fn read_raw() -> u64 {
            unsafe { Self::MSR.read() }
        }

        /// Write the MTRRphysMask flags.
        ///
        /// Does not preserve any bits, including reserved fields.
        ///
        /// ## Safety
        ///
        /// Unsafe because it's possible to
        /// break memory safety with wrong flags
        #[inline]
        pub unsafe fn write_raw(flags: u64) {
            let mut msr = Self::MSR;
            msr.write(flags);
        }

        /// Write the MTRRphysMask flags, preserving reserved values.
        ///
        /// Preserves the value of reserved fields.
        ///
        /// ## Safety
        ///
        /// Unsafe because it's possible to break memory
        /// safety with wrong flags
        #[inline]
        pub unsafe fn write(flags: MTRRphysMaskFlags) {
            let old_value = Self::read_raw();
            let reserved = old_value & !(MTRRphysMaskFlags::all().bits());
            let new_value = reserved | flags.bits();

            Self::write_raw(new_value);
        }

        /// Read the current MTRRphysMask flags.
        #[inline]
        pub fn read() -> MTRRphysMaskFlags {
            MTRRphysMaskFlags::from_bits_truncate(Self::read_raw())
        }
    }

    impl MTRRphysMask1 {
        /// Read the current raw MTRRphysMask flags.
        #[inline]
        pub fn read_raw() -> u64 {
            unsafe { Self::MSR.read() }
        }

        /// Write the MTRRphysMask flags.
        ///
        /// Does not preserve any bits, including reserved fields.
        ///
        /// ## Safety
        ///
        /// Unsafe because it's possible to
        /// break memory safety with wrong flags
        #[inline]
        pub unsafe fn write_raw(flags: u64) {
            let mut msr = Self::MSR;
            msr.write(flags);
        }

        /// Write the MTRRphysMask flags, preserving reserved values.
        ///
        /// Preserves the value of reserved fields.
        ///
        /// ## Safety
        ///
        /// Unsafe because it's possible to break memory
        /// safety with wrong flags
        #[inline]
        pub unsafe fn write(flags: MTRRphysMaskFlags) {
            let old_value = Self::read_raw();
            let reserved = old_value & !(MTRRphysMaskFlags::all().bits());
            let new_value = reserved | flags.bits();

            Self::write_raw(new_value);
        }

        /// Read the current MTRRphysMask flags.
        #[inline]
        pub fn read() -> MTRRphysMaskFlags {
            MTRRphysMaskFlags::from_bits_truncate(Self::read_raw())
        }
    }

    impl MTRRphysMask2 {
        /// Read the current raw MTRRphysMask flags.
        #[inline]
        pub fn read_raw() -> u64 {
            unsafe { Self::MSR.read() }
        }

        /// Write the MTRRphysMask flags.
        ///
        /// Does not preserve any bits, including reserved fields.
        ///
        /// ## Safety
        ///
        /// Unsafe because it's possible to
        /// break memory safety with wrong flags
        #[inline]
        pub unsafe fn write_raw(flags: u64) {
            let mut msr = Self::MSR;
            msr.write(flags);
        }

        /// Write the MTRRphysMask flags, preserving reserved values.
        ///
        /// Preserves the value of reserved fields.
        ///
        /// ## Safety
        ///
        /// Unsafe because it's possible to break memory
        /// safety with wrong flags
        #[inline]
        pub unsafe fn write(flags: MTRRphysMaskFlags) {
            let old_value = Self::read_raw();
            let reserved = old_value & !(MTRRphysMaskFlags::all().bits());
            let new_value = reserved | flags.bits();

            Self::write_raw(new_value);
        }

        /// Read the current MTRRphysMask flags.
        #[inline]
        pub fn read() -> MTRRphysMaskFlags {
            MTRRphysMaskFlags::from_bits_truncate(Self::read_raw())
        }
    }

    impl MTRRphysMask3 {
        /// Read the current raw MTRRphysMask flags.
        #[inline]
        pub fn read_raw() -> u64 {
            unsafe { Self::MSR.read() }
        }

        /// Write the MTRRphysMask flags.
        ///
        /// Does not preserve any bits, including reserved fields.
        ///
        /// ## Safety
        ///
        /// Unsafe because it's possible to
        /// break memory safety with wrong flags
        #[inline]
        pub unsafe fn write_raw(flags: u64) {
            let mut msr = Self::MSR;
            msr.write(flags);
        }

        /// Write the MTRRphysMask flags, preserving reserved values.
        ///
        /// Preserves the value of reserved fields.
        ///
        /// ## Safety
        ///
        /// Unsafe because it's possible to break memory
        /// safety with wrong flags
        #[inline]
        pub unsafe fn write(flags: MTRRphysMaskFlags) {
            let old_value = Self::read_raw();
            let reserved = old_value & !(MTRRphysMaskFlags::all().bits());
            let new_value = reserved | flags.bits();

            Self::write_raw(new_value);
        }

        /// Read the current MTRRphysMask flags.
        #[inline]
        pub fn read() -> MTRRphysMaskFlags {
            MTRRphysMaskFlags::from_bits_truncate(Self::read_raw())
        }
    }

    impl MTRRphysMask4 {
        /// Read the current raw MTRRphysMask flags.
        #[inline]
        pub fn read_raw() -> u64 {
            unsafe { Self::MSR.read() }
        }

        /// Write the MTRRphysMask flags.
        ///
        /// Does not preserve any bits, including reserved fields.
        ///
        /// ## Safety
        ///
        /// Unsafe because it's possible to
        /// break memory safety with wrong flags
        #[inline]
        pub unsafe fn write_raw(flags: u64) {
            let mut msr = Self::MSR;
            msr.write(flags);
        }

        /// Write the MTRRphysMask flags, preserving reserved values.
        ///
        /// Preserves the value of reserved fields.
        ///
        /// ## Safety
        ///
        /// Unsafe because it's possible to break memory
        /// safety with wrong flags
        #[inline]
        pub unsafe fn write(flags: MTRRphysMaskFlags) {
            let old_value = Self::read_raw();
            let reserved = old_value & !(MTRRphysMaskFlags::all().bits());
            let new_value = reserved | flags.bits();

            Self::write_raw(new_value);
        }

        /// Read the current MTRRphysMask flags.
        #[inline]
        pub fn read() -> MTRRphysMaskFlags {
            MTRRphysMaskFlags::from_bits_truncate(Self::read_raw())
        }
    }

    impl MTRRphysMask5 {
        /// Read the current raw MTRRphysMask flags.
        #[inline]
        pub fn read_raw() -> u64 {
            unsafe { Self::MSR.read() }
        }

        /// Write the MTRRphysMask flags.
        ///
        /// Does not preserve any bits, including reserved fields.
        ///
        /// ## Safety
        ///
        /// Unsafe because it's possible to
        /// break memory safety with wrong flags
        #[inline]
        pub unsafe fn write_raw(flags: u64) {
            let mut msr = Self::MSR;
            msr.write(flags);
        }

        /// Write the MTRRphysMask flags, preserving reserved values.
        ///
        /// Preserves the value of reserved fields.
        ///
        /// ## Safety
        ///
        /// Unsafe because it's possible to break memory
        /// safety with wrong flags
        #[inline]
        pub unsafe fn write(flags: MTRRphysMaskFlags) {
            let old_value = Self::read_raw();
            let reserved = old_value & !(MTRRphysMaskFlags::all().bits());
            let new_value = reserved | flags.bits();

            Self::write_raw(new_value);
        }

        /// Read the current MTRRphysMask flags.
        #[inline]
        pub fn read() -> MTRRphysMaskFlags {
            MTRRphysMaskFlags::from_bits_truncate(Self::read_raw())
        }
    }

    impl MTRRphysMask6 {
        /// Read the current raw MTRRphysMask flags.
        #[inline]
        pub fn read_raw() -> u64 {
            unsafe { Self::MSR.read() }
        }

        /// Write the MTRRphysMask flags.
        ///
        /// Does not preserve any bits, including reserved fields.
        ///
        /// ## Safety
        ///
        /// Unsafe because it's possible to
        /// break memory safety with wrong flags
        #[inline]
        pub unsafe fn write_raw(flags: u64) {
            let mut msr = Self::MSR;
            msr.write(flags);
        }

        /// Write the MTRRphysMask flags, preserving reserved values.
        ///
        /// Preserves the value of reserved fields.
        ///
        /// ## Safety
        ///
        /// Unsafe because it's possible to break memory
        /// safety with wrong flags
        #[inline]
        pub unsafe fn write(flags: MTRRphysMaskFlags) {
            let old_value = Self::read_raw();
            let reserved = old_value & !(MTRRphysMaskFlags::all().bits());
            let new_value = reserved | flags.bits();

            Self::write_raw(new_value);
        }

        /// Read the current MTRRphysMask flags.
        #[inline]
        pub fn read() -> MTRRphysMaskFlags {
            MTRRphysMaskFlags::from_bits_truncate(Self::read_raw())
        }
    }

    impl MTRRphysMask7 {
        /// Read the current raw MTRRphysMask flags.
        #[inline]
        pub fn read_raw() -> u64 {
            unsafe { Self::MSR.read() }
        }

        /// Write the MTRRphysMask flags.
        ///
        /// Does not preserve any bits, including reserved fields.
        ///
        /// ## Safety
        ///
        /// Unsafe because it's possible to
        /// break memory safety with wrong flags
        #[inline]
        pub unsafe fn write_raw(flags: u64) {
            let mut msr = Self::MSR;
            msr.write(flags);
        }

        /// Write the MTRRphysMask flags, preserving reserved values.
        ///
        /// Preserves the value of reserved fields.
        ///
        /// ## Safety
        ///
        /// Unsafe because it's possible to break memory
        /// safety with wrong flags
        #[inline]
        pub unsafe fn write(flags: MTRRphysMaskFlags) {
            let old_value = Self::read_raw();
            let reserved = old_value & !(MTRRphysMaskFlags::all().bits());
            let new_value = reserved | flags.bits();

            Self::write_raw(new_value);
        }

        /// Read the current MTRRphysMask flags.
        #[inline]
        pub fn read() -> MTRRphysMaskFlags {
            MTRRphysMaskFlags::from_bits_truncate(Self::read_raw())
        }
    }

    impl MTRRdefType {
        /// Read the current raw MTRRdefType flags.
        #[inline]
        pub fn read_raw() -> u64 {
            unsafe { Self::MSR.read() }
        }

        /// Write the MTRRdefType flags.
        ///
        /// Does not preserve any bits, including reserved fields.
        ///
        /// ## Safety
        ///
        /// Unsafe because it's possible to
        /// break memory safety with wrong flags
        #[inline]
        pub unsafe fn write_raw(flags: u64) {
            let mut msr = Self::MSR;
            msr.write(flags);
        }

        /// Write the MTRRdefType flags, preserving reserved values.
        ///
        /// Preserves the value of reserved fields.
        ///
        /// ## Safety
        ///
        /// Unsafe because it's possible to break memory
        /// safety with wrong flags
        #[inline]
        pub unsafe fn write(flags: MTRRdefTypeFlags) {
            let old_value = Self::read_raw();
            let reserved = old_value & !(MTRRdefTypeFlags::all().bits());
            let new_value = reserved | flags.bits();

            Self::write_raw(new_value);
        }

        /// Read the current MTRRdefType flags.
        #[inline]
        pub fn read() -> MTRRdefTypeFlags {
            MTRRdefTypeFlags::from_bits_truncate(Self::read_raw())
        }
    }

    impl MTRRfix64K00000 {
        /// Read the raw register
        #[inline]
        pub fn read_raw() -> u64 {
            unsafe { Self::MSR.read() }
        }

        /// Writes the MTRR fixed range memory types.
        ///
        /// Does not preserve any bits, including reserved fields.
        ///
        /// ## Safety
        ///
        /// Unsafe because it's possible to
        /// break memory safety with wrong flags
        #[inline]
        pub unsafe fn write_raw(flags: u64) {
            let mut msr = Self::MSR;
            msr.write(flags);
        }

        /// Reads the MTRR fixed range memory types.
        /// The 512 Kbytes of memory spanning addresses 00_0000h to 07_FFFFh are segmented into eight
        /// 64-Kbyte ranges. A single MTRR is used to characterize this address space.
        pub fn read() -> FixMemRangeReg {
            let r = Self::read_raw();
            let one = FixMemRange::new(0x00000, 0x0FFFF, (r & 0xff).try_into().unwrap());
            let two = FixMemRange::new(
                0x10000,
                0x1FFFF,
                ((r & (0xff << 8)) >> 8).try_into().unwrap(),
            );
            let three = FixMemRange::new(
                0x20000,
                0x2FFFF,
                ((r & (0xff << 16)) >> 16).try_into().unwrap(),
            );
            let four = FixMemRange::new(
                0x30000,
                0x3FFFF,
                ((r & (0xff << 24)) >> 24).try_into().unwrap(),
            );
            let five = FixMemRange::new(
                0x40000,
                0x4FFFF,
                ((r & (0xff << 32)) >> 32).try_into().unwrap(),
            );
            let six = FixMemRange::new(
                0x50000,
                0x5FFFF,
                ((r & (0xff << 40)) >> 40).try_into().unwrap(),
            );
            let seven = FixMemRange::new(
                0x60000,
                0x6FFFF,
                ((r & (0xff << 48)) >> 48).try_into().unwrap(),
            );
            let eight = FixMemRange::new(
                0x70000,
                0x7FFFF,
                ((r & (0xff << 56)) >> 56).try_into().unwrap(),
            );
            [one, two, three, four, five, six, seven, eight]
        }
    }

    impl MTRRfix16K80000 {
        /// Reads the raw register
        #[inline]
        pub fn read_raw() -> u64 {
            unsafe { Self::MSR.read() }
        }

        /// Writes the MTRR fixed range memory types.
        ///
        /// Does not preserve any bits, including reserved fields.
        ///
        /// ## Safety
        ///
        /// Unsafe because it's possible to
        /// break memory safety with wrong flags
        #[inline]
        pub unsafe fn write_raw(flags: u64) {
            let mut msr = Self::MSR;
            msr.write(flags);
        }

        /// Reads the MTRR fixed range memory types.
        /// The 256 Kbytes of memory spanning addresses 08_0000h to 0B_FFFFh are segmented into 16 16-
        /// Kbyte ranges. Two MTRRs are used to characterize this address space.
        pub fn read() -> FixMemRangeReg {
            let r = Self::read_raw();
            let one = FixMemRange::new(0x80000, 0x83FFF, (r & 0xff).try_into().unwrap());
            let two = FixMemRange::new(
                0x84000,
                0x87FFF,
                ((r & (0xff << 8)) >> 8).try_into().unwrap(),
            );
            let three = FixMemRange::new(
                0x88000,
                0x8BFFF,
                ((r & (0xff << 16)) >> 16).try_into().unwrap(),
            );
            let four = FixMemRange::new(
                0x8C000,
                0x8FFFF,
                ((r & (0xff << 24)) >> 24).try_into().unwrap(),
            );
            let five = FixMemRange::new(
                0x90000,
                0x93FFF,
                ((r & (0xff << 32)) >> 32).try_into().unwrap(),
            );
            let six = FixMemRange::new(
                0x94000,
                0x97FFF,
                ((r & (0xff << 40)) >> 40).try_into().unwrap(),
            );
            let seven = FixMemRange::new(
                0x98000,
                0x9BFFF,
                ((r & (0xff << 48)) >> 48).try_into().unwrap(),
            );
            let eight = FixMemRange::new(
                0x9C000,
                0x9FFFF,
                ((r & (0xff << 56)) >> 56).try_into().unwrap(),
            );
            [one, two, three, four, five, six, seven, eight]
        }
    }

    impl MTRRfix16KA0000 {
        /// Reads the raw register value
        #[inline]
        pub fn read_raw() -> u64 {
            unsafe { Self::MSR.read() }
        }

        /// Writes the MTRR fixed range memory types.
        ///
        /// Does not preserve any bits, including reserved fields.
        ///
        /// ## Safety
        ///
        /// Unsafe because it's possible to
        /// break memory safety with wrong flags
        #[inline]
        pub unsafe fn write_raw(flags: u64) {
            let mut msr = Self::MSR;
            msr.write(flags);
        }

        /// Reads the MTRR fixed range memory types.
        /// The 256 Kbytes of memory spanning addresses 08_0000h to 0B_FFFFh are segmented into 16 16-
        /// Kbyte ranges. Two MTRRs are used to characterize this address space.
        pub fn read() -> FixMemRangeReg {
            let r = Self::read_raw();
            let one = FixMemRange::new(0xA0000, 0xA3FFF, (r & 0xff).try_into().unwrap());
            let two = FixMemRange::new(
                0xA4000,
                0xA7FFF,
                ((r & (0xff << 8)) >> 8).try_into().unwrap(),
            );
            let three = FixMemRange::new(
                0xA8000,
                0xABFFF,
                ((r & (0xff << 16)) >> 16).try_into().unwrap(),
            );
            let four = FixMemRange::new(
                0xAC000,
                0xAFFFF,
                ((r & (0xff << 24)) >> 24).try_into().unwrap(),
            );
            let five = FixMemRange::new(
                0xB0000,
                0xB3FFF,
                ((r & (0xff << 32)) >> 32).try_into().unwrap(),
            );
            let six = FixMemRange::new(
                0xB4000,
                0xB7FFF,
                ((r & (0xff << 40)) >> 40).try_into().unwrap(),
            );
            let seven = FixMemRange::new(
                0xB8000,
                0xBBFFF,
                ((r & (0xff << 48)) >> 48).try_into().unwrap(),
            );
            let eight = FixMemRange::new(
                0xBC000,
                0xBFFFF,
                ((r & (0xff << 56)) >> 56).try_into().unwrap(),
            );
            [one, two, three, four, five, six, seven, eight]
        }
    }

    impl MTRRfix4KC0000 {
        /// Reads the MTRR fixed range memory types.
        #[inline]
        pub fn read_raw() -> u64 {
            unsafe { Self::MSR.read() }
        }

        /// Writes the MTRR fixed range memory types.
        ///
        /// Does not preserve any bits, including reserved fields.
        ///
        /// ## Safety
        ///
        /// Unsafe because it's possible to
        /// break memory safety with wrong flags
        #[inline]
        pub unsafe fn write_raw(flags: u64) {
            let mut msr = Self::MSR;
            msr.write(flags);
        }

        /// The 256 Kbytes of memory spanning addresses 0C_0000h to 0F_FFFFh are segmented into 64 4-
        /// Kbyte ranges. Eight MTRRs are used to characterize this address space.
        pub fn read() -> FixMemRangeReg {
            let r = Self::read_raw();
            let one = FixMemRange::new(0xC0000, 0xC0FFF, (r & 0xff).try_into().unwrap());
            let two = FixMemRange::new(
                0xC1000,
                0xC1FFF,
                ((r & (0xff << 8)) >> 8).try_into().unwrap(),
            );
            let three = FixMemRange::new(
                0xC2000,
                0xC2FFF,
                ((r & (0xff << 16)) >> 16).try_into().unwrap(),
            );
            let four = FixMemRange::new(
                0xC3000,
                0xC3FFF,
                ((r & (0xff << 24)) >> 24).try_into().unwrap(),
            );
            let five = FixMemRange::new(
                0xC4000,
                0xC4FFF,
                ((r & (0xff << 32)) >> 32).try_into().unwrap(),
            );
            let six = FixMemRange::new(
                0xC5000,
                0xC5FFF,
                ((r & (0xff << 40)) >> 40).try_into().unwrap(),
            );
            let seven = FixMemRange::new(
                0xC6000,
                0xCFFF,
                ((r & (0xff << 48)) >> 48).try_into().unwrap(),
            );
            let eight = FixMemRange::new(
                0xC7000,
                0xC7FFF,
                ((r & (0xff << 56)) >> 56).try_into().unwrap(),
            );
            [one, two, three, four, five, six, seven, eight]
        }
    }

    impl MTRRfix4KC8000 {
        /// Reads the MTRR fixed range memory types.
        #[inline]
        pub fn read_raw() -> u64 {
            unsafe { Self::MSR.read() }
        }

        /// Writes the MTRR fixed range memory types.
        ///
        /// Does not preserve any bits, including reserved fields.
        ///
        /// ## Safety
        ///
        /// Unsafe because it's possible to
        /// break memory safety with wrong flags
        #[inline]
        pub unsafe fn write_raw(flags: u64) {
            let mut msr = Self::MSR;
            msr.write(flags);
        }

        /// The 256 Kbytes of memory spanning addresses 0C_0000h to 0F_FFFFh are segmented into 64 4-
        /// Kbyte ranges. Eight MTRRs are used to characterize this address space.
        pub fn read() -> FixMemRangeReg {
            let r = Self::read_raw();
            let one = FixMemRange::new(0xC8000, 0xC8FFF, (r & 0xff).try_into().unwrap());
            let two = FixMemRange::new(
                0xC9000,
                0xC9FFF,
                ((r & (0xff << 8)) >> 8).try_into().unwrap(),
            );
            let three = FixMemRange::new(
                0xCA000,
                0xCAFFF,
                ((r & (0xff << 16)) >> 16).try_into().unwrap(),
            );
            let four = FixMemRange::new(
                0xCB000,
                0xCBFFF,
                ((r & (0xff << 24)) >> 24).try_into().unwrap(),
            );
            let five = FixMemRange::new(
                0xCC000,
                0xCCFFF,
                ((r & (0xff << 32)) >> 32).try_into().unwrap(),
            );
            let six = FixMemRange::new(
                0xCD000,
                0xCDFFF,
                ((r & (0xff << 40)) >> 40).try_into().unwrap(),
            );
            let seven = FixMemRange::new(
                0xCE000,
                0xCEFFF,
                ((r & (0xff << 48)) >> 48).try_into().unwrap(),
            );
            let eight = FixMemRange::new(
                0xCF000,
                0xCFFFF,
                ((r & (0xff << 56)) >> 56).try_into().unwrap(),
            );
            [one, two, three, four, five, six, seven, eight]
        }
    }

    impl MTRRfix4KD0000 {
        /// Reads the MTRR fixed range memory types.
        #[inline]
        pub fn read_raw() -> u64 {
            unsafe { Self::MSR.read() }
        }

        /// Writes the MTRR fixed range memory types.
        ///
        /// Does not preserve any bits, including reserved fields.
        ///
        /// ## Safety
        ///
        /// Unsafe because it's possible to
        /// break memory safety with wrong flags
        #[inline]
        pub unsafe fn write_raw(flags: u64) {
            let mut msr = Self::MSR;
            msr.write(flags);
        }

        /// The 256 Kbytes of memory spanning addresses 0C_0000h to 0F_FFFFh are segmented into 64 4-
        /// Kbyte ranges. Eight MTRRs are used to characterize this address space.
        pub fn read() -> FixMemRangeReg {
            let r = Self::read_raw();
            let one = FixMemRange::new(0xD0000, 0xD0FFF, (r & 0xff).try_into().unwrap());
            let two = FixMemRange::new(
                0xD1000,
                0xD1FFF,
                ((r & (0xff << 8)) >> 8).try_into().unwrap(),
            );
            let three = FixMemRange::new(
                0xD2000,
                0xD2FFF,
                ((r & (0xff << 16)) >> 16).try_into().unwrap(),
            );
            let four = FixMemRange::new(
                0xD3000,
                0xD3FFF,
                ((r & (0xff << 24)) >> 24).try_into().unwrap(),
            );
            let five = FixMemRange::new(
                0xD4000,
                0xD4FFF,
                ((r & (0xff << 32)) >> 32).try_into().unwrap(),
            );
            let six = FixMemRange::new(
                0xD5000,
                0xD5FFF,
                ((r & (0xff << 40)) >> 40).try_into().unwrap(),
            );
            let seven = FixMemRange::new(
                0xD6000,
                0xD6FFF,
                ((r & (0xff << 48)) >> 48).try_into().unwrap(),
            );
            let eight = FixMemRange::new(
                0xD7000,
                0xD7FFF,
                ((r & (0xff << 56)) >> 56).try_into().unwrap(),
            );
            [one, two, three, four, five, six, seven, eight]
        }
    }

    impl MTRRfix4KD8000 {
        /// Reads the MTRR fixed range memory types.
        #[inline]
        pub fn read_raw() -> u64 {
            unsafe { Self::MSR.read() }
        }

        /// Writes the MTRR fixed range memory types.
        ///
        /// Does not preserve any bits, including reserved fields.
        ///
        /// ## Safety
        ///
        /// Unsafe because it's possible to
        /// break memory safety with wrong flags
        #[inline]
        pub unsafe fn write_raw(flags: u64) {
            let mut msr = Self::MSR;
            msr.write(flags);
        }

        /// The 256 Kbytes of memory spanning addresses 0C_0000h to 0F_FFFFh are segmented into 64 4-
        /// Kbyte ranges. Eight MTRRs are used to characterize this address space.
        pub fn read() -> FixMemRangeReg {
            let r = Self::read_raw();
            let one = FixMemRange::new(0xD8000, 0xD8FFF, (r & 0xff).try_into().unwrap());
            let two = FixMemRange::new(
                0xD9000,
                0xD9FFF,
                ((r & (0xff << 8)) >> 8).try_into().unwrap(),
            );
            let three = FixMemRange::new(
                0xDA000,
                0xDAFFF,
                ((r & (0xff << 16)) >> 16).try_into().unwrap(),
            );
            let four = FixMemRange::new(
                0xDB000,
                0xDBFFF,
                ((r & (0xff << 24)) >> 24).try_into().unwrap(),
            );
            let five = FixMemRange::new(
                0xDC000,
                0xDCFFF,
                ((r & (0xff << 32)) >> 32).try_into().unwrap(),
            );
            let six = FixMemRange::new(
                0xDD000,
                0xDDFFF,
                ((r & (0xff << 40)) >> 40).try_into().unwrap(),
            );
            let seven = FixMemRange::new(
                0xDE000,
                0xDEFFF,
                ((r & (0xff << 48)) >> 48).try_into().unwrap(),
            );
            let eight = FixMemRange::new(
                0xDF000,
                0xDFFFF,
                ((r & (0xff << 56)) >> 56).try_into().unwrap(),
            );
            [one, two, three, four, five, six, seven, eight]
        }
    }

    impl MTRRfix4KE0000 {
        /// Reads the MTRR fixed range memory types.
        #[inline]
        pub fn read_raw() -> u64 {
            unsafe { Self::MSR.read() }
        }

        /// Writes the MTRR fixed range memory types.
        ///
        /// Does not preserve any bits, including reserved fields.
        ///
        /// ## Safety
        ///
        /// Unsafe because it's possible to
        /// break memory safety with wrong flags
        #[inline]
        pub unsafe fn write_raw(flags: u64) {
            let mut msr = Self::MSR;
            msr.write(flags);
        }

        /// The 256 Kbytes of memory spanning addresses 0C_0000h to 0F_FFFFh are segmented into 64 4-
        /// Kbyte ranges. Eight MTRRs are used to characterize this address space.
        pub fn read() -> FixMemRangeReg {
            let r = Self::read_raw();
            let one = FixMemRange::new(0xE0000, 0xE0FFF, (r & 0xff).try_into().unwrap());
            let two = FixMemRange::new(
                0xE1000,
                0xE1FFF,
                ((r & (0xff << 8)) >> 8).try_into().unwrap(),
            );
            let three = FixMemRange::new(
                0xE2000,
                0xE2FFF,
                ((r & (0xff << 16)) >> 16).try_into().unwrap(),
            );
            let four = FixMemRange::new(
                0xE3000,
                0xE3FFF,
                ((r & (0xff << 24)) >> 24).try_into().unwrap(),
            );
            let five = FixMemRange::new(
                0xE4000,
                0xE4FFF,
                ((r & (0xff << 32)) >> 32).try_into().unwrap(),
            );
            let six = FixMemRange::new(
                0xE5000,
                0xE5FFF,
                ((r & (0xff << 40)) >> 40).try_into().unwrap(),
            );
            let seven = FixMemRange::new(
                0xE6000,
                0xE6FFF,
                ((r & (0xff << 48)) >> 48).try_into().unwrap(),
            );
            let eight = FixMemRange::new(
                0xE7000,
                0xE7FFF,
                ((r & (0xff << 56)) >> 56).try_into().unwrap(),
            );
            [one, two, three, four, five, six, seven, eight]
        }
    }

    impl MTRRfix4KE8000 {
        /// Reads the MTRR fixed range memory types.
        #[inline]
        pub fn read_raw() -> u64 {
            unsafe { Self::MSR.read() }
        }

        /// Writes the MTRR fixed range memory types.
        ///
        /// Does not preserve any bits, including reserved fields.
        ///
        /// ## Safety
        ///
        /// Unsafe because it's possible to
        /// break memory safety with wrong flags
        #[inline]
        pub unsafe fn write_raw(flags: u64) {
            let mut msr = Self::MSR;
            msr.write(flags);
        }

        /// The 256 Kbytes of memory spanning addresses 0C_0000h to 0F_FFFFh are segmented into 64 4-
        /// Kbyte ranges. Eight MTRRs are used to characterize this address space.
        pub fn read() -> FixMemRangeReg {
            let r = Self::read_raw();
            let one = FixMemRange::new(0xE8000, 0xE8FFF, (r & 0xff).try_into().unwrap());
            let two = FixMemRange::new(
                0xE9000,
                0xE9FFF,
                ((r & (0xff << 8)) >> 8).try_into().unwrap(),
            );
            let three = FixMemRange::new(
                0xEA000,
                0xEAFFF,
                ((r & (0xff << 16)) >> 16).try_into().unwrap(),
            );
            let four = FixMemRange::new(
                0xEA000,
                0xEAFFF,
                ((r & (0xff << 24)) >> 24).try_into().unwrap(),
            );
            let five = FixMemRange::new(
                0xEB000,
                0xEBFFF,
                ((r & (0xff << 32)) >> 32).try_into().unwrap(),
            );
            let six = FixMemRange::new(
                0xEC000,
                0xECFFF,
                ((r & (0xff << 40)) >> 40).try_into().unwrap(),
            );
            let seven = FixMemRange::new(
                0xED000,
                0xEDFFF,
                ((r & (0xff << 48)) >> 48).try_into().unwrap(),
            );
            let eight = FixMemRange::new(
                0xEE000,
                0xEEFFF,
                ((r & (0xff << 56)) >> 56).try_into().unwrap(),
            );
            [one, two, three, four, five, six, seven, eight]
        }
    }

    impl MTRRfix4KF0000 {
        /// Reads the MTRR fixed range memory types.
        #[inline]
        pub fn read_raw() -> u64 {
            unsafe { Self::MSR.read() }
        }

        /// Writes the MTRR fixed range memory types.
        ///
        /// Does not preserve any bits, including reserved fields.
        ///
        /// ## Safety
        ///
        /// Unsafe because it's possible to
        /// break memory safety with wrong flags
        #[inline]
        pub unsafe fn write_raw(flags: u64) {
            let mut msr = Self::MSR;
            msr.write(flags);
        }

        /// The 256 Kbytes of memory spanning addresses 0C_0000h to 0F_FFFFh are segmented into 64 4-
        /// Kbyte ranges. Eight MTRRs are used to characterize this address space.
        pub fn read() -> FixMemRangeReg {
            let r = Self::read_raw();
            let one = FixMemRange::new(0xF0000, 0xF0FFF, (r & 0xff).try_into().unwrap());
            let two = FixMemRange::new(
                0xF1000,
                0xF1FFF,
                ((r & (0xff << 8)) >> 8).try_into().unwrap(),
            );
            let three = FixMemRange::new(
                0xF2000,
                0xF2FFF,
                ((r & (0xff << 16)) >> 16).try_into().unwrap(),
            );
            let four = FixMemRange::new(
                0xF3000,
                0xF3FFF,
                ((r & (0xff << 24)) >> 24).try_into().unwrap(),
            );
            let five = FixMemRange::new(
                0xF4000,
                0xF4FFF,
                ((r & (0xff << 32)) >> 32).try_into().unwrap(),
            );
            let six = FixMemRange::new(
                0xF5000,
                0xF5FFF,
                ((r & (0xff << 40)) >> 40).try_into().unwrap(),
            );
            let seven = FixMemRange::new(
                0xF6000,
                0xF6FFF,
                ((r & (0xff << 48)) >> 48).try_into().unwrap(),
            );
            let eight = FixMemRange::new(
                0xF7000,
                0xF7FFF,
                ((r & (0xff << 56)) >> 56).try_into().unwrap(),
            );
            [one, two, three, four, five, six, seven, eight]
        }
    }

    impl MTRRfix4KF8000 {
        /// Reads the MTRR fixed range memory types.
        #[inline]
        pub fn read_raw() -> u64 {
            unsafe { Self::MSR.read() }
        }

        /// Writes the MTRR fixed range memory types.
        ///
        /// Does not preserve any bits, including reserved fields.
        ///
        /// ## Safety
        ///
        /// Unsafe because it's possible to
        /// break memory safety with wrong flags
        #[inline]
        pub unsafe fn write_raw(flags: u64) {
            let mut msr = Self::MSR;
            msr.write(flags);
        }

        /// The 256 Kbytes of memory spanning addresses 0C_0000h to 0F_FFFFh are segmented into 64 4-
        /// Kbyte ranges. Eight MTRRs are used to characterize this address space.
        pub fn read() -> FixMemRangeReg {
            let r = Self::read_raw();
            let one = FixMemRange::new(0xF8000, 0xF8FFF, (r & 0xff).try_into().unwrap());
            let two = FixMemRange::new(
                0xF9000,
                0xF9FFF,
                ((r & (0xff << 8)) >> 8).try_into().unwrap(),
            );
            let three = FixMemRange::new(
                0xFA000,
                0xFAFFF,
                ((r & (0xff << 16)) >> 16).try_into().unwrap(),
            );
            let four = FixMemRange::new(
                0xFB000,
                0xFBFFF,
                ((r & (0xff << 24)) >> 24).try_into().unwrap(),
            );
            let five = FixMemRange::new(
                0xFC000,
                0xFCFFF,
                ((r & (0xff << 32)) >> 32).try_into().unwrap(),
            );
            let six = FixMemRange::new(
                0xFD000,
                0xFDFFF,
                ((r & (0xff << 40)) >> 40).try_into().unwrap(),
            );
            let seven = FixMemRange::new(
                0xFE000,
                0xFEFFF,
                ((r & (0xff << 48)) >> 48).try_into().unwrap(),
            );
            let eight = FixMemRange::new(
                0xFF000,
                0xFFFFF,
                ((r & (0xff << 56)) >> 56).try_into().unwrap(),
            );
            [one, two, three, four, five, six, seven, eight]
        }
    }
}
