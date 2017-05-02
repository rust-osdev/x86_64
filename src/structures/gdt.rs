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
#[derive(Clone, Copy)]
pub struct SegmentSelector(pub u16);

impl SegmentSelector {
    /// Creates a new SegmentSelector
    ///
    /// # Arguments
    ///  * `index`: index within in the GDT or LDT.  If 0, doesn't
    ///  actually select a segment, but indicates an invalid selector.
    ///  * `rpl`: the requested privilege level
    ///  * `local`: If true, the request is for the LDT.
    pub const fn new(index: u16, rpl: PrivilegeLevel, local: bool) -> SegmentSelector {
        SegmentSelector(index << 3 | (rpl as u16) | (local as u16) << 2)
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
    fn new() -> Self;

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
        /// Not present.
        const NOTPRESENT = 0,

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
    fn new() -> Self {
        Self::_EXECUTABLE | Self::_NONSYSTEM | Self::PRESENT
    }
}

bitflags! {
    pub flags GdtDataEntryAccess: u8 {
        /// Not present.
        const NOTPRESENT = 0,

        /// The accessed flag is set by the processor on first use.
        const ACCESSED = 1 << 0,

        /// Controls write access.
        const WRITE = 1 << 1,

        /// Limit grows down from base.
        const DIRECTION = 1 << 2,

        /// Must be set to 1 to be a regular segment.
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
    fn new() -> Self {
        Self::_NONSYSTEM | Self::PRESENT
    }
}

bitflags! {
    pub flags GdtSystemEntryAccess: u8 {
        /// Not present.
        const NOTPRESENT = 0,

        /// Must be set to 0 to be valid for system entries.
        const _NONSYSTEM = 1 << 4,


        /// Should be set if the segment is valid.
        const PRESENT = 1 << 7,

        /// Long mode extended base address for previous entry.
        const UpperBits =     0,
        const LDT =           0b0010,
        const TSS64 =         0b1001,
        const TSS64Busy =     0b1011,
        const CallGate64 =    0b1100,
        const IntGate64 =     0b1110,
        const TrapGate64 =    0b1111,

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

    /// Type will default to UpperBits.  UpperBits entries are not marked present, so this amounts
    /// to a full 32-bits of 0.
    fn new() -> Self {
        Self::UpperBits
    }
}

/// Various constructors for different types of system segments.
///
impl GdtSystemEntryAccess {
    /// Returns a new Upper Bits segment entry by itself.  There are only a few cases where this
    /// should be used outside this module.
    fn new_UpperBits() -> Self {
        Self::new()
    }
    /// Returns a new LDT segment entry.
    fn new_LDT() -> Self {
        Self::PRESENT | Self::LDT
    }
    /// Returns a TSS64 Entry
    fn new_TSS64() -> Self {
        Self::PRESENT | Self::TSS64
    }
    /// Returns a TSS64 Entry, marked busy.  Usually, you wouldn't actually initialize something
    /// this way.
    fn new_TSS64Busy() -> Self {
        Self::PRESENT | Self::TSS64Busy
    }
    /// For a call gate
    fn new_CallGate64() -> Self {
        Self::PRESENT | Self::CallGate64
    }
    /// For an interrupt gate
    fn new_IntGate64() -> Self {
        Self::PRESENT | Self::IntGate64
    }
    /// For a trap gate
    fn new_TrapGate64() -> Self {
        Self::PRESENT | Self::TrapGate64
    }
}

/// Flag set for flag byte.
///
bitflags! {
    pub flags GdtFlags: u8 {
        const NEW = 0,

        const ACCESS_BITS = 0xf,
        const FLAGS_BITS = 0xf0,

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

/*impl From<u8> for GdtFlags {
    fn from(b: u8) -> Self {
        unsafe { transmute::<u8, Self>(b) }
    }
}

impl From<GdtFlags> for u8 {
    fn from(b: GdtFlags) -> u8 {
        unsafe { transmute::<GdtFlags, u8>(b) }
    }
}*/



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
            access: A::new(),
            limit_flags: GdtFlags::NEW,
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

    /// The limit has to be in the low 20 bits.  If the limit is above 1MB, sets granularity bit
    /// and shifts the size down by 12 bits.
    pub fn set_limit(&mut self, full_limit: u32) {

        let mut granularity = GdtFlags::NEW;
        let mut limit = full_limit;
        if limit > 0xfffff {
            limit = limit >> 12;
            granularity = GdtFlags::GRANULARITY;
        }
        self.limit = (limit & 0xffff) as u16;
        self.limit_flags = granularity | ((self.limit_flags & GdtFlags::FLAGS_BITS) | (GdtFlags::ACCESS_BITS & unsafe { transmute::<u8, GdtFlags>((limit >> 16) as u8) } ))
    }

    pub fn set_long_mode(&mut self) {
        self.limit_flags = self.limit_flags | GdtFlags::LONG_MODE;
    }
}

/// A Global Descriptor Table entry for a call gate.
///
#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
pub struct GdtCallGate64<F> {
    offset0: u16,
    segment: SegmentSelector,
    _res0: u8,
    access: GdtSystemEntryAccess,
    offset1: u16,
    offset2: u32,
    _res1: u8,
    _access_fake: GdtSystemEntryAccess,
    _res2: u16,
    phantom: PhantomData<F>,
}


impl<F> GdtCallGate64<F> {

    /// Creates an empty GdtEntry
    pub fn missing() -> Self {
        GdtCallGate64 {
            offset0: 0,
            offset1: 0,
            offset2: 0,
            segment: SegmentSelector::new(0, PrivilegeLevel::Ring0, false),
            _res0: 0,
            _res1: 0,
            _res2: 0,
            access: GdtSystemEntryAccess::new_CallGate64(),
            _access_fake: GdtSystemEntryAccess::new_UpperBits(),
            phantom: PhantomData
        }
    }

    /// Sets the base address for the segment.  Only sets the low 32 bits.  For system segments
    /// it will be necessary to set the high bits in the following 8-byte field.
    pub fn set_offset(&mut self, offset_addr: u64) {
        self.offset0 = (offset_addr & 0xffff) as u16;
        self.offset1 = (offset_addr & 0xffff0000 >> 16) as u16;
        self.offset2 = (offset_addr & 0xffffffff00000000 >> 32) as u32;
    }

    pub fn set_segment(&mut self, segment: SegmentSelector) {
        self.segment = segment.clone();
    }

}

/// A Global Descriptor Table entry for a interrupt and trap gates.  From the GDT's perspective,
/// they have exactly the same format, and only differ by type (one bit).
///
#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
pub struct GdtIntTrapGate64<F> {
    offset0: u16,
    segment: SegmentSelector,
    ist: u8,    // Low three bits only
    access: GdtSystemEntryAccess,
    offset1: u16,
    offset2: u32,
    _res0: u32,
    phantom: PhantomData<F>,
}


impl<F> GdtIntTrapGate64<F> {

    /// Creates an empty GdtEntry
    pub fn missing() -> Self {
        GdtIntTrapGate64 {
            offset0: 0,
            offset1: 0,
            offset2: 0,
            segment: SegmentSelector::new(0, PrivilegeLevel::Ring0, false),
            _res0: 0,
            access: GdtSystemEntryAccess::new_IntGate64(),
            ist: 0,
            phantom: PhantomData
        }
    }

    /// Sets the base address for the segment.  Only sets the low 32 bits.  For system segments
    /// it will be necessary to set the high bits in the following 8-byte field.
    pub fn set_offset(&mut self, offset_addr: u64) {
        self.offset0 = (offset_addr & 0xffff) as u16;
        self.offset1 = (offset_addr & 0xffff0000 >> 16) as u16;
        self.offset2 = (offset_addr & 0xffffffff00000000 >> 32) as u32;
    }

    /// The segment to switch to.
    pub fn set_segment(&mut self, segment: SegmentSelector) {
        self.segment = segment.clone();
    }

    /// The IST entry for the new stack.
    pub fn set_ist(&mut self, ist: u8) {

        self.ist = ist & 0b111;
    }

    /// Sets the gate as a trap gate instead of an interrupt gate.
    pub fn set_trap(&mut self) {
        let current_dpl = self.access.get_dpl();
        self.access = GdtSystemEntryAccess::new_TrapGate64().set_dpl(current_dpl)
    }

    /// Sets the gate as an interrupt gate instead of a trap gate.
    pub fn set_int(&mut self) {
        let current_dpl = self.access.get_dpl();
        self.access = GdtSystemEntryAccess::new_IntGate64().set_dpl(current_dpl)
    }
}


/// A Global Descriptor Table entry for a TSS.
///
#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
pub struct GdtTSS64<F> {
    base_entry: GdtEntry<F, GdtSystemEntryAccess>,
    base_extended: u32,
    _res: u32,
    phantom: PhantomData<F>,
}


impl<F> GdtTSS64<F> {

    /// Creates an empty GdtEntry
    pub fn missing() -> Self {
        GdtTSS64 {
            base_entry: GdtEntry::missing(),
            base_extended: 0,
            _res: 0,
            phantom: PhantomData
        }
    }

    /// Sets the base address for the segment.  Only sets the low 32 bits.  For system segments
    /// it will be necessary to set the high bits in the following 8-byte field.
    pub fn set_base(&mut self, base_addr: u64) {
        self.base_entry.set_base((base_addr & 0xffffffff) as u32);
        self.base_extended = ((base_addr & 0xffffffff00000000) >> 32) as u32;
    }

    pub fn set_limit(&mut self, limit: u32) {
        self.base_entry.set_limit(limit)
    }

}

/// A Global Descriptor Table entry for a call gate.
///
#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
pub struct GdtUpperBits<F> {
    upper_bits: u32,
    _res0: u8,
    access: GdtSystemEntryAccess,
    _res1: u16,
    phantom: PhantomData<F>,
}


impl<F> GdtUpperBits<F> {

    /// Creates an empty GdtEntry
    pub fn missing() -> Self {
        GdtUpperBits {
            upper_bits: 0,
            _res0: 0,
            _res1: 0,
            access: GdtSystemEntryAccess::new_CallGate64(),
            phantom: PhantomData
        }
    }

    /// Sets the base address for the segment.  Only sets the low 32 bits.  For system segments
    /// it will be necessary to set the high bits in the following 8-byte field.
    pub fn set_upper(&mut self, upper_bits: u32) {
        self.upper_bits = upper_bits;
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
