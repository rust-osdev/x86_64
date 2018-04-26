use segmentation::{SegmentSelector, DescriptorBuilder, GateDescriptorBuilder, DescriptorType, SystemDescriptorTypes32};

impl GateDescriptorBuilder<u16> for DescriptorBuilder {

    fn tss_descriptor(selector: SegmentSelector, offset: u16, available: bool) -> DescriptorBuilder {
        let typ = match available {
            true => DescriptorType::System32(SystemDescriptorTypes32::TSSAvailable16),
            false => DescriptorType::System32(SystemDescriptorTypes32::TssBusy16),
        };

        DescriptorBuilder::with_selector_offset(selector, offset.into()).set_type(typ)
    }

    fn call_gate_descriptor(selector: SegmentSelector, offset: u16) -> DescriptorBuilder {
        DescriptorBuilder::with_selector_offset(selector, offset.into()).set_type(DescriptorType::System32(SystemDescriptorTypes32::CallGate16))
    }

    fn interrupt_descriptor(selector: SegmentSelector, offset: u16) -> DescriptorBuilder {
        DescriptorBuilder::with_selector_offset(selector, offset.into()).set_type(DescriptorType::System32(SystemDescriptorTypes32::InterruptGate16))
    }

    fn trap_gate_descriptor(selector: SegmentSelector, offset: u16) -> DescriptorBuilder {
        DescriptorBuilder::with_selector_offset(selector, offset.into()).set_type(DescriptorType::System32(SystemDescriptorTypes32::TrapGate16))
    }
}