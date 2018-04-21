#[allow(unused_imports)]
use segmentation::SegmentSelector;

/// Reload code segment register.
/// Note this is special since we can not directly move
/// to %cs. Instead we push the new segment selector
/// and return value on the stack and use lretq
/// to reload cs and continue at 1:.
#[cfg(target_arch="x86_64")]
pub unsafe fn set_cs(sel: SegmentSelector) {
    asm!("pushq $0; \
          leaq  1f(%rip), %rax; \
          pushq %rax; \
          lretq; \
          1:" :: "ri" (sel.bits() as usize) : "rax" "memory");
}
