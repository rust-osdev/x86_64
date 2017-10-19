use core::mem::size_of;

use bits32::task::*;
use shared::descriptor;
use shared::PrivilegeLevel;
pub use shared::segmentation::*;

/// Reload code segment register.
/// Note this is special since we can not directly move
/// to %cs. Instead we push the new segment selector
/// and return value on the stack and use lretl
/// to reload cs and continue at 1:.
pub unsafe fn set_cs(sel: SegmentSelector) {
    asm!("pushl $0; \
          pushl $$1f; \
          lretl; \
          1:" :: "ri" (sel.bits() as usize) : "memory");
}

impl SegmentDescriptor {
    pub fn new_memory(base: u32, limit: u32, ty: Type, accessed: bool, dpl: PrivilegeLevel) -> SegmentDescriptor {
        let ty1 = descriptor::Type::SegmentDescriptor {
            ty: ty,
            accessed: accessed,
        };
        let flags = FLAGS_DB;
        let seg = SegmentDescriptor::memory_or_tss(base, limit, ty1, dpl, flags);
        seg
    }

    pub fn new_tss(tss: &TaskStateSegment, dpl: PrivilegeLevel) -> [SegmentDescriptor; 2] {
        let tss_ptr = tss as *const TaskStateSegment;
        let ty1 = descriptor::Type::SystemDescriptor {
            size: true,
            ty: descriptor::SystemType::TssAvailable,
        };
        let seg = SegmentDescriptor::memory_or_tss(tss_ptr as u32, size_of::<TaskStateSegment>() as u32, ty1, dpl, Flags::empty());
        seg
    }
}
