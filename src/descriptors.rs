use core::fmt;

/// Entry in GDT or LDT. Provides size and location of a segment.
bitflags! {
    flags SegmentDescriptor: u64 {
        /// Descriptor type (0 = system; 1 = code or data).
        const DESC_S    = 1 << (31+12),
        /// Descriptor privilege level 0.
        const DESC_DPL0 = 0b00 << (31+13),
        /// Descriptor privilege level 1.
        const DESC_DPL1 = 0b01 << (31+13),
        /// Descriptor privilege level 2.
        const DESC_DPL2 = 0b10 << (31+13),
        /// Descriptor privilege level 3.
        const DESC_DPL3 = 0b11 << (31+13),
        /// Descriptor is Present.
        const DESC_P = 1 << (31+15),
        /// Available for use by system software.
        const DESC_AVL  = 1 << (31+20),
        /// 64-bit code segment (IA-32e mode only).
        const DESC_L    = 1 << (31+21),
        /// Default operation size (0 = 16-bit segment, 1 = 32-bit segment)
        const DESC_DB   = 1 << (31+22),
        ///  Granularity.
        const DESC_G    = 1 << (31+23),

        // System-Segment and Gate-Descriptor Types for IA32e mode.
        // When the S (descriptor type) flag in a segment descriptor is clear,
        // the descriptor type is a system descriptor.

        const TYPE_SYS_LDT = 0b0010 << (31+8),
        const TYPE_SYS_TSS_AVAILABLE = 0b1001 << (31+8),
        const TYPE_SYS_TSS_BUSY = 0b1011 << (31+8),
        const TYPE_SYS_CALL_GATE = 0b1100 << (31+8),
        const TYPE_SYS_INTERRUPT_GATE = 0b1110 << (31+8),
        const TYPE_SYS_TRAP_GATE = 0b1111 << (31+8),

        // Code- and Data-Segment Descriptor Types.
        // When the S (descriptor type) flag in a segment descriptor is set,
        // the descriptor is for either a code or a data segment.

        /// Data Read-Only
        const TYPE_D_RO = 0b0000 << (31+8),
        /// Data Read-Only, accessed
        const TYPE_D_ROA = 0b0001 << (31+8),
        /// Data Read/Write
        const TYPE_D_RW = 0b0010 << (31+8),
        /// Data Read/Write, accessed
        const TYPE_D_RWA = 0b0011 << (31+8),
        /// Data Read-Only, expand-down
        const TYPE_D_ROEXD = 0b0100 << (31+8),
        /// Data Read-Only, expand-down, accessed
        const TYPE_D_ROEXDA = 0b0101 << (31+8),
        /// Data Read/Write, expand-down
        const TYPE_D_RWEXD = 0b0110 << (31+8),
        /// Data Read/Write, expand-down, accessed
        const TYPE_D_RWEXDA = 0b0111 << (31+8),

        /// Code Execute-Only
        const TYPE_C_EO = 0b1000 << (31+8),
        /// Code Execute-Only, accessed
        const TYPE_C_EOA = 0b1001 << (31+8),
        /// Code Execute/Read
        const TYPE_C_ER = 0b1010 << (31+8),
        /// Code Execute/Read, accessed
        const TYPE_C_ERA = 0b1011 << (31+8),
        /// Code Execute-Only, conforming
        const TYPE_C_EOC = 0b1100 << (31+8),
        /// Code Execute-Only, conforming, accessed
        const TYPE_C_EOCA = 0b1101 << (31+8),
        /// Code Execute/Read, conforming
        const TYPE_C_ERC = 0b1110 << (31+8),
        /// Code Execute/Read, conforming, accessed
        const TYPE_C_ERCA = 0b1111 << (31+8),
    }
}

/// This is data-structure is a ugly mess thing so we provide some
/// convenience function to program it.
impl SegmentDescriptor {
    pub fn new(base: u32, limit: u32) -> SegmentDescriptor {
        let base_low: u64 = base as u64 & 0xffffff;
        let base_high: u64 = base as u64 & 0xff000000 >> 24;
        let limit_low: u64 = limit as u64 & 0xffff;
        let limit_high: u64 = (limit as u64 & (0b1111 << 16)) >> 16;

        SegmentDescriptor{
            bits: limit_low | base_low << 16 | limit_high << (31+16) | base_high << (31+24)
        }
    }
}

impl fmt::Debug for SegmentDescriptor {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "SD: 0x{:x}", self.bits)
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

/// A struct describing a pointer to a descriptor table (GDT / IDT).
/// This is in a format suitable for giving to 'lgdt' or 'lidt'.
#[derive(Debug)]
#[repr(C, packed)]
pub struct DescriptorTablePointer {
   /// Size of the DT.
   pub limit: u16,
   /// Pointer to the memory region containing the DT.
   pub base: u64
}

/// Load GDT table.
pub unsafe fn lgdt(idt: &DescriptorTablePointer) {
    asm!("lgdt ($0)" :: "r" (idt));
}