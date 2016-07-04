//! Helpers to program the task state segment.
//! See Intel 3a, Chapter 7

pub use shared::segmentation;

/// Load the task state register.
pub unsafe fn load_tr(sel: segmentation::SegmentSelector) {
    asm!("ltr $0" :: "r" (sel));
}
