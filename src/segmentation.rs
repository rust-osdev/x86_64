use core::fmt;
use core::iter::{Chain, IntoIterator, Iterator, Once, once};
use core::option::IntoIter;

use PrivilegeLevel;

/// Specifies which element to load into a segment from
/// descriptor tables (i.e., is a index to LDT or GDT table
/// with some additional flags).
///
/// See Intel 3a, Section 3.4.2 "Segment Selectors"
bitflags! {
    #[repr(C, packed)]
    pub flags SegmentSelector: u16 {
        /// Requestor Privilege Level
        const RPL_0 = 0b00,
        const RPL_1 = 0b01,
        const RPL_2 = 0b10,
        const RPL_3 = 0b11,

        /// Table Indicator (TI) 0 means GDT is used.
        const TI_GDT = 0 << 3,
        /// Table Indicator (TI) 1 means LDT is used.
        const TI_LDT = 1 << 3,
    }
}

/// Reload code segment register.
/// Note this is special since we can not directly move
/// to %cs. Instead we push the new segment selector
/// and return value on the stack and use lretq
/// to reload cs and continue at 1:.
pub unsafe fn set_cs(sel: SegmentSelector) {

    #[cfg(target_arch="x86")]
    #[inline(always)]
    unsafe fn inner(sel: SegmentSelector) {
        asm!("pushl $0; \
              pushl $$1f; \
              lretl; \
              1:" :: "ri" (sel.bits() as usize) : "rax" "memory");
    }

    #[cfg(target_arch="x86_64")]
    #[inline(always)]
    unsafe fn inner(sel: SegmentSelector) {
        asm!("pushq $0; \
              leaq  1f(%rip), %rax; \
              pushq %rax; \
              lretq; \
              1:" :: "ri" (sel.bits() as usize) : "rax" "memory");
    }

    inner(sel)
}


impl SegmentSelector {
    /// Create a new SegmentSelector
    ///
    /// # Arguments
    ///  * `index` index in GDT or LDT array.
    ///
    pub const fn new(index: u16, rpl: PrivilegeLevel) -> SegmentSelector {
        SegmentSelector { bits: index << 3 | (rpl as u16) }
    }

    pub const fn from_raw(bits: u16) -> SegmentSelector {
        SegmentSelector { bits: bits }
    }
}

impl fmt::Display for SegmentSelector {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let r0 = match self.contains(SegmentSelector::RPL_0) {
            false => "",
            true => "Ring 0 segment selector.",
        };
        let r1 = match self.contains(SegmentSelector::RPL_1) {
            false => "",
            true => "Ring 1 segment selector.",
        };
        let r2 = match self.contains(SegmentSelector::RPL_2) {
            false => "",
            true => "Ring 2 segment selector.",
        };
        let r3 = match self.contains(SegmentSelector::RPL_3) {
            false => "",
            true => "Ring 3 segment selector.",
        };
        let tbl = match self.contains(SegmentSelector::TI_LDT) {
            false => "GDT Table",
            true => "LDT Table",
        };

        write!(f,
               "Index {} in {}, {}{}{}{}",
               self.bits >> 3,
               tbl,
               r0,
               r1,
               r2,
               r3)
        // write!(f, "Index")
    }
}


/// Reload stack segment register.
pub unsafe fn load_ss(sel: SegmentSelector) {
    asm!("movw $0, %ss " :: "r" (sel.bits()) : "memory");
}

/// Reload data segment register.
pub unsafe fn load_ds(sel: SegmentSelector) {
    asm!("movw $0, %ds " :: "r" (sel.bits()) : "memory");
}

/// Reload es segment register.
pub unsafe fn load_es(sel: SegmentSelector) {
    asm!("movw $0, %es " :: "r" (sel.bits()) : "memory");
}

/// Reload fs segment register.
pub unsafe fn load_fs(sel: SegmentSelector) {
    asm!("movw $0, %fs " :: "r" (sel.bits()) : "memory");
}

/// Reload gs segment register.
pub unsafe fn load_gs(sel: SegmentSelector) {
    asm!("movw $0, %gs " :: "r" (sel.bits()) : "memory");
}

/// Returns the current value of the code segment register.
pub fn cs() -> SegmentSelector {
    let segment: u16;
    unsafe { asm!("mov %cs, $0" : "=r" (segment) ) };
    SegmentSelector::from_raw(segment)
}


bitflags! {
    /// Data segment types. All are readable.
    ///
    /// See Table 3-1, "Code- and Data-Segment Types"
    pub flags DataAccess: u8 {
        /// Segment is writable
        const DATA_WRITE = 1 << 1,
        /// Segment grows down, for stack
        const DATA_EXPAND_DOWN = 1 << 2,
    }
}

bitflags! {
    /// Code segment types. All are executable.
    ///
    /// See Table 3-1, "Code- and Data-Segment Types"
    pub flags CodeAccess: u8 {
        /// Segment is readable
        const CODE_READ = 1 << 1,
        /// Segment is callable from segment with fewer privileges.
        const CODE_CONFORMING = 1 << 2,
    }
}

/// Umbrella Segment Type.
///
/// See Table 3-1, "Code- and Data-Segment Types"
#[repr(u8)]
pub enum Type {
    Data(DataAccess),
    Code(CodeAccess),
}

impl Type {
    pub fn pack(self) -> u8 {
        match self {
            Type::Data(d) => d.bits | 0b0_000,
            Type::Code(c) => c.bits | 0b1_000,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct GdtEntry(u64);

impl GdtEntry {
    pub fn null() -> GdtEntry {
        GdtEntry(0)
    }
}

enum Descriptor {
    UserSegment(GdtEntry),
    SystemSegment(GdtEntry, GdtEntry),
}

pub struct GdtDescriptor(Descriptor);

impl GdtDescriptor {
    pub fn user_segment(typ: Type, present: bool, long_mode: bool) -> GdtDescriptor {
        use bit_field::BitField;

        let mut flags = 0;

        flags.set_bit(44, true); // User segment bit.
        flags.set_range(40..44, typ.pack().into()); // Type bits.
        flags.set_bit(47, present);
        flags.set_bit(53, long_mode);

        GdtDescriptor(Descriptor::UserSegment(GdtEntry(flags)))
    }

    pub fn tss_system_segment(tss: &'static ::task::TaskStateSegment) -> GdtDescriptor {
        use bit_field::BitField;

        let ptr = tss as *const _ as u64;

        let mut low = 0;
        // present
        low.set_bit(47, true);
        // base
        low.set_range(16..40, ptr.get_range(0..24));
        low.set_range(56..64, ptr.get_range(24..32));
        // limit (the -1 is needed because the bound is inclusive)
        low.set_range(0..16, (::core::mem::size_of::<::task::TaskStateSegment>() - 1) as u64);
        // type (0b1001 = available 64-bit TSS)
        low.set_range(40..44, 0b1001);

        let mut high = 0;
        high.set_range(0..32, ptr.get_range(32..64));

        GdtDescriptor(Descriptor::SystemSegment(GdtEntry(low), GdtEntry(high)))
    }
}

impl IntoIterator for GdtDescriptor {
    type Item = GdtEntry;
    type IntoIter = Chain<Once<GdtEntry>, IntoIter<GdtEntry>>;

    fn into_iter(self) -> Self::IntoIter {
        match self.0 {
            Descriptor::UserSegment(u) => once(u).chain(None),
            Descriptor::SystemSegment(s1, s2) => once(s1).chain(Some(s2)),
        }
    }
}
