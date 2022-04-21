use lazy_static::lazy_static;
use x86_64::structures::gdt::{Descriptor, GlobalDescriptorTable, SegmentSelector};
use x86_64::structures::tss::TaskStateSegment;
use x86_64::VirtAddr;

pub const DOUBLE_FAULT_IST_INDEX: u8 = 0;

lazy_static! {
    static ref TSS: TaskStateSegment = {
        let mut tss = TaskStateSegment::new();
        tss.interrupt_stack_table[DOUBLE_FAULT_IST_INDEX as usize] = {
            const STACK_SIZE: usize = 4096;
            static mut STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];

            let stack_start = VirtAddr::from_ptr(unsafe { &STACK });
            let stack_end = stack_start + STACK_SIZE as u64;
            stack_end
        };
        tss
    };
    static ref GDT: (GlobalDescriptorTable, Selectors) = {
        let mut gdt = GlobalDescriptorTable::new();
        // Add an unused segment so we get a different value for CS
        gdt.append(Descriptor::kernel_data_segment());
        let code_selector = gdt.append(Descriptor::kernel_code_segment());
        let tss_selector = gdt.append(Descriptor::tss_segment(&TSS));
        (
            gdt,
            Selectors {
                code_selector,
                tss_selector,
            },
        )
    };
}

struct Selectors {
    code_selector: SegmentSelector,
    tss_selector: SegmentSelector,
}

pub fn init() {
    use x86_64::instructions::segmentation::{Segment, CS};
    use x86_64::instructions::tables::load_tss;

    // Make sure loading CS actually changes the value
    GDT.0.load();
    assert_ne!(CS::get_reg(), GDT.1.code_selector);
    unsafe { CS::set_reg(GDT.1.code_selector) };
    assert_eq!(CS::get_reg(), GDT.1.code_selector);

    // Loading the TSS should mark the GDT entry as busy
    let tss_idx: usize = GDT.1.tss_selector.index().into();
    let old_tss_entry = GDT.0.entries()[tss_idx].clone();
    unsafe { load_tss(GDT.1.tss_selector) };
    let new_tss_entry = GDT.0.entries()[tss_idx].clone();
    assert_ne!(old_tss_entry, new_tss_entry);
}
