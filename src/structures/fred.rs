//! Types for Flexible Event and Return Delivery (FRED).

use core::fmt;

use bitflags::bitflags;

use crate::VirtAddr;
use crate::registers::rflags::RFlags;
use crate::structures::gdt::SegmentSelector;

/// Represents a stack level under FRED.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum FredStackLevel {
    /// Represents FRED stack level 0.
    Level0 = 0,
    /// Represents FRED stack level 1.
    Level1 = 1,
    /// Represents FRED stack level 2.
    Level2 = 2,
    /// Represents FRED stack level 3.
    Level3 = 3,
}

impl FredStackLevel {
    /// Creates a `FredStackLevel` from a numeric value. The value must be in the range 0..4.
    ///
    /// This function panics if the passed value is >3.
    #[inline]
    pub const fn from_u16(value: u16) -> Self {
        match value {
            0 => Self::Level0,
            1 => Self::Level1,
            2 => Self::Level2,
            3 => Self::Level3,
            _ => panic!("invalid stack level"),
        }
    }
}

/// Stack-level assignments for exception vectors 0 through 31 and the NMI vector.
///
/// This is the value of `IA32_FRED_STKLVLS`. Each vector has a two-bit field
/// specifying the minimum stack level used when the event occurs in ring 0,
/// is a nested exception, or is a double fault.
#[derive(Clone, Copy)]
#[repr(C)]
pub struct FredStackLevelsValue {
    bits: u64,
}

impl FredStackLevelsValue {
    /// Convert from underlying bit representation, preserving all bits,
    /// Even those not corresponding to a defined field.
    ///
    /// # Safety
    ///
    /// The bit representation must be a valid [`FredStackLevelsValue`].
    #[inline]
    pub const unsafe fn from_bits_unchecked(bits: u64) -> Self {
        Self { bits }
    }

    /// Returns the raw value of the fields currently stored.
    #[inline]
    pub const fn bits(&self) -> u64 {
        self.bits
    }

    /// Gets the stack level associated to the vector.
    #[inline]
    pub const fn get(&self, vector: u8) -> FredStackLevel {
        assert!(vector < 32);
        FredStackLevel::from_u16(((self.bits >> (vector * 2)) & 0b11) as u16)
    }

    /// Sets the stack level associated to the vector.
    #[inline]
    pub const fn set(&mut self, vector: u8, level: FredStackLevel) {
        assert!(vector < 32);
        let shift = vector * 2;
        let mask = 0b11u64 << shift;
        self.bits = (self.bits & !mask) | ((level as u64) << shift);
    }
}

impl fmt::Debug for FredStackLevelsValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let levels: [FredStackLevel; 32] = core::array::from_fn(|vector| self.get(vector as u8));

        f.debug_struct("FredStackLevelsValue")
            .field("bits", &format_args!("{:#018x}", self.bits))
            .field("levels", &levels)
            .finish()
    }
}

/// Wrapper for IA32_FRED_CONFIG.
/// Bits `0..=1` contains current stack level (CSL).
/// Bit `3` if set, if event delivery doesn't change stacks, decrement SSP by 8.
/// Bits `6..=8` denotes how much the RSP decrements when event delivery uses the same stack.
/// Bits `9..=10` identifies the stack level for maskable interrupts in ring 0.
/// Bits `12..=63` contain the linear address of a page containing event handlers.
#[derive(Clone, Copy)]
#[repr(C)]
pub struct FredConfigValue {
    bits: u64,
}

impl FredConfigValue {
    /// Convert from underlying bit representation, preserving all bits,
    /// Even those not corresponding to a defined field.
    ///
    /// # Safety
    ///
    /// The bit representation must be a valid [`FredConfigValue`].
    #[inline]
    pub const unsafe fn from_bits_unchecked(bits: u64) -> Self {
        Self { bits }
    }

    /// Returns the raw value of the fields currently stored.
    #[inline]
    pub const fn bits(&self) -> u64 {
        self.bits
    }

    /// Gets the current stack level.
    #[inline]
    pub const fn csl(&self) -> FredStackLevel {
        let csl = self.bits & 0b11;
        FredStackLevel::from_u16(csl as u16)
    }

    /// Sets the current stack level.
    #[inline]
    pub const fn set_csl(&mut self, csl: FredStackLevel) {
        self.bits = (self.bits & !0b11) | csl as u64
    }

    /// Whether FRED decrements SSP by 8 bytes when the stack does not change.
    #[inline]
    pub const fn decrement_ssp(&self) -> bool {
        self.bits & (1 << 3) != 0
    }

    /// Configures the 8-byte SSP decrement for events that stay on the same stack.
    #[inline]
    pub const fn set_decrement_ssp(&mut self, value: bool) {
        self.bits = (self.bits & !(1 << 3)) | ((value as u64) << 3);
    }

    /// Gets the RSP decrement in bytes when the stack does not change.
    #[inline]
    pub const fn redzone(&self) -> u16 {
        (self.bits & 0x1C0) as u16
    }

    /// Sets the RSP decrement in bytes when the stack does not change.
    ///
    /// Panics unless `value` is a multiple of 64 in the range 0..=448.
    #[inline]
    pub const fn set_redzone(&mut self, value: u16) {
        assert!(value <= 448 && value.is_multiple_of(64));
        self.bits = (self.bits & !0x1C0) | value as u64;
    }

    /// Gets the stack level for maskable interrupts delivered in ring 0.
    #[inline]
    pub const fn ring0_interrupt_stack_level(&self) -> FredStackLevel {
        let ring0_isl = (self.bits >> 9) & 0b11;
        FredStackLevel::from_u16(ring0_isl as u16)
    }

    /// Sets the stack level for maskable interrupts delivered in ring 0.
    #[inline]
    pub const fn set_ring0_interrupt_stack_level(&mut self, sl: FredStackLevel) {
        self.bits = (self.bits & !(0b11 << 9)) | ((sl as u64) << 9);
    }

    /// Gets the linear address of FRED event entrypoint.
    #[inline]
    pub const fn entry_addr(&self) -> VirtAddr {
        let addr = self.bits & !0xFFF;
        VirtAddr::new(addr)
    }

    /// Sets the linear address of FRED event entrypoint.
    #[inline]
    pub const fn set_entry_addr(&mut self, addr: VirtAddr) {
        assert!(addr.is_aligned_u64(4096));
        self.bits = (self.bits & 0xFFF) | (addr.as_u64() & !0xFFF)
    }
}

impl fmt::Debug for FredConfigValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FredConfigValue")
            .field("bits", &format_args!("{:#018x}", self.bits))
            .field("current_stack_level", &(self.csl()))
            .field("shadow_stack_decrement", &(self.decrement_ssp()))
            .field("stack_decrement_bytes", &(self.redzone()))
            .field(
                "interrupt_stack_level",
                &(self.ring0_interrupt_stack_level()),
            )
            .field("entry_base", &self.entry_addr())
            .finish()
    }
}

// The FRED event frame pushed is as follows:
//	  +--------+-----------------+
//	  | Bytes  | Usage           |
//	  +--------+-----------------+
//	  | 63:56  | Reserved        |
//	  | 55:48  | Event Data      |
//	  | 47:44  | Event Info      |
//	  | 43:42  | Return flags    |
//	  | 41:40  | SS              |
//	  | 39:32  | RSP             |
//	  | 31:24  | RFLAGS          |
//	  | 23:20  | Reserved        |
//	  | 19:18  | Aux Info        |
//	  | 17:16  | CS              |
//	  |  15:8  | RIP             |
//	  |   7:2  | Reserved        |
//	  |   1:0  | Error Code      |
//	  +--------+-----------------+
//
/// Represents the interrupt stack frame pushed by the CPU on interrupt or exception entry.
#[derive(Clone, Copy)]
#[repr(C)]
pub struct FredEventFrame {
    /// Defined only for certain exceptions; zero if none is defined.
    pub error_code: FredEventErrorCode,
    _reserved1: [u8; 6],
    /// Instruction pointer of the previous context.
    pub instruction_pointer: VirtAddr,
    /// The code segment.
    pub code_segment: SegmentSelector,
    /// Bits `0..=1` contains the current stack level (CSL).
    /// Bit 2 is 1 if event occurred in Ring 0 and IBT tracker enabled.
    auxiliary: u16,
    _reserved2: [u8; 4],
    /// The flags register.
    pub cpu_flags: RFlags,
    /// The stack pointer.
    pub stack_pointer: VirtAddr,
    /// The stack segment descriptor.
    pub stack_segment: SegmentSelector,
    /// Bit 0 is 1 for event delivery of a hardware exception if STI was enabled.
    /// Bit 1 is 1 for event delivery of SYSCALL, SYSENTER, or INT n.
    /// Bit 2 is 1 for event delivery for NMIs.
    return_flags: u16,
    /// Contains event information.
    /// This provides software with additional information about the event.
    event_information: u32,
    /// Contains data related to the event.
    pub event_data: u64,
    _reserved3: [u8; 8],
}
const _: () = assert!(size_of::<FredEventFrame>() == 64);

impl FredEventFrame {
    /// Creates a new FRED event frame with the given values.
    #[allow(clippy::too_many_arguments)]
    #[inline]
    pub const fn new(
        error_code: FredEventErrorCode,
        instruction_pointer: VirtAddr,
        code_segment: SegmentSelector,
        auxiliary: u16,
        cpu_flags: RFlags,
        stack_pointer: VirtAddr,
        stack_segment: SegmentSelector,
        return_flags: u16,
        event_information: u32,
        event_data: u64,
    ) -> Self {
        Self {
            error_code,
            _reserved1: [0; 6],
            instruction_pointer,
            code_segment,
            auxiliary,
            _reserved2: [0; 4],
            cpu_flags,
            stack_pointer,
            stack_segment,
            return_flags,
            event_information,
            event_data,
            _reserved3: [0; 8],
        }
    }

    /// For delivery of events that occur in ring 0, these bits report the current
    /// stack level (CSL) at the time the event occurred.
    ///
    /// For delivery of events that occur in ring 3, these bits are cleared to 0.
    #[inline]
    pub const fn csl(&self) -> FredStackLevel {
        let bits = self.auxiliary & 0b11;
        FredStackLevel::from_u16(bits)
    }

    /// Set to 1 if the event occurred in ring 0 and the indirect branch tracker
    /// was in the WAIT_FOR_ENDBRANCH state.
    #[inline]
    pub const fn ibt_in_wfe(&self) -> bool {
        self.auxiliary & (1 << 2) != 0
    }

    /// Set to 1 for FRED event delivery of a hardware exception if interrupt blocking by STI
    /// was in effect at the time the exception occurred and is otherwise cleared to 0.
    #[inline]
    pub const fn sti_blocking(&self) -> bool {
        self.return_flags & (1 << 0) != 0
    }

    /// Set to 1 for FRED event delivery of SYSCALL, SYSENTER, or INT n
    /// (for any value of n) and is otherwise cleared to 0.
    #[inline]
    pub const fn from_syscall_sysenter_or_int(&self) -> bool {
        self.return_flags & (1 << 1) != 0
    }

    /// Set to 1 if the event being delivered is a non-maskable interrupt (NMI)
    /// and is otherwise cleared to 0.
    #[inline]
    pub const fn is_nmi(&self) -> bool {
        self.return_flags & (1 << 2) != 0
    }

    /// Gets the event vector. The vector depends on the type of event:
    /// - For an external interrupt, the vector is provided by the local APIC.
    /// - or a non-maskable interrupt (NMI), the vector is 2.
    /// - or a hardware exception, the vector is determined by the exception.
    /// - For a software interrupt (INT n), the vector comes from the instruction’s opcode (value n)
    /// - For a software exception, the vector is determined by the instruction opcode:
    ///     - INT1 has vector 1;
    ///     - INT3 has vector 3;
    ///     - INTO has vector 4.
    /// - For SYSCALL and SYSENTER, vectors 1 and 2 are used, respectively.
    #[inline]
    pub const fn event_vector(&self) -> u8 {
        (self.event_information & 0xff) as u8
    }

    /// Gets the event type.
    #[inline]
    pub const fn event_type(&self) -> FredEventType {
        FredEventType(((self.event_information >> 16) & 0xf) as u8)
    }

    /// Interprets the event data using the event type and vector.
    ///
    /// Panics if a page-fault address is not representable by [`VirtAddr`].
    #[inline]
    pub const fn event_data(&self) -> FredEventData {
        let vector = self.event_vector();
        let event_type = self.event_type();

        match (event_type, vector) {
            (FredEventType::HARDWARE_EXCEPTION, 14) => {
                FredEventData::PageFault(VirtAddr::new(self.event_data))
            }
            (FredEventType::HARDWARE_EXCEPTION | FredEventType::PRIV_SOFTWARE_EXCEPTION, 1) => {
                FredEventData::DebugException(FredEventDataDebug::from_bits_retain(self.event_data))
            }
            (FredEventType::HARDWARE_EXCEPTION, 7) => {
                FredEventData::DeviceNotAvailable(self.event_data)
            }
            (FredEventType::NON_MASKABLE_INTERRUPT, 2) => {
                FredEventData::NonMaskableInterrupt(self.event_data as u16)
            }
            _ => FredEventData::Other(self.event_data),
        }
    }
}

/// Contains FRED Event error code.
/// Defined only for certain exceptions; zero if none is defined.
#[derive(Clone, Copy, Debug)]
#[repr(transparent)]
pub struct FredEventErrorCode(pub u16);

bitflags! {
    /// Return-state flags saved in bits 18:16 of the augmented SS field.
    #[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Clone, Copy)]
    #[repr(transparent)]
    pub struct FredEventReturnFlags: u16 {
        /// Interrupt blocking by STI was in effect when a hardware exception occurred.
        const STI_BLOCKED = 1 << 0;
        /// The event was delivered for SYSCALL, SYSENTER, or INT n.
        const SOFTWARE_EVENT = 1 << 1;
        /// The event being delivered is a non-maskable interrupt.
        const NMI = 1 << 2;
    }
}

/// Stores the type of the FRED event.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FredEventType(pub u8);

impl FredEventType {
    /// External interrupt.
    pub const EXTERNAL_INTERRUPT: Self = Self(0);
    /// Non Maskable Interrupt (NMI).
    pub const NON_MASKABLE_INTERRUPT: Self = Self(2);
    /// Hardware exception, for example a page fault.
    pub const HARDWARE_EXCEPTION: Self = Self(3);
    /// Software interrupt, for example INT n.
    pub const SOFTWARE_INTERRUPT: Self = Self(4);
    /// Privileged software exception, for example INT 1.
    pub const PRIV_SOFTWARE_EXCEPTION: Self = Self(5);
    /// Software exception, for example INT 3 or INTO.
    pub const SOFTWARE_EXCEPTION: Self = Self(6);
    /// Other kind of event, used for SYSCALL and SYSENTER.
    pub const OTHER_EVENT: Self = Self(7);
}

/// Possible FRED Event Data types.
#[derive(Clone, Copy, Debug)]
pub enum FredEventData {
    /// If the event is a page fault, event data is the faulting linear address.
    PageFault(VirtAddr),
    /// If the event is a debug exception, event data identifies it.
    DebugException(FredEventDataDebug),
    /// If the event is a #NM, event data identifies it.
    /// The event data identical to what is loaded in the the IA32_XFD_ERR.
    /// If the #NM was not caused by extended feature disable, the event data is zero.
    DeviceNotAvailable(u64),
    /// If the event is an NMI, event data contains the NMI-source bitmap.
    NonMaskableInterrupt(u16),
    /// For any other event, event data is zero.
    /// The event data is determined differently when FRED event delivery is
    /// used for an event injected by VM entry.
    Other(u64),
}

bitflags! {
    /// Encodes FRED event data for an event caused by an exception.
    #[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Clone, Copy)]
    #[repr(transparent)]
    pub struct FredEventDataDebug: u64 {
        /// This bit indicates that the breakpoint condition 0 was met.
        const B0 = 1 << 0;
        /// This bit indicates that the breakpoint condition 1 was met.
        const B1 = 1 << 1;
        /// This bit indicates that the breakpoint condition 2 was met.
        const B2 = 1 << 2;
        /// This bit indicates that the breakpoint condition 3 was met.
        const B3 = 1 << 3;
        /// This bit indicates that the cause of the exception was a bus lock acquisition.
        const BLD = 1 << 11;
        /// This bit indicates that the cause of the exception was debug register access.
        const BD = 1 << 13;
        /// This bit indicates that the cause of the exception was a single instruction.
        const BS = 1 << 14;
        /// This bit indicates that a debug exception or a breakpoint exception occurred
        /// inside of an RTM region while advanced debugging of RTM transactional regions was enabled.
        const RTM = 1 << 16;
    }
}

impl fmt::Debug for FredEventFrame {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("FredEventFrame")
            .field("error_code", &self.error_code)
            .field("reserved", &self._reserved1)
            .field("instruction_pointer", &self.instruction_pointer)
            .field("code_segment", &self.code_segment)
            .field("auxiliary", &self.auxiliary)
            .field("reserved", &self._reserved2)
            .field("cpu_flags", &self.cpu_flags)
            .field("stack_pointer", &self.stack_pointer)
            .field("stack_segment", &self.stack_segment)
            .field("event_flags", &self.return_flags)
            .field("event_information", &self.event_information)
            .field("event_data", &self.event_data)
            .field("reserved", &self._reserved3)
            .finish()
    }
}
