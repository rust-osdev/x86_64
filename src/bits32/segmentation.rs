#[allow(unused_imports)]
use segmentation::{SegmentSelector};
use segmentation::{DescriptorBuilder, DescriptorType, GateDescriptorBuilder, TaskGateDescriptorBuilder, SegmentDescriptorBuilder, LdtDescriptorBuilder, BuildDescriptor, SystemDescriptorTypes32, CodeSegmentType, DataSegmentType};
use ::Ring;

/// Entry for IDT, GDT or LDT. Provides size and location of a segment.
///
/// See Intel 3a, Section 3.4.5 "Segment Descriptors", and Section 3.5.2
#[derive(Copy, Clone, Debug, Default)]
#[repr(C, packed)]
pub struct Descriptor {
    pub lower: u32,
    pub upper: u32,
}

impl Descriptor {

    pub(crate) fn apply_builder_settings(&mut self, builder: &DescriptorBuilder) {
        builder.dpl.map(|ring| self.set_dpl(ring));
        builder.base_limit.map(|(base, limit)| self.set_base_limit(base as u32, limit as u32));
        builder.selector_offset.map(|(selector, offset)| self.set_selector_offset(selector, offset as u32));

        if builder.present {
            self.set_p();
        }
        if builder.avl {
            self.set_avl();
        }
        if builder.db {
            self.set_db();
        }
        if builder.limit_granularity_4k {
            self.set_g();
        }
    }

    /// Create a new segment, TSS or LDT descriptor
    /// by setting the three base and two limit fields.
    pub fn set_base_limit(&mut self, base: u32, limit: u32) {
        // Clear the base and limit fields in Descriptor
        self.lower = 0;
        self.upper = self.upper & 0x00F0FF00;

        // Set the new base
        self.lower |= base << 16;
        self.upper |= (base >> 16) & 0xff;
        self.upper |= (base >> 24) << 24;

        // Set the new limit
        self.lower |= limit & 0xffff;
        let limit_last_four_bits = (limit >> 16) & 0x0f;
        self.upper |= limit_last_four_bits << 16;
    }

    /// Creates a new descriptor with selector and offset (for IDT Gate descriptors, 
    /// e.g. Trap, Interrupts and Task gates)
    pub fn set_selector_offset(&mut self, selector: SegmentSelector, offset: u32) {
        // Clear the selector and offset
        self.lower = 0;
        self.upper = self.upper & 0x0000ffff;

        // Set selector
        self.lower |= (selector.bits() as u32) << 16;

        // Set offset
        self.lower |= offset & 0x0000ffff;
        self.upper |= offset & 0xffff0000;
    }

    /// Set the type of the descriptor (bits 8-11).
    /// Indicates the segment or gate type and specifies the kinds of access that can be made to the 
    /// segment and the direction of growth. The interpretation of this field depends on whether the descriptor 
    /// type flag specifies an application (code or data) descriptor or a system descriptor.
    pub fn set_type(&mut self, typ: u8) {
        self.upper &= !(0x0f << 8); // clear
        self.upper |= (typ as u32 & 0x0f) << 8;
    }

    /// Specifies whether the segment descriptor is for a system segment (S flag is clear) or a code or data segment (S flag is set).
    pub fn set_s(&mut self) {
        self.upper |= bit!(12);
    }

    /// Specifies the privilege level of the segment. The DPL is used to control access to the segment.
    pub fn set_dpl(&mut self, ring: Ring) {
        assert!(ring as u32 <= 0b11);
        self.upper &= !(0b11 << 13);
        self.upper |= (ring as u32) << 13;
    }

    /// Set Present bit.
    /// Indicates whether the segment is present in memory (set) or not present (clear). 
    /// If this flag is clear, the processor generates a segment-not-present exception (#NP) when a segment selector 
    /// that points to the segment descriptor is loaded into a segment register.
    pub fn set_p(&mut self) {
        self.upper |= bit!(15);
    }
    
    /// Set AVL bit. System software can use this bit to store information.
    pub fn set_avl(&mut self) {
        self.upper |= bit!(20);
    }

    /// Set L
    /// In IA-32e mode, bit 21 of the second doubleword of the segment descriptor indicates whether a 
    /// code segment contains native 64-bit code. A value of 1 indicates instructions in this code 
    /// segment are executed in 64-bit mode. A value of 0 indicates the instructions in this code segment 
    /// are executed in compatibility mode. If L-bit is set, then D-bit must be cleared.
    pub fn set_l(&mut self) {
        self.upper |= bit!(21);
    }
    
    /// Set D/B.
    /// Performs different functions depending on whether the segment descriptor is an executable code segment, 
    /// an expand-down data segment, or a stack segment.
    pub fn set_db(&mut self) {
        self.upper |= bit!(22);
    }

    /// Set G bit
    /// Determines the scaling of the segment limit field. 
    /// When the granularity flag is clear, the segment limit is interpreted in byte units; 
    /// when flag is set, the segment limit is interpreted in 4-KByte units.
    pub fn set_g(&mut self) {
        self.upper |= bit!(23);
    }
}

impl GateDescriptorBuilder<u32> for DescriptorBuilder {

    fn tss_descriptor(selector: SegmentSelector, offset: u32, available: bool) -> DescriptorBuilder {
        let typ = match available {
            true => DescriptorType::System32(SystemDescriptorTypes32::TssAvailable32),
            false => DescriptorType::System32(SystemDescriptorTypes32::TssBusy32),
        };

        DescriptorBuilder::with_selector_offset(selector, offset.into()).set_type(typ)
    }

    fn call_gate_descriptor(selector: SegmentSelector, offset: u32) -> DescriptorBuilder {
        DescriptorBuilder::with_selector_offset(selector, offset.into()).set_type(DescriptorType::System32(SystemDescriptorTypes32::CallGate32))
    }

    fn interrupt_descriptor(selector: SegmentSelector, offset: u32) -> DescriptorBuilder {
        DescriptorBuilder::with_selector_offset(selector, offset.into()).set_type(DescriptorType::System32(SystemDescriptorTypes32::InterruptGate32))
    }

    fn trap_gate_descriptor(selector: SegmentSelector, offset: u32) -> DescriptorBuilder {
        DescriptorBuilder::with_selector_offset(selector, offset.into()).set_type(DescriptorType::System32(SystemDescriptorTypes32::TrapGate32))
    }
}

impl TaskGateDescriptorBuilder for DescriptorBuilder {
    fn task_gate_descriptor(selector: SegmentSelector) -> DescriptorBuilder {
        DescriptorBuilder::with_selector_offset(selector, 0).set_type(DescriptorType::System32(SystemDescriptorTypes32::TaskGate))
    }
}

impl SegmentDescriptorBuilder<u32> for DescriptorBuilder {
    fn code_descriptor(base: u32, limit: u32, cst: CodeSegmentType) -> DescriptorBuilder {
        DescriptorBuilder::with_base_limit(base.into(), limit.into()).set_type(DescriptorType::Code(cst)).db()
    }

    fn data_descriptor(base: u32, limit: u32, dst: DataSegmentType) -> DescriptorBuilder {
        DescriptorBuilder::with_base_limit(base.into(), limit.into()).set_type(DescriptorType::Data(dst)).db()
    }
}

impl LdtDescriptorBuilder<u32> for DescriptorBuilder {
    fn ldt_descriptor(base: u32, limit: u32) -> DescriptorBuilder {
        DescriptorBuilder::with_base_limit(base.into(), limit.into()).set_type(DescriptorType::System32(SystemDescriptorTypes32::LDT))
    }
}

impl BuildDescriptor<Descriptor> for DescriptorBuilder {
    fn finish(&self) -> Descriptor {
        let mut desc: Descriptor = Default::default();
        desc.apply_builder_settings(self);

        let typ = match self.typ {
            Some(DescriptorType::System64(_)) => panic!("You shall not use 64-bit types on 32-bit descriptor."),
            Some(DescriptorType::System32(typ)) => {
                typ as u8
            },
            Some(DescriptorType::Data(typ)) => {
                desc.set_s();
                typ as u8
            },
            Some(DescriptorType::Code(typ)) => {
                desc.set_s();
                typ as u8
            },
            None => unreachable!("Type not set, this is a library bug in x86."),
        };
        desc.set_type(typ);

        desc
    }
}

/// Reload code segment register.
/// Note this is special since we can not directly move
/// to %cs. Instead we push the new segment selector
/// and return value on the stack and use lretl
/// to reload cs and continue at 1:.
#[cfg(target_arch="x86")]
pub unsafe fn set_cs(sel: SegmentSelector) {
    asm!("pushl $0; \
          pushl $$1f; \
          lretl; \
          1:" :: "ri" (sel.bits() as u32) : "memory");
}