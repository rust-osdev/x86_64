//! Fields which are common to all segment-section and gate descriptors

use shared::PrivilegeLevel;

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
pub enum Type {
    SystemDescriptor {
        /// false/0: 16-bit
        /// true/1: native (32- or 64-bit)
        size: bool,
        ty: SystemType
    },
}

impl Type {
    pub fn pack(self) -> u8 {
        match self {
            Type::SystemDescriptor { size, ty } =>
                (size as u8) << 3 | (ty as u8) | FLAGS_TYPE_SYS.bits,
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
