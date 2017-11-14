use core::fmt;

use shared::descriptor;
use shared::PrivilegeLevel;

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
        const TI_GDT = 0 << 2,
        /// Table Indicator (TI) 1 means LDT is used.
        const TI_LDT = 1 << 2,
    }
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
        let r0 = match self.contains(RPL_0) {
            false => "",
            true => "Ring 0 segment selector.",
        };
        let r1 = match self.contains(RPL_1) {
            false => "",
            true => "Ring 1 segment selector.",
        };
        let r2 = match self.contains(RPL_2) {
            false => "",
            true => "Ring 2 segment selector.",
        };
        let r3 = match self.contains(RPL_3) {
            false => "",
            true => "Ring 3 segment selector.",
        };
        let tbl = match self.contains(TI_LDT) {
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
            Type::Data(d) => d.bits | 0b10_000,
            Type::Code(c) => c.bits | 0b11_000,
        }
    }
}


/// Entry for GDT or LDT. Provides size and location of a segment.
///
/// See Intel 3a, Section 3.4.5 "Segment Descriptors", and Section 3.5.2
/// "Segment Descriptor Tables in IA-32e Mode", especially Figure 3-8.
#[derive(Copy, Clone, Debug)]
#[repr(C, packed)]
pub struct SegmentDescriptor {
    limit1: u16,
    base1: u16,
    base2: u8,
    access: descriptor::Flags,
    limit2_flags: Flags,
    base3: u8,
}


/// This data-structure is an ugly mess thing so we provide some
/// convenience function to program it.
impl SegmentDescriptor {
    pub const NULL: SegmentDescriptor = SegmentDescriptor {
        limit1: 0,
        base1: 0,
        base2: 0,
        access: descriptor::Flags::BLANK,
        limit2_flags: Flags::BLANK,
        base3: 0,
    };

    /// Outputs a memory or TSS descriptor.
    /// For a TSS descriptor on x86-64, you also need a high descriptor as second entry (see below).
    pub(crate) fn memory_or_tss(base: u32, limit: u32, ty: descriptor::Type, dpl: PrivilegeLevel, flags: Flags) -> SegmentDescriptor {
        let fine_grained = limit < 0x100000;
        let (limit1, limit2) = if fine_grained {
            ((limit & 0xFFFF) as u16, ((limit & 0xF0000) >> 16) as u8)
        } else {
            if ((limit - 0xFFF) & 0xFFF) > 0 {
                panic!("bad segment limit for GDT entry");
            }
            (((limit & 0xFFFF000) >> 12) as u16, ((limit & 0xF0000000) >> 28) as u8)
        };
        SegmentDescriptor {
            limit1: limit1,
            base1: base as u16,
            base2: ((base as usize & 0xFF0000) >> 16) as u8,
            access: descriptor::Flags::from_type(ty)
                |   descriptor::Flags::from_priv(dpl)
                |   descriptor::FLAGS_PRESENT,
            limit2_flags: if fine_grained { Flags::empty() } else { FLAGS_G }
                | flags
                | Flags::from_limit2(limit2),
            base3: ((base as usize & 0xFF000000) >> 24) as u8,
        }
    }

    /// Outputs a descriptor containing the high 32 bits of a memory address.
    /// Serves as the second entry for descriptors that consume 2 table entries in x86-64.
    pub(crate) const fn high(address: u64) -> SegmentDescriptor {
        SegmentDescriptor {
            limit1: (address >> 32) as u16,
            base1: (address >> 48) as u16,
            base2: 0,
            access: descriptor::Flags::BLANK,
            limit2_flags: Flags::BLANK,
            base3: 0,
        }
    }
}

bitflags! {
    pub flags Flags: u8 {
        /// Available for use by system software.
        const FLAGS_AVL  = 1 << 4,
        /// 64-bit code segment (IA-32e mode only).
        const FLAGS_L    = 1 << 5,
        /// Default operation size (0 = 16-bit segment, 1 = 32-bit segment).
        const FLAGS_DB   = 1 << 6,
        /// Granularity (0 = limit in bytes, 1 = limit in 4 KiB Pages).
        const FLAGS_G    = 1 << 7,

    }
}

impl Flags {
    pub const BLANK: Flags = Flags { bits: 0 };

    pub fn from_limit2(limit2: u8) -> Flags {
        assert_eq!(limit2 & !0b1111, 0);
        Flags { bits: limit2 }
    }
}
