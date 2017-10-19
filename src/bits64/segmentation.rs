use core::mem::size_of;

use bits64::task::*;
use shared::descriptor;
use shared::PrivilegeLevel;
pub use shared::segmentation::*;

/// Reload code segment register.
/// Note this is special since we can not directly move
/// to %cs. Instead we push the new segment selector
/// and return value on the stack and use lretq
/// to reload cs and continue at 1:.
pub unsafe fn set_cs(sel: SegmentSelector) {
    asm!("pushq $0; \
          leaq  1f(%rip), %rax; \
          pushq %rax; \
          lretq; \
          1:" :: "ri" (sel.bits() as usize) : "rax" "memory");
}

pub enum SegmentBitness {
    Bits32,
    Bits64,
}

impl SegmentBitness {
    pub fn pack(self) -> Flags {
        match self {
            SegmentBitness::Bits32 => FLAGS_DB,
            SegmentBitness::Bits64 => FLAGS_L,
        }
    }
}

impl SegmentDescriptor {
    pub fn new_memory(base: u32, limit: u32, ty: Type, accessed: bool, dpl: PrivilegeLevel, bitness: SegmentBitness) -> SegmentDescriptor {
        let ty1 = descriptor::Type::SegmentDescriptor {
            ty: ty,
            accessed: accessed,
        };
        let flags = bitness.pack();
        let seg = SegmentDescriptor::memory_or_tss(base, limit, ty1, dpl, flags);
        seg
    }

    pub fn new_tss(tss: &TaskStateSegment, dpl: PrivilegeLevel) -> [SegmentDescriptor; 2] {
        let tss_ptr = tss as *const TaskStateSegment;
        let ty1 = descriptor::Type::SystemDescriptor {
            size: true,
            ty: descriptor::SystemType::TssAvailable,
        };
        let seg1 = SegmentDescriptor::memory_or_tss(tss_ptr as u32, size_of::<TaskStateSegment>() as u32, ty1, dpl, Flags::empty());
        let seg2 = SegmentDescriptor::high(tss_ptr as u64);
        [seg1, seg2]
    }
}
