//! Types for the Global Descriptor Table and segment selectors.

use core::fmt;
use core::convert::{From,Into};
use core::intrinsics::transmute;
use core::mem::size_of;
use PrivilegeLevel;
use bit_field::BitField;

use instructions::tables::{lgdt,DescriptorTablePointer};
use registers::msr;

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
        SegmentSelector((index << 3) | (rpl as u16) | ((local as u16) << 2))
    }

    /// Returns the GDT index.
    pub fn index(&self) -> u16 {
        self.0 >> 3
    }

    /// Returns the requested privilege level.
    pub fn rpl(&self) -> PrivilegeLevel {
        PrivilegeLevel::from_uint(self.0.get_bits(0..2) as u8)
    }

    /// Returns true if the selector is for the LDT.  Otherwise returns false for GDT.
    pub fn local(&self) -> bool {
        (self.0 & 0b100) > 0
    }

    pub fn as_int(self) -> u16 {
        self.0
    }
}

impl fmt::Debug for SegmentSelector {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut s = f.debug_struct("SegmentSelector");
        s.field("index", &self.index());
        s.field("local", &self.local());
        s.field("rpl", &self.rpl());
        s.finish()
    }
}

impl fmt::Display for SegmentSelector {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "<{}, local: {}, rpl: {}>", self.index(), self.local(), self.rpl() as u8)
    }
}

impl fmt::Binary for SegmentSelector {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:b}",  self.0)
    }
}


/// A generic access byte trait.
pub trait GdtAccess : Sized + Into<u8> + From<u8> {
    
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
    pub flags GdtCodeAccess: u8 {
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


impl From<u8> for GdtCodeAccess {
    fn from(b: u8) -> Self {
        unsafe { transmute::<u8, Self>(b) }
    }
}

impl From<GdtCodeAccess> for u8 {
    fn from(b: GdtCodeAccess) -> Self {
        unsafe { transmute::<GdtCodeAccess, Self>(b) }
    }
}

impl GdtAccess for GdtCodeAccess {
    fn new() -> Self {
        Self::_EXECUTABLE | Self::_NONSYSTEM | Self::PRESENT
    }
}

bitflags! {
    pub flags GdtDataAccess: u8 {
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
impl From<u8> for GdtDataAccess {
    fn from(b: u8) -> Self {
        unsafe { transmute::<u8, Self>(b) }
    }
}

impl From<GdtDataAccess> for u8 {
    fn from(b: GdtDataAccess) -> Self {
        unsafe { transmute::<GdtDataAccess, Self>(b) }
    }
}


impl GdtAccess for GdtDataAccess {
    fn new() -> Self {
        Self::_NONSYSTEM | Self::PRESENT
    }
}

impl GdtDataAccess {
    fn set_write(&mut self) {
        *self = GdtDataAccess::WRITE | *self;
    }
}

bitflags! {
    pub flags GdtSystemAccess: u8 {
        /// Not present.
        const NOTPRESENT = 0,

        /// Must be set to 0 to be valid for system entries.
        const _NONSYSTEM = 1 << 4,


        /// Should be set if the segment is valid.
        const PRESENT = 1 << 7,

        /// Long mode extended base address for previous entry.
        const UPPERBITS =     0,
        const LDT =           0b0010,
        const TSS64 =         0b1001,
        const TSS64_BUSY =    0b1011,
        const CALLGATE64 =    0b1100,
        const INTGATE64 =     0b1110,
        const TRAPGATE64 =    0b1111,

    }
}


impl From<u8> for GdtSystemAccess {
    fn from(b: u8) -> Self {
        unsafe { transmute::<u8, Self>(b) }
    }
}

impl From<GdtSystemAccess> for u8 {
    fn from(b: GdtSystemAccess) -> Self {
        unsafe { transmute::<GdtSystemAccess, Self>(b) }
    }
}

impl GdtAccess for GdtSystemAccess {

    /// Type will default to UpperBits.  UpperBits entries are not marked present, so this amounts
    /// to a full 32-bits of 0.
    fn new() -> Self {
        Self::UPPERBITS
    }
}

/// Various constructors for different types of system segments.
///
impl GdtSystemAccess {
    /// Returns a new Upper Bits segment entry by itself.  There are only a few cases where this
    /// should be used outside this module.
    pub fn new_upper_bits() -> Self {
        Self::new()
    }
    /// Returns a new LDT segment entry.
    pub fn new_ldt() -> Self {
        Self::PRESENT | Self::LDT
    }
    /// Returns a TSS64 Entry
    pub fn new_tss64() -> Self {
        Self::PRESENT | Self::TSS64
    }
    /// Returns a TSS64 Entry, marked busy.  Usually, you wouldn't actually initialize something
    /// this way.
    pub fn new_tss64_busy() -> Self {
        Self::PRESENT | Self::TSS64_BUSY
    }
    /// For a call gate
    pub fn new_call_gate64() -> Self {
        Self::PRESENT | Self::CALLGATE64
    }
    /// For an interrupt gate
    pub fn new_int_gate64() -> Self {
        Self::PRESENT | Self::INTGATE64
    }
    /// For a trap gate
    pub fn new_trap_gate64() -> Self {
        Self::PRESENT | Self::TRAPGATE64
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

impl From<u8> for GdtFlags {
    fn from(b: u8) -> Self {
        unsafe { transmute::<u8, Self>(b) }
    }
}

impl From<GdtFlags> for u8 {
    fn from(b: GdtFlags) -> u8 {
        unsafe { transmute::<GdtFlags, u8>(b) }
    }
}



/// A Global Descriptor Table entry.
///
#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
pub struct GdtEntry<A: GdtAccess> {
    limit: u16,
    base0: u16,
    base1: u8,
    access: A,
    limit_flags: GdtFlags,
    base2: u8,
}

impl<A: GdtAccess + Clone> GdtEntry<A> {

    /// Creates an empty GdtEntry
    pub fn missing() -> Self {
        GdtEntry {
            limit: 0,
            base0: 0,
            base1: 0,
            access: A::new(),
            limit_flags: GdtFlags::NEW,
            base2: 0,
        }
    }

    /// Sets the base address for the segment.  Only sets the low 32 bits.  For system segments
    /// it will be necessary to set the high bits in the following 8-byte field.
    pub fn set_base(&mut self, base_addr: u32) {
        self.base0 = (base_addr & 0xffff) as u16;
        self.base1 = ((base_addr & 0xff0000) >> 16) as u8;
        self.base2 = ((base_addr & 0xff000000) >> 24) as u8;
    }

    /// The limit has to be in the low 20 bits.  If the limit is above 1MB, sets granularity bit
    /// and shifts the size down by 12 bits.
    pub fn set_limit(&mut self, full_limit: u32) {

        let mut granularity = GdtFlags::NEW;
        let mut limit = full_limit;

        // In order to make a segment larger than 1MB, the GDT uses the granularity flag to set the
        // resolution to 4KB instead of 1 byte.
        if limit > 0xfffff {
            limit = limit >> 12;
            granularity = GdtFlags::GRANULARITY;
        }
        self.limit = (limit & 0xffff) as u16;
        self.limit_flags = granularity | ((self.limit_flags & GdtFlags::FLAGS_BITS) | (GdtFlags::ACCESS_BITS & ((limit >> 16) as u8).into() ))
    }

    /// Set the long mode bit on the segment.
    pub fn set_long_mode(&mut self) {
        self.limit_flags = self.limit_flags | GdtFlags::LONG_MODE;
    }

    /// Returns a new segment descriptor entry like the original with the dpl set as specified.
    pub fn set_dpl(&self, dpl: PrivilegeLevel) -> Self {
        let mut new_entry = self.clone();

        new_entry.access = self.access.clone();
        new_entry.access.set_dpl(dpl);
        new_entry
    }

    /// Returns the dpl of the segment descriptor entry.
    pub fn get_dpl(self) -> PrivilegeLevel {
        self.access.get_dpl()
    }
}

impl fmt::Binary for GdtEntry<GdtDataAccess> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let b = self as *const _ as *const u64;
        write!(f, "{:016b}",  unsafe {*b})
    }
}
impl fmt::LowerHex for GdtEntry<GdtDataAccess> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let b = self as *const _ as *const u64;
        write!(f, "{:016x}",  unsafe {*b})
    }
}
impl fmt::Binary for GdtEntry<GdtCodeAccess> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let b = self as *const _ as *const u64;
        write!(f, "{:016b}",  unsafe {*b})
    }
}
impl fmt::LowerHex for GdtEntry<GdtCodeAccess> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let b = self as *const _ as *const u64;
        write!(f, "{:016x}",  unsafe {*b})
    }
}


/// A Global Descriptor Table entry for a call gate.
///
#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
pub struct GdtCallGate64 {
    offset0: u16,
    segment: SegmentSelector,
    _res0: u8,
    access: GdtSystemAccess,
    offset1: u16,
    offset2: u32,
    _res1: u8,
    _access_fake: GdtSystemAccess,
    _res2: u16,
}


impl GdtCallGate64 {

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
            access: GdtSystemAccess::new_call_gate64(),
            _access_fake: GdtSystemAccess::new_upper_bits(),
        }
    }

    /// Sets the base address for the segment.  Only sets the low 32 bits.  For system segments
    /// it will be necessary to set the high bits in the following 8-byte field.
    pub fn set_offset(&mut self, offset_addr: u64) {
        self.offset0 = (offset_addr & 0xffff) as u16;
        self.offset1 = (offset_addr & 0xffff0000 >> 16) as u16;
        self.offset2 = (offset_addr & 0xffffffff00000000 >> 32) as u32;
    }

    /// Sets the segment selector of the gate.
    pub fn set_segment(&mut self, segment: SegmentSelector) {
        self.segment = segment.clone();
    }

}

/// A Global Descriptor Table entry for a interrupt and trap gates.  From the GDT's perspective,
/// they have exactly the same format, and only differ by type (one bit).
///
#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
pub struct GdtIntTrapGate64 {
    offset0: u16,
    segment: SegmentSelector,
    ist: u8,    // Low three bits only
    access: GdtSystemAccess,
    offset1: u16,
    offset2: u32,
    _res0: u32,
}


impl GdtIntTrapGate64 {

    /// Creates an empty GdtEntry
    pub fn missing() -> Self {
        GdtIntTrapGate64 {
            offset0: 0,
            offset1: 0,
            offset2: 0,
            segment: SegmentSelector::new(0, PrivilegeLevel::Ring0, false),
            _res0: 0,
            access: GdtSystemAccess::new_int_gate64(),
            ist: 0,
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
        self.access = GdtSystemAccess::new_trap_gate64().set_dpl(current_dpl)
    }

    /// Sets the gate as an interrupt gate instead of a trap gate.
    pub fn set_int(&mut self) {
        let current_dpl = self.access.get_dpl();
        self.access = GdtSystemAccess::new_int_gate64().set_dpl(current_dpl)
    }
}


/// A Global Descriptor Table entry for a TSS.
///
#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
pub struct GdtTSS64 {
    base_entry: GdtEntry<GdtSystemAccess>,
    base_extended: u32,
    _res: u32,
}


impl GdtTSS64 {

    /// Creates an empty GdtEntry
    pub fn missing() -> Self {
        let mut tss = GdtTSS64 {
            base_entry: GdtEntry::missing(),
            base_extended: 0,
            _res: 0,
        };

        tss.base_entry.access = GdtSystemAccess::new_tss64();

        tss
    }

    /// Sets the base address for the segment.  Only sets the low 32 bits.  For system segments
    /// it will be necessary to set the high bits in the following 8-byte field.
    pub fn set_base(&mut self, base_addr: u64) {
        self.base_entry.set_base((base_addr & 0xffffffff) as u32);
        self.base_extended = ((base_addr & 0xffffffff00000000) >> 32) as u32;
    }

    /// Sets the limit for the TSS.
    pub fn set_limit(&mut self, limit: u32) {
        self.base_entry.set_limit(limit)
    }

}


impl fmt::Binary for GdtTSS64 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let b0 = self as *const _ as *const u64;
        let b1 = ((self as *const _) as u64 + 8) as *const u64;
        write!(f, "{:064b}{:064b}",  unsafe {*b1}, unsafe {*b0})
    }
}
impl fmt::LowerHex for GdtTSS64 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let b0 = self as *const _ as *const u64;
        let b1 = ((self as *const _) as u64 + 8) as *const u64;
        write!(f, "{:016x}{:016x}",  unsafe {*b1}, unsafe {*b0})
    }
}


/// A Global Descriptor Table entry for a call gate.
///
#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
pub struct GdtUpperBits {
    upper_bits: u32,
    _res0: u8,
    access: GdtSystemAccess,
    _res1: u16,
}

impl GdtUpperBits {

    /// Creates an empty GdtEntry
    pub fn missing() -> Self {
        GdtUpperBits {
            upper_bits: 0,
            _res0: 0,
            _res1: 0,
            access: GdtSystemAccess::new_call_gate64(),
        }
    }

    /// Sets the base address for the segment.  Only sets the low 32 bits.  For system segments
    /// it will be necessary to set the high bits in the following 8-byte field.
    pub fn set_upper(&mut self, upper_bits: u32) {
        self.upper_bits = upper_bits;
    }


}

/// Some generic functions for particular GDT structures.
pub trait Gdt: Sized {

    /// Loads the gdtr to point to the gdt.
    fn load(&'static self) {
        let ptr = DescriptorTablePointer {
            base: self as *const _ as u64,
            limit: (size_of::<Self>() - 1) as u16,
        };

        unsafe { lgdt(&ptr) };
    }
}

/// The segment descriptor number is equal to the byte offset within the GDT, or the byte offset +
/// 0b100 within the LDT, minus the rpl.  To set the rpl, call the macro, and then use the .rpl()
/// method.  To get a descriptor for an LDT entry, use the .local() method on the result of the
/// macro.
#[macro_export]
macro_rules! segment_of {
    ($ty:ty, $field:ident) => {
        SegmentSelector(unsafe { &(*(0 as *const $ty)).$field as *const _ as u16 })
    }
}

/// A minimal GDT suitable for use with the syscall/sysret pair possessing a single TSS.  Can be
/// embedded at the top of a larger GDT if additional entries are needed.
#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
pub struct GdtSyscall {
    invl: u64,                              // selector 0 isn't valid, but still takes up space.
    pub r0_64cs: GdtEntry<GdtCodeAccess>,       // Ring 0 64-bit CS
    pub r0_64ss: GdtEntry<GdtDataAccess>,       // Ring 0 64-bit DS/SS
    pub r3_32cs: GdtEntry<GdtCodeAccess>,       // Ring 3 32-bit CS
    pub r3_64ss: GdtEntry<GdtDataAccess>,       // Ring 3 64-bit SS
    pub r3_64cs: GdtEntry<GdtCodeAccess>,       // Ring 3 64-bit CS
    pub tss: GdtTSS64,                          // TSS
}

impl GdtSyscall {

    /// Initializes a new basic GDT suitable for syscall/sysret operations.
    pub fn new() -> Self {
        let mut code_seg32 = GdtEntry::<GdtCodeAccess>::missing();
        let mut data_seg32 = GdtEntry::<GdtDataAccess>::missing();

        data_seg32.access.set_write();

        let mut code_seg64 = code_seg32.clone();
        let mut data_seg64 = data_seg32.clone();
        code_seg64.set_long_mode();
        data_seg64.set_long_mode();

        let gdt = GdtSyscall {
            invl: 0,
            r0_64cs: code_seg64.clone(),
            r0_64ss: data_seg64.clone(),
            r3_32cs: code_seg32.set_dpl(PrivilegeLevel::Ring3),
            r3_64ss: data_seg32.set_dpl(PrivilegeLevel::Ring3),
            r3_64cs: code_seg64.set_dpl(PrivilegeLevel::Ring3),
            tss: GdtTSS64::missing(),
        };

        gdt
    }

    /// Loads the MSRs for the system calls.
    pub fn syscall_setup(&self, entry_point: extern "C" fn(), rflags_mask: u64) {
        // The STAR defines segment entries for the syscall as follows:
        //    bits[47:32]:      cs
        //    bits[47:32] + 8:  ss
        //    bits[63:48]:      32-bit cs
        //    bits[63:48] + 8:  ss
        //    bits[63:48] + 16: 64-bit cs

        let star = ((segment_of!(GdtSyscall, r0_64cs).as_int() as u64) << 16) | ((segment_of!(GdtSyscall, r3_32cs).as_int() as u64) << 32);
        unsafe {

            msr::wrmsr(msr::IA32_STAR, star);
            msr::wrmsr(msr::IA32_LSTAR, entry_point as u64);
            msr::wrmsr(msr::IA32_FMASK, rflags_mask as u64);

            // Set syscall enable bit
            msr::wrmsr(msr::IA32_EFER, msr::rdmsr(msr::IA32_EFER) | 1);

        }
    }
}

impl Gdt for GdtSyscall {}

