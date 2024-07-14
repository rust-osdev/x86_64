//! Provides functions to read and write segment registers.

pub use crate::registers::segmentation::{Segment, Segment64, CS, DS, ES, FS, GS, SS};
use crate::{
    registers::model_specific::{FsBase, GsBase, Msr},
    structures::gdt::SegmentSelector,
    VirtAddr,
};
use core::arch::asm;

macro_rules! get_reg_impl {
    ($name:literal) => {
        #[inline]
        fn get_reg() -> SegmentSelector {
            let segment: u16;
            unsafe {
                asm!(concat!("mov {0:x}, ", $name), out(reg) segment, options(nomem, nostack, preserves_flags));
            }
            SegmentSelector(segment)
        }
    };
}

macro_rules! segment_impl {
    ($type:ty, $name:literal) => {
        impl Segment for $type {
            get_reg_impl!($name);

            #[inline]
            unsafe fn set_reg(sel: SegmentSelector) {
                unsafe {
                    asm!(concat!("mov ", $name, ", {0:x}"), in(reg) sel.0, options(nostack, preserves_flags));
                }
            }
        }
    };
}

macro_rules! segment64_impl {
    ($type:ty, $name:literal, $base:ty) => {
        impl Segment64 for $type {
            const BASE: Msr = <$base>::MSR;
            #[inline]
            fn read_base() -> VirtAddr {
                unsafe {
                    let val: u64;
                    asm!(concat!("rd", $name, "base {}"), out(reg) val, options(nomem, nostack, preserves_flags));
                    VirtAddr::new_unsafe(val)
                }
            }

            #[inline]
            unsafe fn write_base(base: VirtAddr) {
                unsafe{
                    asm!(concat!("wr", $name, "base {}"), in(reg) base.as_u64(), options(nostack, preserves_flags));
                }
            }
        }
    };
}

impl Segment for CS {
    get_reg_impl!("cs");

    /// Note this is special since we cannot directly move to [`CS`]; x86 requires the instruction
    /// pointer and [`CS`] to be set at the same time. To do this, we push the new segment selector
    /// and return value onto the stack and use a "far return" (`retfq`) to reload [`CS`] and
    /// continue at the end of our function.
    ///
    /// Note we cannot use a "far call" (`lcall`) or "far jmp" (`ljmp`) to do this because then we
    /// would only be able to jump to 32-bit instruction pointers. Only Intel implements support
    /// for 64-bit far calls/jumps in long-mode, AMD does not.
    #[inline]
    unsafe fn set_reg(sel: SegmentSelector) {
        unsafe {
            asm!(
                "push {sel}",
                "lea {tmp}, [55f + rip]",
                "push {tmp}",
                "retfq",
                "55:",
                sel = in(reg) u64::from(sel.0),
                tmp = lateout(reg) _,
                options(preserves_flags),
            );
        }
    }
}

segment_impl!(SS, "ss");
segment_impl!(DS, "ds");
segment_impl!(ES, "es");
segment_impl!(FS, "fs");
segment64_impl!(FS, "fs", FsBase);
segment_impl!(GS, "gs");
segment64_impl!(GS, "gs", GsBase);

impl GS {
    /// Swap `KernelGsBase` MSR and `GsBase` MSR.
    ///
    /// ## Safety
    ///
    /// This function is unsafe because the caller must ensure that the
    /// swap operation cannot lead to undefined behavior.
    #[inline]
    pub unsafe fn swap() {
        unsafe {
            asm!("swapgs", options(nostack, preserves_flags));
        }
    }
}
