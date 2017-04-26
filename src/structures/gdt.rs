//! Types for the Global Descriptor Table and segment selectors.

use core::fmt;
use core::marker::PhantomData;
use core::convert::{From,Into};
use core::intrinsics::transmute;
use PrivilegeLevel;
use bit_field::BitField;


/// Specifies which element to load into a segment from
/// descriptor tables (i.e., is a index to LDT or GDT table
/// with some additional flags).
///
/// See Intel 3a, Section 3.4.2 "Segment Selectors"
pub struct SegmentSelector(pub u16);

impl SegmentSelector {
    /// Creates a new SegmentSelector
    ///
    /// # Arguments
    ///  * `index`: index in GDT or LDT array.
    ///  * `rpl`: the requested privilege level
    pub const fn new(index: u16, rpl: PrivilegeLevel) -> SegmentSelector {
        SegmentSelector(index << 3 | (rpl as u16))
    }

    /// Returns the GDT index.
    pub fn index(&self) -> u16 {
        self.0 >> 3
    }

    /// Returns the requested privilege level.
    pub fn rpl(&self) -> PrivilegeLevel {
        PrivilegeLevel::from_uint(self.0.get_bits(0..2) as u8)
    }
}

/// A generic access byte trait.
pub trait GdtEntryAccess : Sized + Into<u8> + From<u8> {
    
    /// Simple constructor that creates an empty descriptor of the given type.
    fn base() -> Self;

    /// Returns the access byte with the dpl set.
    fn set_dpl(&self, dpl: PrivilegeLevel) -> Self {
        let t: *const u8 = unsafe { transmute(self) };
        Self::from(dpl.get_bits() << 5 & (0b10011111 & unsafe {*t}))
    }

    /// Returns the dpl of the access byte.
    fn get_dpl(self) -> PrivilegeLevel {
        PrivilegeLevel::from_uint((self.into() & 0b01100000) >> 5)
    }
}

bitflags! {
    pub flags GdtCodeEntryAccess: u8 {

        /// The accessed flag is set by the processor on first use.
        const ACCESSED = 1 << 0,

        /// Segment supports reads.
        const READ = 1 << 1,

        /// Segment is "conforming", meaning that the code can be run when RFLAGS indicates a lower
        /// privilege.
        const CONFORMING = 1 << 2,

        /// Must be set for code segments.  Setting to 0 makes this effectively a data segment.
        const _EXECUTABLE = 1 << 3,

        /// Must be set to 1 to be valid.
        const _NONSYSTEM = 1 << 4,


        /// Should be set if the segment is valid.
        const PRESENT = 1 << 7,

    }
}

impl From<u8> for GdtCodeEntryAccess {
    fn from(b: u8) -> Self {
        unsafe { transmute::<u8, Self>(b) }
    }
}

impl From<GdtCodeEntryAccess> for u8 {
    fn from(b: GdtCodeEntryAccess) -> Self {
        unsafe { transmute::<GdtCodeEntryAccess, Self>(b) }
    }
}

impl GdtEntryAccess for GdtCodeEntryAccess {
    fn base() -> Self {
        Self::_EXECUTABLE | Self::_NONSYSTEM | Self::PRESENT
    }
}

bitflags! {
    pub flags GdtDataEntryAccess: u8 {

        /// The accessed flag is set by the processor on first use.
        const ACCESSED = 1 << 0,

        /// Controls write access.
        const WRITE = 1 << 1,

        /// Limit grows down from base.
        const DIRECTION = 1 << 2,

        /// Must be set to 1 to be valid.
        const _NONSYSTEM = 1 << 4,


        /// Should be set if the segment is valid.
        const PRESENT = 1 << 7,

    }
}
impl From<u8> for GdtDataEntryAccess {
    fn from(b: u8) -> Self {
        unsafe { transmute::<u8, Self>(b) }
    }
}

impl From<GdtDataEntryAccess> for u8 {
    fn from(b: GdtDataEntryAccess) -> Self {
        unsafe { transmute::<GdtDataEntryAccess, Self>(b) }
    }
}


impl GdtEntryAccess for GdtDataEntryAccess {
    fn base() -> Self {
        Self::_NONSYSTEM | Self::PRESENT
    }
}

/// Various system gdt entries
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(u8)]
pub enum GdtSystemTypes {
    /// Long mode extended base address for previous entry.
    UpperBits =     0,
    LDT =           0b0010,
    TSS64 =         0b1001,
    TSS64Busy =     0b1011,
    CallGate64 =    0b1100,
    IntGate64 =     0b1110,
    TrapGate64 =    0b1111,

}

bitflags! {
    pub flags GdtSystemEntryAccess: u8 {

        /// Must be set to 0 to be valid for system entries.
        const _NONSYSTEM = 1 << 4,


        /// Should be set if the segment is valid.
        const PRESENT = 1 << 7,

    }
}


impl From<u8> for GdtSystemEntryAccess {
    fn from(b: u8) -> Self {
        unsafe { transmute::<u8, Self>(b) }
    }
}

impl From<GdtSystemEntryAccess> for u8 {
    fn from(b: GdtSystemEntryAccess) -> Self {
        unsafe { transmute::<GdtSystemEntryAccess, Self>(b) }
    }
}

impl GdtEntryAccess for GdtSystemEntryAccess {

    /// Type will default to UPPERBITS.  The type should always be set.
    fn base() -> Self {
        Self::PRESENT
    }
}

impl GdtSystemEntryAccess {

    /// Sets the type of the GDT entry to the specified type and returns the modified value.
    pub fn set_type(&self, segment_type: GdtSystemTypes) -> Self {
        let t: *const u8 = unsafe { transmute(self) };
        Self::from(segment_type as u8 & (0b11110000 & unsafe {*t}))
    }
}

/// Flag set for flag byte.
///
bitflags! {
    pub flags GdtFlags: u8 {
        const BASE = 0,

        /// This flag is available for user definition.
        const AVAILABLE = 1 << 4,


        /// Segment is a 64-bit code segment.
        const LONG_MODE = 1 << 5,

        /// If the code or data in the segment is 16 or 32 bit.
        const DB = 1 << 6,

        /// The granularity bit.
        const GRANULARITY = 1 << 7,
    }
}

/// A Global Descriptor Table entry.
///
#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
pub struct GdtEntry<F, A: GdtEntryAccess> {
    limit: u16,
    base0: u16,
    base1: u8,
    access: A,
    limit_flags: GdtFlags,
    base2: u8,
    phantom: PhantomData<F>,
}


impl<F, A: GdtEntryAccess> GdtEntry<F, A> {

    /// Creates an empty GdtEntry
    pub fn missing() -> Self {
        GdtEntry {
            limit: 0,
            base0: 0,
            base1: 0,
            access: A::base(),
            limit_flags: GdtFlags::BASE,
            base2: 0,
            phantom: PhantomData
        }
    }

    /// Sets the base address for the segment.  Only sets the low 32 bits.  For system segments
    /// it will be necessary to set the high bits in the following 8-byte field.
    pub fn set_base(&mut self, base_addr: u32) {
        self.base0 = (base_addr & 0xffff) as u16;
        self.base1 = (base_addr & 0xff0000 >> 16) as u8;
        self.base2 = (base_addr & 0xff000000 >> 24) as u8;
    }

}

impl fmt::Debug for SegmentSelector {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut s = f.debug_struct("SegmentSelector");
        s.field("index", &self.index());
        s.field("rpl", &self.rpl());
        s.finish()
    }
}
