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
    type_access: u8,
    limit2_flags: u8,
    base3: u8,
}

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

/// System-Segment and Gate-Descriptor Types 32-bit mode
#[repr(u8)]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
enum SystemDescriptor32 {
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

/// System-Segment and Gate-Descriptor Types 64-bit mode
#[repr(u8)]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
enum SystemDescriptor64 {
    //Reserved0 = 0b0000,
    //Reserved1 = 0b0001,
    LDT = 0b0010,
    //Reserved = 0b0011,
    //Reserved = 0b0100,
    //Reserved = 0b0101,
    //Reserved = 0b0110,
    //Reserved = 0b0111,
    //Reserved = 0b1000,
    TssAvailable64 = 0b1001,
    //Reserved = 0b1010,
    TssBusy64 = 0b1011,
    CallGate64 = 0b1100,
    //Reserved = 0b1101,
    InterruptGate64 = 0b1110,
    TrapGate64 = 0b1111,
}

#[derive(Debug, Eq, PartialEq)]
pub enum SystemMode {
    Mode16,
    Mode32,
    Mode64,
}

/// Makes building descriptors easier (hopefully).
pub struct DescriptorBuilder {
    /// What privilege level the descriptor is for.
    mode: SystemMode,

    /// Defines the location of byte 0 of the segment within the 4-GByte linear address space.
    base: u32,
    /// The size of the range covered by the segment. Really a 20bit value.
    limit: u32,

    /// The type of the segment if we have a data segment.
    dst: Option<DataSegmentType>,
    /// The type of the segment if we have a code segment.
    cst: Option<CodeSegmentType>,
    /// The type of the segment if we have a system segment in 32bit mode.
    system_type32: Option<SystemDescriptor32>,
    /// The type of the segment if we have a system segment in 64bit mode.
    system_type64: Option<SystemDescriptor64>,

    /// Specifies the privilege level of the segment. The privilege level can range from 0 to 3, with 0 being the most privileged level.
    dpl: Option<Ring>,
    /// Indicates whether the segment is present in memory (set) or not present (clear).
    present: bool,
    /// Available for use by system software
    avl: bool,
    /// Performs different functions depending on whether the segment descriptor is an executable code segment, an expand-down data segment, or a stack segment.
    db: bool,
    /// Determines the scaling of the segment limit field. When the granularity flag is clear, the segment limit is interpreted in byte units; when flag is set, the segment limit is interpreted in 4-KByte units.
    limit_granularity_4k: bool,
}

impl DescriptorBuilder {
    pub fn new(mode: SystemMode) -> DescriptorBuilder {
        DescriptorBuilder {
            base: 0,
            limit: 0,
            mode: mode,
            dst: None,
            cst: None,
            system_type32: None,
            system_type64: None,
            dpl: None,
            present: false,
            db: false,
            avl: false,
            limit_granularity_4k: false,
        }
    }

    /// Set a base for the descriptor.
    pub fn base(mut self, base: u32) -> DescriptorBuilder {
        self.base = base;
        self
    }

    /// Set the limit for the descriptor.
    pub fn limit(mut self, limit: u32) -> DescriptorBuilder {
        self.limit = limit;
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

    /// Make a ldt descriptor.
    pub fn ldt_descriptor(mut self) -> DescriptorBuilder {
        match self.mode {
            SystemMode::Mode16 => self.system_type32 = Some(SystemDescriptor32::LDT),
            SystemMode::Mode32 => self.system_type32 = Some(SystemDescriptor32::LDT),
            SystemMode::Mode64 => self.system_type64 = Some(SystemDescriptor64::LDT),
        }
        self
    }

    /// Make a tss descriptor.
    pub fn tss_descriptor(mut self, available: bool) -> DescriptorBuilder {
        match (available, &self.mode)  {
            (true, SystemMode::Mode16) => self.system_type32 = Some(SystemDescriptor32::TSSAvailable16),
            (true, SystemMode::Mode32) => self.system_type32 = Some(SystemDescriptor32::TssAvailable32),
            (true, SystemMode::Mode64) => self.system_type64 = Some(SystemDescriptor64::TssAvailable64),
            (false, SystemMode::Mode16) => self.system_type32 = Some(SystemDescriptor32::TSSBusy16),
            (false, SystemMode::Mode32) => self.system_type32 = Some(SystemDescriptor32::TssBusy32),
            (false, SystemMode::Mode64) => self.system_type64 = Some(SystemDescriptor64::TssBusy64),
        }
        self
    }

    /// Make a call gate descriptor.
    pub fn call_gate_descriptor(mut self) -> DescriptorBuilder {
        match self.mode {
            SystemMode::Mode16 => self.system_type32 = Some(SystemDescriptor32::CallGate16),
            SystemMode::Mode32 => self.system_type32 = Some(SystemDescriptor32::CallGate32),
            SystemMode::Mode64 => self.system_type64 = Some(SystemDescriptor64::CallGate64),
        }
        self
    }

    /// Make an interrupt descriptor.
    pub fn interrupt_descriptor(mut self) -> DescriptorBuilder {
        match self.mode  {
            SystemMode::Mode16 => self.system_type32 = Some(SystemDescriptor32::InterruptGate16),
            SystemMode::Mode32 => self.system_type32 = Some(SystemDescriptor32::InterruptGate32),
            SystemMode::Mode64 => self.system_type64 = Some(SystemDescriptor64::InterruptGate64),
        }
        self
    }

    /// Make a trap gate descriptor
    pub fn trap_gate_descriptor(mut self) -> DescriptorBuilder {
        match self.mode  {
            SystemMode::Mode16 => self.system_type32 = Some(SystemDescriptor32::TrapGate16),
            SystemMode::Mode32 => self.system_type32 = Some(SystemDescriptor32::TrapGate32),
            SystemMode::Mode64 => self.system_type64 = Some(SystemDescriptor64::TrapGate64),
        }
        self
    }

    /// Make a task gate descriptor. Note: This call will panic if mode is not 32bit!
    pub fn task_gate_descriptor(mut self) -> DescriptorBuilder {
        match self.mode {
            SystemMode::Mode32 => self.system_type32 = Some(SystemDescriptor32::TaskGate),
            _ => panic!("Can't build a taskgate for {:?}", self.mode)
        }
        self
    }

    // Make a code segment descriptor.
    pub fn new_code_descriptor(mut self, cst: CodeSegmentType) -> DescriptorBuilder {
        self.cst = Some(cst);
        if self.mode == SystemMode::Mode32 {
            // Not sure it's always ok to do this here but the manual says:
            // This flag should always be set to 1 for 32-bit code and data segments and to 0 for 16-bit code and data segments.
            self.db = true;
        }
        self
    }

    // Make a data segment descriptor.
    pub fn new_data_descriptor(mut self, dst: DataSegmentType) -> DescriptorBuilder {
        self.dst = Some(dst);
        if self.mode == SystemMode::Mode32 {
            // Not sure it's always ok to do this here but the manual says:
            // This flag should always be set to 1 for 32-bit code and data segments and to 0 for 16-bit code and data segments.
            self.db = true;
        }
        self
    }

    // Build the final segment descriptor.
    pub fn finish(&self) -> SegmentDescriptor {
        let mut sd = SegmentDescriptor {
            limit1: 0,
            base1: 0,
            base2: 0,
            type_access: 0,
            limit2_flags: 0,
            base3: 0,
        };

        // Set base
        sd.base1 = self.base as u16;
        sd.base2 = (self.base >> 16) as u8;
        sd.base3 = (self.base >> 24) as u8;

        // Set limit
        sd.limit1 = self.limit as u16;
        sd.limit2_flags = (sd.limit2_flags & 0xf0) | (((self.limit >> 16) as u8) & 0x0f);

        // Set Type and S
        // s_bit specifies whether the segment descriptor is for a system segment (S flag is clear) or a code or data segment (S flag is set).
        let s_bit = 1 << 4;
        match (self.dst, self.cst, self.system_type32, self.system_type64) {
            (Some(typ), None, None, None) => sd.type_access = (sd.type_access & 0xf0) | s_bit | (typ as u8 & 0x0f),
            (None, Some(typ), None, None) => sd.type_access = (sd.type_access & 0xf0) | s_bit | (typ as u8  & 0x0f),
            (None, None, Some(typ), None) => sd.type_access = (sd.type_access & 0xf0) | (typ as u8 & 0x0f),
            (None, None, None, Some(typ)) => sd.type_access = (sd.type_access & 0xf0) | (typ as u8 & 0x0f),
            (None, None, None, None) => {/* do nothing */},
            _ => panic!("Trying to build a segment descriptor that is multiple types is not possible."),
        }

        // Set DPL
        self.dpl.map(|ring| {
            sd.type_access |= (ring as u8) << 5;
        });
        // Set P
        sd.type_access |= (self.present as u8) << 7; 
        // Set AVL
        sd.limit2_flags |= (self.avl as u8) << 4;
        // Set L
        sd.limit2_flags |= ((self.mode == SystemMode::Mode64) as u8) << 5;
        // Set D/B
        sd.limit2_flags |= (self.db as u8) << 6;
        // Set G
        sd.limit2_flags |= (self.limit_granularity_4k as u8) << 7;
        
        sd
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
