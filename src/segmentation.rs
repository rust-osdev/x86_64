use core::fmt;

use Ring;

/// Specifies which element to load into a segment from
/// descriptor tables (i.e., is a index to LDT or GDT table
/// with some additional flags).
///
/// See Intel 3a, Section 3.4.2 "Segment Selectors"
bitflags! {
    pub struct SegmentSelector: u16 {
        /// Requestor Privilege Level
        const RPL_0 = 0b00;
        const RPL_1 = 0b01;
        const RPL_2 = 0b10;
        const RPL_3 = 0b11;

        /// Table Indicator (TI) 0 means GDT is used.
        const TI_GDT = 0 << 2;
        /// Table Indicator (TI) 1 means LDT is used.
        const TI_LDT = 1 << 2;
    }
}

impl SegmentSelector {
    /// Create a new SegmentSelector
    ///
    /// # Arguments
    ///  * `index` - index in GDT or LDT array.
    ///  * `rpl` - Requested privilege level of the selector  
    pub const fn new(index: u16, rpl: Ring) -> SegmentSelector {
        SegmentSelector {
            bits: index << 3 | (rpl as u16),
        }
    }

    /// Make a new segment selector from a untyped u16 value.
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

        write!(
            f,
            "Index {} in {}, {}{}{}{}",
            self.bits >> 3,
            tbl,
            r0,
            r1,
            r2,
            r3
        )
    }
}

/// System-Segment and Gate-Descriptor Types 64-bit mode
/// See also Intel 3a, Table 3-2 System Segment and Gate-Descriptor Types.
#[repr(u8)]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum SystemDescriptorTypes64 {
    //Reserved0 = 0b0000,
    //Reserved1 = 0b0001,
    LDT = 0b0010,
    //Reserved = 0b0011,
    //Reserved = 0b0100,
    //Reserved = 0b0101,
    //Reserved = 0b0110,
    //Reserved = 0b0111,
    //Reserved = 0b1000,
    TssAvailable = 0b1001,
    //Reserved = 0b1010,
    TssBusy = 0b1011,
    CallGate = 0b1100,
    //Reserved = 0b1101,
    InterruptGate = 0b1110,
    TrapGate = 0b1111,
}

/// System-Segment and Gate-Descriptor Types 32-bit mode.
/// See also Intel 3a, Table 3-2 System Segment and Gate-Descriptor Types.
#[repr(u8)]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum SystemDescriptorTypes32 {
    //Reserved0 = 0b0000,
    TSSAvailable16 = 0b0001,
    LDT = 0b0010,
    TSSBusy16 = 0b0011,
    CallGate16 = 0b0100,
    TaskGate = 0b0101,
    InterruptGate16 = 0b0110,
    TrapGate16 = 0b0111,
    //Reserved1 = 0b1000,
    TssAvailable32 = 0b1001,
    //Reserved2 = 0b1010,
    TssBusy32 = 0b1011,
    CallGate32 = 0b1100,
    //Reserved3 = 0b1101,
    InterruptGate32 = 0b1110,
    TrapGate32 = 0b1111,
}

/// Data Segment types for descriptors.
/// See also Intel 3a, Table 3-1 Code- and Data-Segment Types.
#[repr(u8)]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum DataSegmentType {
    /// Data Read-Only
    ReadOnly = 0b0000,
    /// Data Read-Only, accessed
    ReadOnlyAccessed = 0b0001,
    /// Data Read/Write
    ReadWrite = 0b0010,
    /// Data Read/Write, accessed
    ReadWriteAccessed = 0b0011,
    /// Data Read-Only, expand-down
    ReadExpand = 0b0100,
    /// Data Read-Only, expand-down, accessed
    ReadExpandAccessed = 0b0101,
    /// Data Read/Write, expand-down
    ReadWriteExpand = 0b0110,
    /// Data Read/Write, expand-down, accessed
    ReadWriteExpandAccessed = 0b0111,
}

/// Code Segment types for descriptors.
/// See also Intel 3a, Table 3-1 Code- and Data-Segment Types.
#[repr(u8)]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum CodeSegmentType {
    /// Code Execute-Only
    Execute = 0b1000,
    /// Code Execute-Only, accessed
    ExecuteAccessed = 0b1001,
    /// Code Execute/Read
    ExecuteRead = 0b1010,
    /// Code Execute/Read, accessed
    ExecuteReadAccessed = 0b1011,
    /// Code Execute-Only, conforming
    ExecuteConforming = 0b1100,
    /// Code Execute-Only, conforming, accessed
    ExecuteConformingAccessed = 0b1101,
    /// Code Execute/Read, conforming
    ExecuteReadConforming = 0b1110,
    /// Code Execute/Read, conforming, accessed
    ExecuteReadConformingAccessed = 0b1111,
}

/// Helper enum type to differentiate between the different descriptor types that all end up written in the same field.
#[derive(Debug, Eq, PartialEq)]
pub(crate) enum DescriptorType {
    System64(SystemDescriptorTypes64),
    System32(SystemDescriptorTypes32),
    Data(DataSegmentType),
    Code(CodeSegmentType)
}

/// Trait that defines the architecture specific functions for building various system segment descriptors
/// which are available on all 16, 32, and 64 bits.
pub(crate) trait GateDescriptorBuilder<Size> {
    fn tss_descriptor(selector: SegmentSelector, offset: Size, available: bool) -> Self;
    fn call_gate_descriptor(selector: SegmentSelector, offset: Size) -> Self;
    fn interrupt_descriptor(selector: SegmentSelector, offset: Size) -> Self;
    fn trap_gate_descriptor(selector: SegmentSelector, offset: Size) -> Self;
}

/// Trait to implement for building a task-gate (this descriptor is not implemented for 64-bit systems since 
/// Hardware task switches are not supported in IA-32e mode.).
pub(crate) trait TaskGateDescriptorBuilder {
    fn task_gate_descriptor(selector: SegmentSelector) -> Self;
}

/// Trait to define functions that build architecture specific code and data descriptors.
pub(crate) trait SegmentDescriptorBuilder<Size> {
    fn code_descriptor(base: Size, limit: Size, cst: CodeSegmentType) -> Self;
    fn data_descriptor(base: Size, limit: Size, dst: DataSegmentType) -> Self;
}

/// Trait to define functions that build an architecture specific ldt descriptor.
/// There is no corresponding ldt descriptor type for 16 bit.
pub(crate) trait LdtDescriptorBuilder<Size> {
    fn ldt_descriptor(base: Size, limit: Size) -> Self;
}

pub(crate) trait BuildDescriptor<Descriptor> {
    fn finish(&self) -> Descriptor;
}

/// Makes building descriptors easier (hopefully).
#[derive(Debug)]
pub struct DescriptorBuilder {
    /// The base defines the location of byte 0 of the segment within the 4-GByte linear address space.
    /// The limit is the size of the range covered by the segment. Really a 20bit value.
    pub(crate) base_limit: Option<(u64, u64)>,
    /// Alternative to base_limit we use a selector that points to a segment and an an offset for certain descriptors.
    pub(crate) selector_offset: Option<(SegmentSelector, u64)>,
    /// Descriptor type
    pub(crate) typ: Option<DescriptorType>,
    /// Specifies the privilege level of the segment. The privilege level can range from 0 to 3, with 0 being the most privileged level.
    pub(crate) dpl: Option<Ring>,
    /// Indicates whether the segment is present in memory (set) or not present (clear).
    pub(crate) present: bool,
    /// Available for use by system software
    pub(crate) avl: bool,
    /// Default operation size
    pub(crate) db: bool,
    /// Determines the scaling of the segment limit field. When the granularity flag is clear, the segment limit is interpreted in byte units; when flag is set, the segment limit is interpreted in 4-KByte units.
    pub(crate) limit_granularity_4k: bool,
}

impl DescriptorBuilder {

    /// Start building a new descriptor with a base and limit.
    pub(crate) fn with_base_limit(base: u64, limit: u64) -> DescriptorBuilder {
        DescriptorBuilder { base_limit: Some((base, limit)), selector_offset: None, typ: None, dpl: None, present: false, avl: false, db: false, limit_granularity_4k: false }
    }

    /// Start building a new descriptor with a segment selector and offset.
    pub(crate) fn with_selector_offset(selector: SegmentSelector, offset: u64) -> DescriptorBuilder {
        DescriptorBuilder { base_limit: None, selector_offset: Some((selector, offset)), typ: None, dpl: None, present: false, avl: false, db: false, limit_granularity_4k: false }
    }
    
    pub(crate) fn set_type(mut self, typ: DescriptorType) -> DescriptorBuilder {
        self.typ = Some(typ);
        self
    }

    /// The segment limit is interpreted in 4-KByte units if this is set.
    pub fn limit_granularity_4kb(mut self) -> DescriptorBuilder {
        self.limit_granularity_4k = true;
        self
    }

    /// Indicates whether the segment is present in memory (set) or not present (clear).
    pub fn present(mut self) -> DescriptorBuilder {
        self.present = true;
        self
    }

    /// Specifies the privilege level of the segment.
    pub fn dpl(mut self, dpl: Ring) -> DescriptorBuilder {
        self.dpl = Some(dpl);
        self
    }

    /// Toggle the AVL bit.
    pub fn avl(mut self) -> DescriptorBuilder {
        self.avl = true;
        self
    }

    /// Set default operation size (false for 16bit segment, true for 32bit segments).
    pub fn db(mut self) -> DescriptorBuilder {
        self.db = true;
        self
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
