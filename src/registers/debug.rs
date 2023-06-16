//! Functions to read and write debug registers.

#[cfg(feature = "instructions")]
use core::arch::asm;
use core::ops::Range;

use bit_field::BitField;
use bitflags::bitflags;

/// Debug Address Register
///
/// Holds the address of a hardware breakpoint.
pub trait DebugAddressRegister {
    /// The corresponding [`DebugAddressRegisterNumber`].
    const NUM: DebugAddressRegisterNumber;

    /// Reads the current breakpoint address.
    #[cfg(feature = "instructions")]
    fn read() -> u64;

    /// Writes the provided breakpoint address.
    #[cfg(feature = "instructions")]
    fn write(addr: u64);
}

macro_rules! debug_address_register {
    ($Dr:ident, $name:literal) => {
        /// Debug Address Register
        ///
        /// Holds the address of a hardware breakpoint.
        #[derive(Debug)]
        pub struct $Dr;

        impl DebugAddressRegister for $Dr {
            const NUM: DebugAddressRegisterNumber = DebugAddressRegisterNumber::$Dr;

            #[cfg(feature = "instructions")]
            #[inline]
            fn read() -> u64 {
                let addr;
                unsafe {
                    asm!(concat!("mov {}, ", $name), out(reg) addr, options(nomem, nostack, preserves_flags));
                }
                addr
            }

            #[cfg(feature = "instructions")]
            #[inline]
            fn write(addr: u64) {
                unsafe {
                    asm!(concat!("mov ", $name, ", {}"), in(reg) addr, options(nomem, nostack, preserves_flags));
                }
            }
        }
    };
}

debug_address_register!(Dr0, "dr0");
debug_address_register!(Dr1, "dr1");
debug_address_register!(Dr2, "dr2");
debug_address_register!(Dr3, "dr3");

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
/// A valid debug address register number.
///
/// Must be between 0 and 3 (inclusive).
pub enum DebugAddressRegisterNumber {
    /// The debug address register number of [`Dr0`] (0).
    Dr0,

    /// The debug address register number of [`Dr1`] (1).
    Dr1,

    /// The debug address register number of [`Dr2`] (2).
    Dr2,

    /// The debug address register number of [`Dr3`] (3).
    Dr3,
}

impl DebugAddressRegisterNumber {
    /// Creates a debug address register number if it is valid.
    pub const fn new(n: u8) -> Option<Self> {
        match n {
            0 => Some(Self::Dr0),
            1 => Some(Self::Dr1),
            2 => Some(Self::Dr2),
            3 => Some(Self::Dr3),
            _ => None,
        }
    }

    /// Returns the number as a primitive type.
    pub const fn get(self) -> u8 {
        match self {
            Self::Dr0 => 0,
            Self::Dr1 => 1,
            Self::Dr2 => 2,
            Self::Dr3 => 3,
        }
    }
}

/// Debug Status Register (DR6).
///
/// Reports debug conditions from the last debug exception.
#[derive(Debug)]
pub struct Dr6;

bitflags! {
    /// Debug condition flags of the [`Dr6`] register.
    #[repr(transparent)]
    #[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Clone, Copy)]
    pub struct Dr6Flags: u64 {
        /// Breakpoint condition 0 was detected.
        const TRAP0 = 1;

        /// Breakpoint condition 1 was detected.
        const TRAP1 = 1 << 1;

        /// Breakpoint condition 2 was detected.
        const TRAP2 = 1 << 2;

        /// Breakpoint condition 3 was detected.
        const TRAP3 = 1 << 3;

        /// Breakpoint condition was detected.
        const TRAP = Self::TRAP0.bits() | Self::TRAP1.bits() | Self::TRAP2.bits() | Self::TRAP3.bits();

        /// Next instruction accesses one of the debug registers.
        ///
        /// Enabled via [`Dr7Flags::GENERAL_DETECT_ENABLE`].
        const ACCESS_DETECTED = 1 << 13;

        /// CPU is in single-step execution mode.
        ///
        /// Enabled via [`RFlags::TRAP_FLAG`].
        const STEP = 1 << 14;

        /// Task switch.
        ///
        /// Enabled via the debug trap flag in the TSS of the target task.
        const SWITCH = 1 << 15;

        /// When *clear*, indicates a debug or breakpoint exception inside an RTM region.
        ///
        /// Enabled via [`Dr7Flags::RESTRICTED_TRANSACTIONAL_MEMORY`] and the
        /// RTM flag in the `IA32_DEBUGCTL` [`Msr`].
        const RTM = 1 << 16;
    }
}

impl Dr6Flags {
    /// Returns the trap flag of the provided debug address register.
    pub fn trap(n: DebugAddressRegisterNumber) -> Self {
        match n {
            DebugAddressRegisterNumber::Dr0 => Self::TRAP0,
            DebugAddressRegisterNumber::Dr1 => Self::TRAP1,
            DebugAddressRegisterNumber::Dr2 => Self::TRAP2,
            DebugAddressRegisterNumber::Dr3 => Self::TRAP3,
        }
    }

    #[deprecated = "use the safe `from_bits_retain` method instead"]
    /// Convert from underlying bit representation, preserving all bits (even those not corresponding to a defined flag).
    pub const unsafe fn from_bits_unchecked(bits: u64) -> Self {
        Self::from_bits_retain(bits)
    }
}

bitflags! {
    /// Debug control flags of the [`Dr7`] register.
    #[repr(transparent)]
    #[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Clone, Copy)]
    pub struct Dr7Flags: u64 {
        /// Breakpoint 0 is enabled for the current task.
        const LOCAL_BREAKPOINT_0_ENABLE = 1;

        /// Breakpoint 1 is enabled for the current task.
        const LOCAL_BREAKPOINT_1_ENABLE = 1 << 2;

        /// Breakpoint 2 is enabled for the current task.
        const LOCAL_BREAKPOINT_2_ENABLE = 1 << 4;

        /// Breakpoint 3 is enabled for the current task.
        const LOCAL_BREAKPOINT_3_ENABLE = 1 << 6;

        /// Breakpoint 0 is enabled for all tasks.
        const GLOBAL_BREAKPOINT_0_ENABLE = 1 << 1;

        /// Breakpoint 1 is enabled for all tasks.
        const GLOBAL_BREAKPOINT_1_ENABLE = 1 << 3;

        /// Breakpoint 2 is enabled for all tasks.
        const GLOBAL_BREAKPOINT_2_ENABLE = 1 << 5;

        /// Breakpoint 3 is enabled for all tasks.
        const GLOBAL_BREAKPOINT_3_ENABLE = 1 << 7;

        /// Enable detection of exact instruction causing a data breakpoint condition for the current task.
        ///
        /// This is not supported by `x86_64` processors, but is recommended to be enabled for backward and forward compatibility.
        const LOCAL_EXACT_BREAKPOINT_ENABLE = 1 << 8;

        /// Enable detection of exact instruction causing a data breakpoint condition for all tasks.
        ///
        /// This is not supported by `x86_64` processors, but is recommended to be enabled for backward and forward compatibility.
        const GLOBAL_EXACT_BREAKPOINT_ENABLE = 1 << 9;

        /// Enables advanced debugging of RTM transactional regions.
        ///
        /// The RTM flag in the `IA32_DEBUGCTL` [`Msr`] must also be set.
        const RESTRICTED_TRANSACTIONAL_MEMORY = 1 << 11;

        /// Enables debug register protection.
        ///
        /// This will cause a debug exception before any access to a debug register.
        const GENERAL_DETECT_ENABLE = 1 << 13;
    }
}

impl Dr7Flags {
    /// Returns the local breakpoint enable flag of the provided debug address register.
    pub fn local_breakpoint_enable(n: DebugAddressRegisterNumber) -> Self {
        match n {
            DebugAddressRegisterNumber::Dr0 => Self::LOCAL_BREAKPOINT_0_ENABLE,
            DebugAddressRegisterNumber::Dr1 => Self::LOCAL_BREAKPOINT_1_ENABLE,
            DebugAddressRegisterNumber::Dr2 => Self::LOCAL_BREAKPOINT_2_ENABLE,
            DebugAddressRegisterNumber::Dr3 => Self::LOCAL_BREAKPOINT_3_ENABLE,
        }
    }

    /// Returns the global breakpoint enable flag of the provided debug address register.
    pub fn global_breakpoint_enable(n: DebugAddressRegisterNumber) -> Self {
        match n {
            DebugAddressRegisterNumber::Dr0 => Self::GLOBAL_BREAKPOINT_0_ENABLE,
            DebugAddressRegisterNumber::Dr1 => Self::GLOBAL_BREAKPOINT_1_ENABLE,
            DebugAddressRegisterNumber::Dr2 => Self::GLOBAL_BREAKPOINT_2_ENABLE,
            DebugAddressRegisterNumber::Dr3 => Self::GLOBAL_BREAKPOINT_3_ENABLE,
        }
    }

    #[deprecated = "use the safe `from_bits_retain` method instead"]
    /// Convert from underlying bit representation, preserving all bits (even those not corresponding to a defined flag).
    pub const unsafe fn from_bits_unchecked(bits: u64) -> Self {
        Self::from_bits_retain(bits)
    }
}

/// The condition for a hardware breakpoint.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum BreakpointCondition {
    /// Instruction execution
    InstructionExecution = 0b00,

    /// Data writes
    DataWrites = 0b01,

    /// I/O reads or writes
    IoReadsWrites = 0b10,

    /// Data reads or writes but not instruction fetches
    DataReadsWrites = 0b11,
}

impl BreakpointCondition {
    /// Creates a new hardware breakpoint condition if `bits` is valid.
    pub const fn from_bits(bits: u64) -> Option<Self> {
        match bits {
            0b00 => Some(Self::InstructionExecution),
            0b01 => Some(Self::DataWrites),
            0b10 => Some(Self::IoReadsWrites),
            0b11 => Some(Self::DataReadsWrites),
            _ => None,
        }
    }

    const fn bit_range(n: DebugAddressRegisterNumber) -> Range<usize> {
        let lsb = (16 + 4 * n.get()) as usize;
        lsb..lsb + 2
    }
}

/// The size of a hardware breakpoint.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum BreakpointSize {
    /// 1 byte length
    Length1B = 0b00,

    /// 2 byte length
    Length2B = 0b01,

    /// 8 byte length
    Length8B = 0b10,

    /// 4 byte length
    Length4B = 0b11,
}

impl BreakpointSize {
    /// Creates a new hardware breakpoint size if `size` is valid.
    pub const fn new(size: usize) -> Option<Self> {
        match size {
            1 => Some(Self::Length1B),
            2 => Some(Self::Length2B),
            8 => Some(Self::Length8B),
            4 => Some(Self::Length4B),
            _ => None,
        }
    }

    /// Creates a new hardware breakpoint size if `bits` is valid.
    pub const fn from_bits(bits: u64) -> Option<Self> {
        match bits {
            0b00 => Some(Self::Length1B),
            0b01 => Some(Self::Length2B),
            0b10 => Some(Self::Length8B),
            0b11 => Some(Self::Length4B),
            _ => None,
        }
    }

    const fn bit_range(n: DebugAddressRegisterNumber) -> Range<usize> {
        let lsb = (18 + 4 * n.get()) as usize;
        lsb..lsb + 2
    }
}

/// A valid value of the [`Dr7`] debug register.
///
/// In addition to the [`Dr7Flags`] this value has a condition field and a size field for each debug address register.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(transparent)]
pub struct Dr7Value {
    bits: u64,
}

impl From<Dr7Flags> for Dr7Value {
    fn from(dr7_flags: Dr7Flags) -> Self {
        Self::from_bits_truncate(dr7_flags.bits())
    }
}

impl Dr7Value {
    const fn valid_bits() -> u64 {
        let field_valid_bits = (1 << 32) - (1 << 16);
        let flag_valid_bits = Dr7Flags::all().bits();
        field_valid_bits | flag_valid_bits
    }

    /// Convert from underlying bit representation, unless that representation contains bits that do not correspond to a field.
    #[inline]
    pub const fn from_bits(bits: u64) -> Option<Self> {
        if (bits & !Self::valid_bits()) == 0 {
            Some(Self { bits })
        } else {
            None
        }
    }

    /// Convert from underlying bit representation, dropping any bits that do not correspond to fields.
    #[inline]
    pub const fn from_bits_truncate(bits: u64) -> Self {
        Self {
            bits: bits & Self::valid_bits(),
        }
    }

    /// Convert from underlying bit representation, preserving all bits (even those not corresponding to a defined field).
    ///
    /// # Safety
    ///
    /// The bit representation must be a valid [`Dr7Value`].
    #[inline]
    pub const unsafe fn from_bits_unchecked(bits: u64) -> Self {
        Self { bits }
    }

    /// Returns the raw value of the fields currently stored.
    #[inline]
    pub const fn bits(&self) -> u64 {
        self.bits
    }

    /// Returns the [`Dr7Flags`] in this value.
    #[inline]
    pub const fn flags(self) -> Dr7Flags {
        Dr7Flags::from_bits_truncate(self.bits)
    }

    /// Inserts the specified [`Dr7Flags`] in-place.
    #[inline]
    pub fn insert_flags(&mut self, flags: Dr7Flags) {
        self.bits |= flags.bits();
    }

    /// Removes the specified [`Dr7Flags`] in-place.
    #[inline]
    pub fn remove_flags(&mut self, flags: Dr7Flags) {
        self.bits &= !flags.bits();
    }

    /// Toggles the specified [`Dr7Flags`] in-place.
    #[inline]
    pub fn toggle_flags(&mut self, flags: Dr7Flags) {
        self.bits ^= flags.bits();
    }

    /// Inserts or removes the specified [`Dr7Flags`] depending on the passed value.
    #[inline]
    pub fn set_flags(&mut self, flags: Dr7Flags, value: bool) {
        if value {
            self.insert_flags(flags);
        } else {
            self.remove_flags(flags);
        }
    }

    /// Returns the condition field of a debug address register.
    pub fn condition(&self, n: DebugAddressRegisterNumber) -> BreakpointCondition {
        let condition = self.bits.get_bits(BreakpointCondition::bit_range(n));
        BreakpointCondition::from_bits(condition).expect("condition should be always valid")
    }

    /// Sets the condition field of a debug address register.
    pub fn set_condition(&mut self, n: DebugAddressRegisterNumber, condition: BreakpointCondition) {
        self.bits
            .set_bits(BreakpointCondition::bit_range(n), condition as u64);
    }

    /// Returns the size field of a debug address register.
    pub fn size(&self, n: DebugAddressRegisterNumber) -> BreakpointSize {
        let size = self.bits.get_bits(BreakpointSize::bit_range(n));
        BreakpointSize::from_bits(size).expect("condition should be always valid")
    }

    /// Sets the size field of a debug address register.
    pub fn set_size(&mut self, n: DebugAddressRegisterNumber, size: BreakpointSize) {
        self.bits
            .set_bits(BreakpointSize::bit_range(n), size as u64);
    }
}

/// Debug Control Register (DR7).
///
/// Configures debug conditions for debug exceptions.
#[derive(Debug)]
pub struct Dr7;

#[cfg(feature = "instructions")]
mod x86_64 {
    use super::*;

    impl Dr6 {
        /// Read the current set of DR6 flags.
        #[inline]
        pub fn read() -> Dr6Flags {
            Dr6Flags::from_bits_truncate(Self::read_raw())
        }

        /// Read the current raw DR6 value.
        #[inline]
        pub fn read_raw() -> u64 {
            let value;

            unsafe {
                asm!("mov {}, dr6", out(reg) value, options(nomem, nostack, preserves_flags));
            }

            value
        }
    }

    impl Dr7 {
        /// Read the current set of DR7 flags.
        #[inline]
        pub fn read() -> Dr7Value {
            Dr7Value::from_bits_truncate(Self::read_raw())
        }

        /// Read the current raw DR7 value.
        #[inline]
        pub fn read_raw() -> u64 {
            let value;

            unsafe {
                asm!("mov {}, dr7", out(reg) value, options(nomem, nostack, preserves_flags));
            }

            value
        }

        /// Write DR7 value.
        ///
        /// Preserves the value of reserved fields.
        #[inline]
        pub fn write(value: Dr7Value) {
            let old_value = Self::read_raw();
            let reserved = old_value & !Dr7Value::valid_bits();
            let new_value = reserved | value.bits();

            Self::write_raw(new_value)
        }

        /// Write raw DR7 value.
        #[inline]
        pub fn write_raw(value: u64) {
            unsafe {
                asm!("mov dr7, {}", in(reg) value, options(nomem, nostack, preserves_flags));
            }
        }
    }
}
