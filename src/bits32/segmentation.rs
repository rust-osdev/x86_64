#[allow(unused_imports)]
use segmentation::{SegmentSelector};

/// Reload code segment register.
/// Note this is special since we can not directly move
/// to %cs. Instead we push the new segment selector
/// and return value on the stack and use lretl
/// to reload cs and continue at 1:.
#[cfg(target_arch="x86")]
pub unsafe fn load_cs(sel: SegmentSelector) {
    asm!("pushl $0; \
          pushl $$1f; \
          lretl; \
          1:" :: "ri" (sel.bits() as u32) : "memory");
}