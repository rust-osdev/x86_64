/// Entry in GDT or LDT. Provides size and location of a segment.
bitflags! {
    flags SegmentDescriptor: u64 {
        /// Descriptor type (0 = system; 1 = code or data).
        const SEGMENT_DESCRIPTOR_S    = 1 << (31+12),
        /// Descriptor privilege level 0.
        const SEGMENT_DESCRIPTOR_DPL0 = 0b00 << (31+13),
        /// Descriptor privilege level 1.
        const SEGMENT_DESCRIPTOR_DPL1 = 0b01 << (31+13),
        /// Descriptor privilege level 2.
        const SEGMENT_DESCRIPTOR_DPL2 = 0b10 << (31+13),
        /// Descriptor privilege level 3.
        const SEGMENT_DESCRIPTOR_DPL3 = 0b11 << (31+13),
        /// Available for use by system software.
        const SEGMENT_DESCRIPTOR_AVL  = 1 << (31+20),
        /// 64-bit code segment (IA-32e mode only).
        const SEGMENT_DESCRIPTOR_L    = 1 << (31+21),
        /// Default operation size (0 = 16-bit segment, 1 = 32-bit segment)
        const SEGMENT_DESCRIPTOR_DB   = 1 << (31+22),
        ///  Granularity.
        const SEGMENT_DESCRIPTOR_G    = 1 << (31+23),
    }
}

/// System-Segment and Gate-Descriptor Types for IA32e mode.
/// When the S (descriptor type) flag in a segment descriptor is clear,
/// the descriptor type is a system descriptor.
pub enum SystemDescriptorType {
    Ldt = 0b0010,
    TSSAvailable = 0b1001,
    TSSBudy = 0b1011,
    CallGate = 0b1100,
    InterruptGate = 0b1110,
    TrapGate = 0b1111,
}

/// Code- and Data-Segment Descriptor Types.
/// When the S (descriptor type) flag in a segment descriptor is set,
/// the descriptor is for either a code or a data segment.
pub enum CodeDataDescriptorType {
     /// Data Read-Only
     DataRO = 0b0000,
     /// Data Read-Only, accessed
     DataROA = 0b0001,
     /// Data Read/Write
     DataRW = 0b0010,
     /// Data Read/Write, accessed
     DataRWA = 0b0011,
     /// Data Read-Only, expand-down
     DataROEXD = 0b0100,
     /// Data Read-Only, expand-down, accessed
     DataROEXDA = 0b0101,
     /// Data Read/Write, expand-down
     DataRWEXD = 0b0110,
     /// Data Read/Write, expand-down, accessed
     DataRWEXDA = 0b0111,

     /// Code Execute-Only
     CodeEO = 0b1000,
     /// Code Execute-Only, accessed
     CodeEOA = 0b1001,
     /// Code Execute/Read
     CodeER = 0b1010,
     /// Code Execute/Read, accessed
     CodeERA = 0b1011,
     /// Code Execute-Only, conforming
     CodeEOC = 0b1100,
     /// Code Execute-Only, conforming, accessed
     CodeEOCA = 0b1101,
     /// Code Execute/Read, conforming
     CodeERC = 0b1110,
     /// Code Execute/Read, conforming, accessed
     CodeERCA = 0b1111
}

/// This is data-structure is a ugly mess thing so we provide some
/// convenience function to program it.
impl SegmentDescriptor {
    pub fn new(base: u32, limit: u32) -> SegmentDescriptor {
        let base_low: u64 = base as u64 & 0xffffff;
        let base_high: u64 = base as u64 & 0xff000000 >> 24;
        let limit_low: u64 = limit as u64 & 0xffff;
        let limit_high: u64 = (limit as u64 & (0b1111 << 16)) >> 16;

        let bits: u64 = limit_low | base_low << 16 | limit_high << (31+16) | base_high << (31+24);
        SegmentDescriptor{ bits: bits }
    }
}

/// In 64-bit mode the TSS holds information that is not
/// directly related to the task-switch mechanism.
#[repr(C, packed)]
pub struct TaskStateSegement {
    reserved: u32,
    /// The full 64-bit canonical forms of the stack pointers (RSP) for privilege levels 0-2.
    rsp: [u64; 3],
    reserved2: u64,
    /// The full 64-bit canonical forms of the interrupt stack table (IST) pointers.
    ist: [u64; 7],
    reserved3: u64,
    reserved4: u16,
    /// The 16-bit offset to the I/O permission bit map from the 64-bit TSS base.
    iomap_base: u16,
}
