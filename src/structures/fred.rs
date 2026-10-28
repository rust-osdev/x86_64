//! Types for Flexible Event and Return Delivery (FRED).

use core::fmt;

use bitflags::bitflags;

use crate::VirtAddr;
use crate::registers::rflags::RFlags;
use crate::structures::gdt::SegmentSelector;
use crate::structures::paging::Page;

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
    /// Creates a new FredStackLevelsValue. Each vector's two-bit field specifying
    /// the minimum stack level is 0.
    #[inline]
    pub const fn new() -> Self {
        Self { bits: 0 }
    }

    /// Constructs a new FredStackLevelsValue from an array of FredStackLevel.
    #[inline]
    pub const fn from_levels(levels: [FredStackLevel; 32]) -> Self {
        let mut config = Self::new();
        let mut vector = 0;

        while vector < 32 {
            config.set(vector as u8, levels[vector]);
            vector += 1;
        }

        config
    }

    /// Convert from underlying bit representation, preserving all bits,
    /// even those not corresponding to a defined field.
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

    /// Gets the stack level associated with the vector.
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

impl Default for FredStackLevelsValue {
    #[inline]
    fn default() -> Self {
        Self::new()
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
/// Bits `6..=8` denote how much the RSP decrements when event delivery uses the same stack.
/// Bits `9..=10` identify the stack level for maskable interrupts in ring 0.
/// Bits `12..=63` contain the linear address of a page containing event handlers.
#[derive(Clone, Copy)]
#[repr(C)]
pub struct FredConfigValue {
    bits: u64,
}

impl FredConfigValue {
    /// Constructs a FredConfigValue from an entry_page.
    /// Rest of the fields are set to zero.
    #[inline]
    pub const fn new(entry_page: Page) -> Self {
        Self {
            bits: entry_page.start_address().as_u64(),
        }
    }

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

    /// Gets the page of FRED event entrypoint.
    #[inline]
    pub const fn entry_page(&self) -> Page {
        let addr = VirtAddr::new(self.bits & !0xFFF);
        Page::containing_address(addr)
    }

    /// Sets the page of FRED event entrypoint.
    #[inline]
    pub const fn set_entry_page(&mut self, page: Page) {
        self.bits = (self.bits & 0xFFF) | page.start_address().as_u64();
    }

    /// Gets the linear address of the FRED user event entrypoint.
    pub const fn user_entry_addr(&self) -> VirtAddr {
        self.entry_page().start_address()
    }

    /// Gets the linear address of the FRED supervisor event entrypoint.
    pub const fn supervisor_entry_addr(&self) -> VirtAddr {
        let addr = self.entry_page().start_address().as_u64();
        VirtAddr::new(addr + 0x100u64)
    }
}

impl fmt::Debug for FredConfigValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FredConfigValue")
            .field("bits", &format_args!("{:#018x}", self.bits))
            .field("csl", &(self.csl()))
            .field("decrement_ssp", &(self.decrement_ssp()))
            .field("redzone", &(self.redzone()))
            .field(
                "ring0_interrupt_stack_level",
                &(self.ring0_interrupt_stack_level()),
            )
            .field("entry_page", &self.entry_page())
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
    pub stack_pointer: u64,
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
#[allow(unused_braces)]
const _: () = { assert!(size_of::<FredEventFrame>() == 64) };

impl FredEventFrame {
    /// Creates a new FRED event frame with the given values.
    #[inline]
    pub const fn new(
        instruction_pointer: VirtAddr,
        code_segment: SegmentSelector,
        cpu_flags: RFlags,
        stack_pointer: u64,
        stack_segment: SegmentSelector,
    ) -> Self {
        Self {
            error_code: FredEventErrorCode(0),
            _reserved1: [0; 6],
            instruction_pointer,
            code_segment,
            auxiliary: 0,
            _reserved2: [0; 4],
            cpu_flags,
            stack_pointer,
            stack_segment,
            return_flags: 0,
            event_information: 0,
            event_data: 0,
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

    /// Sets the current stack level of the event frame.
    #[inline]
    pub const fn set_csl(&mut self, csl: FredStackLevel) {
        self.auxiliary = (self.auxiliary & !0b11) | csl as u16;
    }

    /// Set to 1 if the event occurred in ring 0 and the indirect branch tracker
    /// was in the WAIT_FOR_ENDBRANCH state.
    #[inline]
    pub const fn ibt_in_wfe(&self) -> bool {
        self.auxiliary & (1 << 2) != 0
    }

    /// Sets whether the indirect branch tracker was in WAIT_FOR_ENDBRANCH state.
    #[inline]
    pub const fn set_ibt_in_wfe(&mut self, value: bool) {
        self.auxiliary = (self.auxiliary & !(1 << 2)) | ((value as u16) << 2);
    }

    /// Set to 1 for FRED event delivery of a hardware exception if interrupt blocking by STI
    /// was in effect at the time the exception occurred and is otherwise cleared to 0.
    #[inline]
    pub const fn sti_blocking(&self) -> bool {
        self.return_flags & (1 << 0) != 0
    }

    /// Sets whether the interrupt blocking by STI is in effect.
    #[inline]
    pub const fn set_sti_blocking(&mut self, value: bool) {
        self.return_flags = (self.return_flags & !(1 << 0)) | value as u16;
    }

    /// Set to 1 for FRED event delivery of SYSCALL, SYSENTER, or INT n
    /// (for any value of n) and is otherwise cleared to 0.
    #[inline]
    pub const fn from_syscall_sysenter_or_int(&self) -> bool {
        self.return_flags & (1 << 1) != 0
    }

    /// Sets whether the event delivery was by SYSCALL, SYSENTER, or INT n.
    #[inline]
    pub const fn set_from_syscall_sysenter_or_int(&mut self, value: bool) {
        self.return_flags = (self.return_flags & !(1 << 1)) | ((value as u16) << 1);
    }

    /// Set to 1 if the event being delivered is a non-maskable interrupt (NMI)
    /// and is otherwise cleared to 0.
    #[inline]
    pub const fn is_nmi(&self) -> bool {
        self.return_flags & (1 << 2) != 0
    }

    /// Sets whether the event is a non-maskable interrupt (NMI).
    #[inline]
    pub const fn set_is_nmi(&mut self, value: bool) {
        self.return_flags = (self.return_flags & !(1 << 2)) | ((value as u16) << 2);
    }

    /// Gets the event vector. The vector depends on the type of event:
    /// - For an external interrupt, the vector is provided by the local APIC.
    /// - For a non-maskable interrupt (NMI), the vector is 2.
    /// - For a hardware exception, the vector is determined by the exception.
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

    /// Sets the event vector of the event frame.
    #[inline]
    pub const fn set_event_vector(&mut self, vector: u8) {
        self.event_information = (self.event_information & !0xff) | vector as u32;
    }

    /// Gets the event type.
    #[inline]
    pub const fn event_type(&self) -> FredEventType {
        FredEventType(((self.event_information >> 16) & 0xf) as u8)
    }

    /// Sets the event type of the event frame.
    ///
    /// # Panics
    ///
    /// Panics if the event type does not fit in four bits.
    #[inline]
    pub const fn set_event_type(&mut self, event_type: FredEventType) {
        assert!(event_type.0 <= 0xf);
        self.event_information =
            (self.event_information & !(0xf << 16)) | ((event_type.0 as u32) << 16);
    }

    /// Whether the event was incident to enclave execution.
    #[inline]
    pub const fn enclave_event(&self) -> bool {
        self.event_information & (1 << 24) != 0
    }

    /// Sets whether the event is incident to enclave execution.
    #[inline]
    pub const fn set_enclave_event(&mut self, value: bool) {
        self.event_information = (self.event_information & !(1 << 24)) | ((value as u32) << 24);
    }

    /// Whether the previous context was in 64-bit mode.
    /// False indicates compatibility mode.
    #[inline]
    pub const fn was_64_bit_mode(&self) -> bool {
        self.event_information & (1 << 25) != 0
    }

    /// Sets whether the event is in 64-bit mode.
    #[inline]
    pub const fn set_was_64_bit_mode(&mut self, value: bool) {
        self.event_information = (self.event_information & !(1 << 25)) | ((value as u32) << 25);
    }

    /// Whether the exception occurred during FRED event delivery.
    /// Double faults are not reported as nested exceptions.
    #[inline]
    pub const fn nested_exception(&self) -> bool {
        self.event_information & (1 << 26) != 0
    }

    /// Sets whether the event is a nested exception.
    #[inline]
    pub const fn set_nested_exception(&mut self, value: bool) {
        self.event_information = (self.event_information & !(1 << 26)) | ((value as u32) << 26);
    }

    /// Gets the instruction length encoded in the event information.
    #[inline]
    pub const fn instruction_length(&self) -> u8 {
        ((self.event_information >> 28) & 0xF) as u8
    }

    /// Sets the instruction length encoded in the event.
    ///
    /// # Panics
    ///
    /// Panics if `length` is greater than 15.
    #[inline]
    pub const fn set_instruction_length(&mut self, length: u8) {
        assert!(length <= 0xf);
        self.event_information = (self.event_information & !(0xf << 28)) | ((length as u32) << 28);
    }
}

/// Contains FRED Event error code.
/// Defined only for certain exceptions; zero if none is defined.
#[derive(Clone, Copy, Debug)]
#[repr(transparent)]
pub struct FredEventErrorCode(pub u16);

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
    /// Privileged software exception, for example INT1.
    pub const PRIV_SOFTWARE_EXCEPTION: Self = Self(5);
    /// Software exception, for example INT3 or INTO.
    pub const SOFTWARE_EXCEPTION: Self = Self(6);
    /// Other kind of event, used for SYSCALL and SYSENTER.
    pub const OTHER_EVENT: Self = Self(7);
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
            .field("return_flags", &self.return_flags)
            .field("event_information", &self.event_information)
            .field("event_data", &self.event_data)
            .field("reserved", &self._reserved3)
            .finish()
    }
}
