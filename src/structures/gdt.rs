//! Types for the Global Descriptor Table and segment selectors.

use core::fmt;
use core::marker::PhantomData;
use core::mem;
use core::ops::{Index, IndexMut};
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
        const _REQUIRED = 1 << 4,


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
        Self::_EXECUTABLE | Self::_REQUIRED | Self::PRESENT
    }
}

bitflags! {
    pub flags GdtDataEntryAccess: u8 {

        /// The accessed flag is set by the processor on first use.
        const ACCESSED = 1 << 0,

        /// For code segments, this controls read access.  For data segments, it controls write
        /// access.
        const WRITE = 1 << 1,

        /// Limit grows down from base.
        const DIRECTION = 1 << 2,

        /// Must be set to 1 to be valid.
        const _REQUIRED = 1 << 4,


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
        Self::_REQUIRED | Self::PRESENT
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
    limit_flags: u8,
    base2: u8,
    phantom: PhantomData<F>,
}


impl<F, A: GdtEntryAccess> GdtEntry<F, A> {

    fn missing() -> Self {
        GdtEntry {
            limit: 0,
            base0: 0,
            base1: 0,
            access: A::base(),
            limit_flags: 0,
            base2: 0,
            phantom: PhantomData
        }
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
