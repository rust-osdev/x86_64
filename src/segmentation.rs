//! Program x86 segmentation hardware.

use core::fmt;

/// Specifies which element to load into a segment from
/// descriptor tables (i.e., is a index to LDT or GDT table
/// with some additional flags).
bitflags! {
    flags SegmentSelector: u16 {
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

impl SegmentSelector {
    /// Create a new SegmentSelector
    ///
    /// # Arguments
    ///  * `index` index in GDT or LDT array.
    ///
    pub fn new(index: u16) -> SegmentSelector {
        SegmentSelector { bits: index << 3 }
    }

    pub fn from_raw(bits: u16) -> SegmentSelector {
        SegmentSelector { bits: bits }
    }
}

impl fmt::Debug for SegmentSelector {
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


/// Entry for GDT or LDT. Provides size and location of a segment.
bitflags! {
    flags SegmentDescriptor: u64 {
        /// Descriptor type (0 = system; 1 = code or data).
        const DESC_S    = 1 << (32+12),
        /// Descriptor privilege level 0.
        const DESC_DPL0 = 0b00 << (32+13),
        /// Descriptor privilege level 1.
        const DESC_DPL1 = 0b01 << (32+13),
        /// Descriptor privilege level 2.
        const DESC_DPL2 = 0b10 << (32+13),
        /// Descriptor privilege level 3.
        const DESC_DPL3 = 0b11 << (32+13),
        /// Descriptor is Present.
        const DESC_P = 1 << (32+15),
        /// Available for use by system software.
        const DESC_AVL  = 1 << (32+20),
        /// 64-bit code segment (IA-32e mode only).
        const DESC_L    = 1 << (32+21),
        /// Default operation size (0 = 16-bit segment, 1 = 32-bit segment)
        const DESC_DB   = 1 << (32+22),
        ///  Granularity.
        const DESC_G    = 1 << (32+23),

        // System-Segment and Gate-Descriptor Types for IA32e mode.
        // When the S (descriptor type) flag in a segment descriptor is clear,
        // the descriptor type is a system descriptor.

        const TYPE_SYS_LDT = 0b0010 << (32+8),
        const TYPE_SYS_TSS_AVAILABLE = 0b1001 << (32+8),
        const TYPE_SYS_TSS_BUSY = 0b1011 << (32+8),
        const TYPE_SYS_CALL_GATE = 0b1100 << (32+8),
        const TYPE_SYS_INTERRUPT_GATE = 0b1110 << (32+8),
        const TYPE_SYS_TRAP_GATE = 0b1111 << (32+8),

        // Code- and Data-Segment Descriptor Types.
        // When the S (descriptor type) flag in a segment descriptor is set,
        // the descriptor is for either a code or a data segment.

        /// Data Read-Only
        const TYPE_D_RO = 0b0000 << (32+8),
        /// Data Read-Only, accessed
        const TYPE_D_ROA = 0b0001 << (32+8),
        /// Data Read/Write
        const TYPE_D_RW = 0b0010 << (32+8),
        /// Data Read/Write, accessed
        const TYPE_D_RWA = 0b0011 << (32+8),
        /// Data Read-Only, expand-down
        const TYPE_D_ROEXD = 0b0100 << (32+8),
        /// Data Read-Only, expand-down, accessed
        const TYPE_D_ROEXDA = 0b0101 << (32+8),
        /// Data Read/Write, expand-down
        const TYPE_D_RWEXD = 0b0110 << (32+8),
        /// Data Read/Write, expand-down, accessed
        const TYPE_D_RWEXDA = 0b0111 << (32+8),

        /// Code Execute-Only
        const TYPE_C_EO = 0b1000 << (32+8),
        /// Code Execute-Only, accessed
        const TYPE_C_EOA = 0b1001 << (32+8),
        /// Code Execute/Read
        const TYPE_C_ER = 0b1010 << (32+8),
        /// Code Execute/Read, accessed
        const TYPE_C_ERA = 0b1011 << (32+8),
        /// Code Execute-Only, conforming
        const TYPE_C_EOC = 0b1100 << (32+8),
        /// Code Execute-Only, conforming, accessed
        const TYPE_C_EOCA = 0b1101 << (32+8),
        /// Code Execute/Read, conforming
        const TYPE_C_ERC = 0b1110 << (32+8),
        /// Code Execute/Read, conforming, accessed
        const TYPE_C_ERCA = 0b1111 << (32+8),
    }
}

/// This is data-structure is a ugly mess thing so we provide some
/// convenience function to program it.
impl SegmentDescriptor {
    pub fn new(base: u32, limit: u32) -> SegmentDescriptor {
        let base_low: u64 = base as u64 & 0xffffff;
        let base_high: u64 = (base as u64 >> 24) & 0xff;

        let limit_low: u64 = limit as u64 & 0xffff;
        let limit_high: u64 = (limit as u64 & (0b1111 << 16)) >> 16;

        SegmentDescriptor {
            bits: limit_low | base_low << 16 | limit_high << (32 + 16) | base_high << (32 + 24),
        }
    }
}

impl fmt::Debug for SegmentDescriptor {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "SD: 0x{:x}", self.bits)
    }
}

/// Reload stack segment register.
pub unsafe fn load_ss(sel: SegmentSelector) {
    asm!("movw $0, %ss " :: "r" (sel) : "memory");
}

/// Reload data segment register.
pub unsafe fn load_ds(sel: SegmentSelector) {
    asm!("movw $0, %ds " :: "r" (sel) : "memory");
}

/// Reload es segment register.
pub unsafe fn load_es(sel: SegmentSelector) {
    asm!("movw $0, %es " :: "r" (sel) : "memory");
}

/// Reload fs segment register.
pub unsafe fn load_fs(sel: SegmentSelector) {
    asm!("movw $0, %fs " :: "r" (sel) : "memory");
}

/// Reload gs segment register.
pub unsafe fn load_gs(sel: SegmentSelector) {
    asm!("movw $0, %gs " :: "r" (sel) : "memory");
}

/// Reload code segment register.
/// Note this is special since we can not directly move
/// to %cs. Instead we push the new segment selector
/// and return value on the stack and use lretq
/// to reload cs and continue at 1:.
pub unsafe fn load_cs(sel: SegmentSelector) {
    asm!("pushq $0
         lea 1f(%rip), %rax
         pushq %rax
         lretq
         1:" :: "r" (sel.bits() as u64) : "{rax}" "memory");
}
