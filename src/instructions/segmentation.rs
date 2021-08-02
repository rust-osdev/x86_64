//! Provides functions to read and write segment registers.

#[cfg(docsrs)]
use crate::{
    registers::control::Cr4Flags,
    structures::gdt::{Descriptor, GlobalDescriptorTable},
};
use crate::{
    registers::model_specific::{FsBase, GsBase, Msr},
    structures::gdt::SegmentSelector,
    VirtAddr,
};

/// An x86 segment
///
/// Segment registers on x86 are 16-bit [`SegmentSelector`]s, which index into
/// the [`GlobalDescriptorTable`]. The corresponding GDT entry is used to
/// configure the segment itself. Note that most segmentation functionality is
/// disabled in 64-bit mode. See the individual segments for more information.
pub trait Segment {
    /// Returns the current value of the segment register.
    fn get_reg() -> SegmentSelector;
    /// Reload the segment register. Depending on the segment, this may also
    /// reconfigure the corresponding segment.
    ///
    /// ## Safety
    ///
    /// This function is unsafe because the caller must ensure that `sel`
    /// is a valid segment descriptor, and that reconfiguring the segment will
    /// not cause undefined behavior.
    unsafe fn set_reg(sel: SegmentSelector);
}

/// An x86 segment which is actually used in 64-bit mode
///
/// While most segments are unused in 64-bit mode, the FS and GS segments are
/// still partially used. Only the 64-bit segment base address is used, and this
/// address can be set via the GDT, or by using the `FSGSBASE` instructions.
pub trait Segment64: Segment {
    /// MSR containing the segment base. This MSR can be used to set the base
    /// when [`CR4.FSGSBASE`][Cr4Flags::FSGSBASE] is **not** set.
    const BASE: Msr;
    /// Reads the segment base address
    ///
    /// ## Exceptions
    ///
    /// If [`CR4.FSGSBASE`][Cr4Flags::FSGSBASE] is not set, this instruction will throw a `#UD`.
    fn read_base() -> VirtAddr;
    /// Writes the segment base address
    ///
    /// ## Exceptions
    ///
    /// If [`CR4.FSGSBASE`][Cr4Flags::FSGSBASE] is not set, this instruction will throw a `#UD`.
    ///
    /// ## Safety
    ///
    /// The caller must ensure that this write operation has no unsafe side
    /// effects, as the segment base address might be in use.
    unsafe fn write_base(base: VirtAddr);
}

macro_rules! get_reg_impl {
    ($name:literal, $asm_get:ident) => {
        fn get_reg() -> SegmentSelector {
            let segment: u16;
            #[cfg(feature = "inline_asm")]
            unsafe {
                asm!(concat!("mov {0:x}, ", $name), out(reg) segment, options(nomem, nostack, preserves_flags));
            }
            #[cfg(not(feature = "inline_asm"))]
            unsafe {
                segment = crate::asm::$asm_get();
            }
            SegmentSelector(segment)
        }
    };
}

macro_rules! segment_impl {
    ($type:ty, $name:literal, $asm_get:ident, $asm_load:ident) => {
        impl Segment for $type {
            get_reg_impl!($name, $asm_get);

            unsafe fn set_reg(sel: SegmentSelector) {
                #[cfg(feature = "inline_asm")]
                asm!(concat!("mov ", $name, ", {0:x}"), in(reg) sel.0, options(nostack, preserves_flags));

                #[cfg(not(feature = "inline_asm"))]
                crate::asm::$asm_load(sel.0);
            }
        }
    };
}

macro_rules! segment64_impl {
    ($type:ty, $name:literal, $base:ty, $asm_rd:ident, $asm_wr:ident) => {
        impl Segment64 for $type {
            const BASE: Msr = <$base>::MSR;
            fn read_base() -> VirtAddr {
                #[cfg(feature = "inline_asm")]
                unsafe {
                    let val: u64;
                    asm!(concat!("rd", $name, "base {}"), out(reg) val, options(nomem, nostack, preserves_flags));
                    VirtAddr::new_unsafe(val)
                }
                #[cfg(not(feature = "inline_asm"))]
                unsafe {
                    VirtAddr::new_unsafe(crate::asm::$asm_rd())
                }
            }

            unsafe fn write_base(base: VirtAddr) {
                #[cfg(feature = "inline_asm")]
                asm!(concat!("wr", $name, "base {}"), in(reg) base.as_u64(), options(nostack, preserves_flags));

                #[cfg(not(feature = "inline_asm"))]
                crate::asm::$asm_wr(base.as_u64());
            }
        }
    };
}

/// Code Segment
///
/// The segment base and limit are unused in 64-bit mode. Only the L (long), D
/// (default operation size), and DPL (descriptor privilege-level) fields of the
/// descriptor are recognized. So changing the segment register can be used to
/// change privilege level or enable/disable long mode.
#[derive(Debug)]
pub struct CS;
impl Segment for CS {
    get_reg_impl!("cs", x86_64_asm_get_cs);

    /// Note this is special since we cannot directly move to [`CS`]; x86 requires the instruction
    /// pointer and [`CS`] to be set at the same time. To do this, we push the new segment selector
    /// and return value onto the stack and use a "far return" (`retfq`) to reload [`CS`] and
    /// continue at the end of our function.
    ///
    /// Note we cannot use a "far call" (`lcall`) or "far jmp" (`ljmp`) to do this because then we
    /// would only be able to jump to 32-bit instruction pointers. Only Intel implements support
    /// for 64-bit far calls/jumps in long-mode, AMD does not.
    unsafe fn set_reg(sel: SegmentSelector) {
        #[cfg(feature = "inline_asm")]
        asm!(
            "push {sel}",
            "lea {tmp}, [1f + rip]",
            "push {tmp}",
            "retfq",
            "1:",
            sel = in(reg) u64::from(sel.0),
            tmp = lateout(reg) _,
            options(preserves_flags),
        );

        #[cfg(not(feature = "inline_asm"))]
        crate::asm::x86_64_asm_set_cs(u64::from(sel.0));
    }
}

/// Stack Segment
///
/// Entirely unused in 64-bit mode; setting the segment register does nothing.
/// However, in ring 3, the SS register still has to point to a valid
/// [`Descriptor`] (it cannot be zero). This means a user-mode read/write
/// segment descriptor must be present in the GDT.
///
/// This register is also set by the `syscall`/`sysret` and
/// `sysenter`/`sysexit` instructions (even on 64-bit transitions). This is to
/// maintain symmetry with 32-bit transitions where setting SS actually will
/// actually have an effect.
#[derive(Debug)]
pub struct SS;
segment_impl!(SS, "ss", x86_64_asm_get_ss, x86_64_asm_load_ss);

/// Data Segment
///
/// Entirely unused in 64-bit mode; setting the segment register does nothing.
#[derive(Debug)]
pub struct DS;
segment_impl!(DS, "ds", x86_64_asm_get_ds, x86_64_asm_load_ds);

/// ES Segment
///
/// Entirely unused in 64-bit mode; setting the segment register does nothing.
#[derive(Debug)]
pub struct ES;
segment_impl!(ES, "es", x86_64_asm_get_es, x86_64_asm_load_es);

/// FS Segment
///
/// Only base is used in 64-bit mode, see [`Segment64`]. This is often used in
/// user-mode for Thread-Local Storage (TLS).
#[derive(Debug)]
pub struct FS;
segment_impl!(FS, "fs", x86_64_asm_get_fs, x86_64_asm_load_fs);
segment64_impl!(FS, "fs", FsBase, x86_64_asm_rdfsbase, x86_64_asm_wrfsbase);

/// GS Segment
///
/// Only base is used in 64-bit mode, see [`Segment64`]. In kernel-mode, the GS
/// base often points to a per-cpu kernel data structure.
#[derive(Debug)]
pub struct GS;
segment_impl!(GS, "gs", x86_64_asm_get_gs, x86_64_asm_load_gs);
segment64_impl!(GS, "gs", GsBase, x86_64_asm_rdgsbase, x86_64_asm_wrgsbase);

impl GS {
    /// Swap `KernelGsBase` MSR and `GsBase` MSR.
    ///
    /// ## Safety
    ///
    /// This function is unsafe because the caller must ensure that the
    /// swap operation cannot lead to undefined behavior.
    pub unsafe fn swap() {
        #[cfg(feature = "inline_asm")]
        asm!("swapgs", options(nostack, preserves_flags));

        #[cfg(not(feature = "inline_asm"))]
        crate::asm::x86_64_asm_swapgs();
    }
}

/// Alias for [`CS::set_reg()`]
#[deprecated(since = "0.14.4", note = "use `CS::set_reg()` instead")]
#[allow(clippy::missing_safety_doc)]
#[inline]
pub unsafe fn set_cs(sel: SegmentSelector) {
    CS::set_reg(sel)
}
/// Alias for [`SS::set_reg()`]
#[deprecated(since = "0.14.4", note = "use `SS::set_reg()` instead")]
#[allow(clippy::missing_safety_doc)]
#[inline]
pub unsafe fn load_ss(sel: SegmentSelector) {
    SS::set_reg(sel)
}
/// Alias for [`DS::set_reg()`]
#[deprecated(since = "0.14.4", note = "use `DS::set_reg()` instead")]
#[allow(clippy::missing_safety_doc)]
#[inline]
pub unsafe fn load_ds(sel: SegmentSelector) {
    DS::set_reg(sel)
}
/// Alias for [`ES::set_reg()`]
#[deprecated(since = "0.14.4", note = "use `ES::set_reg()` instead")]
#[allow(clippy::missing_safety_doc)]
#[inline]
pub unsafe fn load_es(sel: SegmentSelector) {
    ES::set_reg(sel)
}
/// Alias for [`FS::set_reg()`]
#[deprecated(since = "0.14.4", note = "use `FS::set_reg()` instead")]
#[allow(clippy::missing_safety_doc)]
#[inline]
pub unsafe fn load_fs(sel: SegmentSelector) {
    FS::set_reg(sel)
}
/// Alias for [`GS::set_reg()`]
#[deprecated(since = "0.14.4", note = "use `GS::set_reg()` instead")]
#[allow(clippy::missing_safety_doc)]
#[inline]
pub unsafe fn load_gs(sel: SegmentSelector) {
    GS::set_reg(sel)
}
/// Alias for [`GS::swap()`]
#[deprecated(since = "0.14.4", note = "use `GS::swap()` instead")]
#[allow(clippy::missing_safety_doc)]
#[inline]
pub unsafe fn swap_gs() {
    GS::swap()
}
/// Alias for [`CS::get_reg()`]
#[deprecated(since = "0.14.4", note = "use `CS::get_reg()` instead")]
#[allow(clippy::missing_safety_doc)]
#[inline]
pub fn cs() -> SegmentSelector {
    CS::get_reg()
}
/// Alias for [`FS::write_base()`].
///
/// Panics if the provided address is non-canonical.
#[deprecated(since = "0.14.4", note = "use `FS::write_base()` instead")]
#[allow(clippy::missing_safety_doc)]
#[inline]
pub unsafe fn wrfsbase(val: u64) {
    FS::write_base(VirtAddr::new(val))
}
/// Alias for [`FS::read_base()`]
#[deprecated(since = "0.14.4", note = "use `FS::read_base()` instead")]
#[allow(clippy::missing_safety_doc)]
#[inline]
pub unsafe fn rdfsbase() -> u64 {
    FS::read_base().as_u64()
}
/// Alias for [`GS::write_base()`].
///
/// Panics if the provided address is non-canonical.
#[deprecated(since = "0.14.4", note = "use `GS::write_base()` instead")]
#[allow(clippy::missing_safety_doc)]
#[inline]
pub unsafe fn wrgsbase(val: u64) {
    GS::write_base(VirtAddr::new(val))
}
/// Alias for [`GS::read_base()`]
#[deprecated(since = "0.14.4", note = "use `GS::read_base()` instead")]
#[allow(clippy::missing_safety_doc)]
#[inline]
pub unsafe fn rdgsbase() -> u64 {
    GS::read_base().as_u64()
}
