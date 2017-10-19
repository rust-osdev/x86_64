//! Fields which are common to all segment-section and gate descriptors

use shared::PrivilegeLevel;
use shared::segmentation;

/// System-Segment and Gate-Descriptor Types for IA32e mode.  When the `S`
/// (descriptor type) flag in a segment descriptor is clear, the descriptor type
/// is a system descriptor.
///
/// See Intel manual 3a, 3.5 "System Descriptor Types", and Table 3-2,
/// System-Segment and Gate-Descriptor Types".
#[repr(u8)]
pub enum SystemType {
    // Reserved = 0
    TssAvailable = 1,
    /// Only for 16-bit
    LocalDescriptorTable = 2,
    TssBusy = 3,
    CallGate = 4,
    /// Only for 16-bit
    TaskGate = 5,
    InterruptGate = 6,
    TrapGate = 7,
}

/// A high-level representation of a descriptor type. One can convert to and
/// from the `Flags` bitfield to encode/decode an actual descriptor.
#[repr(u8)]
pub enum Type {
    SystemDescriptor {
        /// false/0: 16-bit
        /// true/1: native (32- or 64-bit)
        size: bool,
        ty: SystemType
    },
    SegmentDescriptor {
        ty: segmentation::Type,
        accessed: bool
    }
}

impl Type {
    pub fn pack(self) -> u8 {
        match self {
            Type::SystemDescriptor { size, ty } =>
                (size as u8) << 3 | (ty as u8) | FLAGS_TYPE_SYS.bits,
            Type::SegmentDescriptor { ty, accessed } =>
                (accessed as u8)  | ty.pack()  | FLAGS_TYPE_SEG.bits,
        }
    }
}


bitflags!{
    /// Actual encoding of the flags in byte 6 common to all descriptors.
    ///
    /// See Intel manual 3a, figures 3-8, 6-2, and 6-7.
    pub flags Flags: u8 {
        /// Descriptor is Present.
        const FLAGS_PRESENT = 1 << 7,

        // Descriptor privilege level
        const FLAGS_DPL_RING_0 = 0b00 << 5,
        const FLAGS_DPL_RING_1 = 0b01 << 5,
        const FLAGS_DPL_RING_2 = 0b10 << 5,
        const FLAGS_DPL_RING_3 = 0b11 << 5,

        // Is system descriptor
        const FLAGS_TYPE_SYS = 0 << 4,
        const FLAGS_TYPE_SEG = 1 << 4,

        // System-Segment and Gate-Descriptor Types.
        // When the S (descriptor type) flag in a segment descriptor is clear,
        // the descriptor type is a system descriptor

        // All modes (supporting segments)
        const TYPE_SYS_LDT = 0b0_0010,

        // Protected Mode and older
        const FLAGS_TYPE_SYS_16BIT_TSS_AVAILABLE = 0b0_0001,
        const FLAGS_TYPE_SYS_16BIT_TSS_BUSY = 0b0_0011,
        const FLAGS_TYPE_SYS_16BIT_CALL_GATE = 0b0_0100,
        const FLAGS_TYPE_SYS_16BIT_TASK_GATE = 0b0_0101,
        const FLAGS_TYPE_SYS_16BIT_INTERRUPT_GATE = 0b0_0110,
        const FLAGS_TYPE_SYS_16BIT_TRAP_GATE = 0b0_0111,

        // 64-bit in IA-32e Mode (either submode), 32-bit in Protected Mode
        const FLAGS_TYPE_SYS_NATIVE_TSS_AVAILABLE = 0b0_1001,
        const FLAGS_TYPE_SYS_NATIVE_TSS_BUSY = 0b0_1011,
        const FLAGS_TYPE_SYS_NATIVE_CALL_GATE = 0b0_1100,
        const FLAGS_TYPE_SYS_NATIVE_INTERRUPT_GATE = 0b0_1110,
        const FLAGS_TYPE_SYS_NATIVE_TRAP_GATE = 0b0_1111,

        // Code- and Data-Segment Descriptor Types.
        // When the S (descriptor type) flag in a segment descriptor is set,
        // the descriptor is for either a code or a data segment.

        /// Data or code, accessed
        const FLAGS_TYPE_SEG_ACCESSED = 0b1_0001,

        const FLAGS_TYPE_DATA = 0b1_0000,
        const FLAGS_TYPE_CODE = 0b1_1000,

        // Data => permissions
        const FLAGS_TYPE_SEG_D_WRITE = 0b1_0010,
        const FLAGS_TYPE_SEG_D_EXPAND_DOWN = 0b1_0100,

        // Code => permissions
        const FLAGS_TYPE_SEG_C_READ = 0b1_0010,
        const FLAGS_TYPE_SEG_C_CONFORMING = 0b1_0100,

        /// Data Read-Only
        const FLAGS_TYPE_SEG_D_RO     = FLAGS_TYPE_DATA.bits,
        /// Data Read-Only, accessed
        const FLAGS_TYPE_SEG_D_ROA    = FLAGS_TYPE_DATA.bits
                                      | FLAGS_TYPE_SEG_ACCESSED.bits,
        /// Data Read/Write
        const FLAGS_TYPE_SEG_D_RW     = FLAGS_TYPE_DATA.bits
                                      | FLAGS_TYPE_SEG_D_WRITE.bits,
        /// Data Read/Write, accessed
        const FLAGS_TYPE_SEG_D_RWA    = FLAGS_TYPE_DATA.bits
                                      | FLAGS_TYPE_SEG_D_WRITE.bits
                                      | FLAGS_TYPE_SEG_ACCESSED.bits,
        /// Data Read-Only, expand-down
        const FLAGS_TYPE_SEG_D_ROEXD  = FLAGS_TYPE_DATA.bits
                                      | FLAGS_TYPE_SEG_D_EXPAND_DOWN.bits,
        /// Data Read-Only, expand-down, accessed
        const FLAGS_TYPE_SEG_D_ROEXDA = FLAGS_TYPE_DATA.bits
                                      | FLAGS_TYPE_SEG_D_EXPAND_DOWN.bits
                                      | FLAGS_TYPE_SEG_ACCESSED.bits,
        /// Data Read/Write, expand-down
        const FLAGS_TYPE_SEG_D_RWEXD  = FLAGS_TYPE_DATA.bits
                                      | FLAGS_TYPE_SEG_D_WRITE.bits
                                      | FLAGS_TYPE_SEG_D_EXPAND_DOWN.bits,
        /// Data Read/Write, expand-down, accessed
        const FLAGS_TYPE_SEG_D_RWEXDA = FLAGS_TYPE_DATA.bits
                                      | FLAGS_TYPE_SEG_D_WRITE.bits
                                      | FLAGS_TYPE_SEG_D_EXPAND_DOWN.bits
                                      | FLAGS_TYPE_SEG_ACCESSED.bits,

        /// Code Execute-Only
        const FLAGS_TYPE_SEG_C_EO     = FLAGS_TYPE_CODE.bits,
        /// Code Execute-Only, accessed
        const FLAGS_TYPE_SEG_C_EOA    = FLAGS_TYPE_CODE.bits
                                      | FLAGS_TYPE_SEG_ACCESSED.bits,
        /// Code Execute/Read
        const FLAGS_TYPE_SEG_C_ER     = FLAGS_TYPE_CODE.bits
                                      | FLAGS_TYPE_SEG_C_READ.bits,
        /// Code Execute/Read, accessed
        const FLAGS_TYPE_SEG_C_ERA    = FLAGS_TYPE_CODE.bits
                                      | FLAGS_TYPE_SEG_C_READ.bits
                                      | FLAGS_TYPE_SEG_ACCESSED.bits,
        /// Code Execute-Only, conforming
        const FLAGS_TYPE_SEG_C_EOC    = FLAGS_TYPE_CODE.bits
                                      | FLAGS_TYPE_SEG_C_CONFORMING.bits,
        /// Code Execute-Only, conforming, accessed
        const FLAGS_TYPE_SEG_C_EOCA   = FLAGS_TYPE_CODE.bits
                                      | FLAGS_TYPE_SEG_C_CONFORMING.bits
                                      | FLAGS_TYPE_SEG_ACCESSED.bits,
        /// Code Execute/Read, conforming
        const FLAGS_TYPE_SEG_C_ERC    = FLAGS_TYPE_CODE.bits
                                      | FLAGS_TYPE_SEG_C_READ.bits
                                      | FLAGS_TYPE_SEG_C_CONFORMING.bits,
        /// Code Execute/Read, conforming, accessed
        const FLAGS_TYPE_SEG_C_ERCA   = FLAGS_TYPE_CODE.bits
                                      | FLAGS_TYPE_SEG_C_READ.bits
                                      | FLAGS_TYPE_SEG_C_CONFORMING.bits
                                      | FLAGS_TYPE_SEG_ACCESSED.bits,
    }
}

impl Flags {
    pub const BLANK: Flags = Flags { bits: 0 };

    pub const fn from_priv(dpl: PrivilegeLevel) -> Flags {
        Flags { bits: (dpl as u8) << 5 }
    }

    pub fn from_type(ty: Type) -> Flags {
        Flags { bits: ty.pack() }
    }

    pub const fn const_or(self, other: Self) -> Flags {
        Flags { bits: self.bits | other.bits }
    }

    pub const fn cond(self, cond: bool) -> Flags {
        Flags { bits: (-(cond as i8) as u8) & self.bits }
    }

    pub const fn const_mux(self, other: Self, cond: bool) -> Flags {
        self.cond(cond).const_or(other.cond(!cond))
    }
}
