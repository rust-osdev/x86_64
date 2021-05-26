#![allow(missing_docs)]

//! TODO: Document module?

use core::marker::PhantomData;
use core::mem::{size_of, MaybeUninit};
use core::ptr::NonNull;

use crate::VirtAddr;

use super::segmentation::{rdfsbase, rdgsbase, wrfsbase, wrgsbase};

/// TODO: Document
pub trait Segment {
    unsafe fn get_base() -> VirtAddr;
    unsafe fn set_base(addr: VirtAddr);

    unsafe fn read_u64(off: usize) -> u64;
    unsafe fn read_u32(off: usize) -> u32;
    unsafe fn read_u16(off: usize) -> u16;
    unsafe fn read_u8(off: usize) -> u8;
    unsafe fn write_u64(off: usize, val: u64);
    unsafe fn write_u32(off: usize, val: u32);
    unsafe fn write_u16(off: usize, val: u16);
    unsafe fn write_u8(off: usize, val: u8);

    #[inline]
    unsafe fn read<T: Copy>(off: usize) -> T {
        let mut val: MaybeUninit<T> = MaybeUninit::uninit();
        read_ptr::<Self>(off, val.as_mut_ptr() as *mut u8, size_of::<T>());
        val.assume_init()
    }
    #[inline]
    unsafe fn write<T: Copy>(off: usize, val: T) {
        write_ptr::<Self>(off, &val as *const T as *const u8, size_of::<T>())
    }
}

/// TODO: Document
#[derive(Debug)]
pub struct Wrapper<S, T>(PhantomData<(S, *mut T)>);
unsafe impl<S, T> Send for Wrapper<S, T> {}
unsafe impl<S, T> Sync for Wrapper<S, T> {}

impl<S: Segment, T> Wrapper<S, T> {
    pub const fn new() -> Self {
        Self(PhantomData)
    }
    pub unsafe fn init(&self, new: Option<NonNull<T>>) -> Option<NonNull<T>> {
        let old = S::get_base().as_mut_ptr();
        S::set_base(match new {
            None => VirtAddr::new(0),
            Some(p) => VirtAddr::from_ptr(p.as_ptr()),
        });
        NonNull::new(old)
    }

    // Hidden helper functions to help with type deduction
    #[doc(hidden)]
    #[inline]
    pub const unsafe fn __uninit(&self) -> MaybeUninit<T> {
        MaybeUninit::uninit()
    }
    #[doc(hidden)]
    #[inline]
    pub unsafe fn __read<U: Copy>(&self, off: usize) -> U {
        S::read::<U>(off)
    }
    #[doc(hidden)]
    #[inline]
    pub unsafe fn __write<U: Copy>(&self, off: usize, _: *const U, val: U) {
        S::write::<U>(off, val)
    }
}

// Hidden helper functions to help with type deduction
#[doc(hidden)]
#[inline]
pub const unsafe fn __ptr_val_agree<U: Copy>(_: *const U, _: U) {}

/// TODO: Document
#[macro_export]
macro_rules! tls_read {
    ($wrapper:path, $field:tt) => {{
        // TODO: Move offset into const when this is stable
        let u: MaybeUninit<_> = $wrapper.__uninit();
        let base: *const _ = u.as_ptr();
        let field: *const _ = ::core::ptr::addr_of!((*base).$field);
        let offset: isize = (field as *const u8).offset_from(base as *const u8);

        let val = $wrapper.__read(offset as usize);
        __ptr_val_agree(field, val);
        val
    }};
}

/// TODO: Document
#[macro_export]
macro_rules! tls_write {
    ($wrapper:path, $field:tt, $val:expr) => {{
        let u: MaybeUninit<_> = $wrapper.__uninit();
        let base: *const _ = u.as_ptr();
        let field: *const _ = ::core::ptr::addr_of!((*base).$field);
        let offset: isize = (field as *const u8).offset_from(base as *const u8);

        $wrapper.__write(offset as usize, field, $val);
    }};
}

/// TODO: Document
#[derive(Debug)]
pub struct FS(());

impl Segment for FS {
    unsafe fn get_base() -> VirtAddr {
        // SAFETY: rdfsbase always returns a canonical address
        VirtAddr::new_unsafe(rdfsbase())
    }
    unsafe fn set_base(addr: VirtAddr) {
        wrfsbase(addr.as_u64())
    }
    unsafe fn read_u64(off: usize) -> u64 {
        let val: u64;
        asm!(
            "mov {}, qword ptr fs:[{}]",
            lateout(reg) val, in(reg) off,
            options(nostack, preserves_flags, pure, readonly),
        );
        val
    }
    unsafe fn read_u32(off: usize) -> u32 {
        let val: u32;
        asm!(
            "mov {:e}, dword ptr fs:[{}]",
            lateout(reg) val, in(reg) off,
            options(nostack, preserves_flags, pure, readonly),
        );
        val
    }
    unsafe fn read_u16(off: usize) -> u16 {
        let val: u32; // Avoid partial register issues
        asm!(
            "movzx {:e}, word ptr fs:[{}]",
            lateout(reg) val, in(reg) off,
            options(nostack, preserves_flags, pure, readonly),
        );
        val as u16
    }
    unsafe fn read_u8(off: usize) -> u8 {
        let val: u32; // Avoid partial register issues
        asm!(
            "movzx {:e}, byte ptr fs:[{}]",
            lateout(reg) val, in(reg) off,
            options(nostack, preserves_flags, pure, readonly),
        );
        val as u8
    }
    unsafe fn write_u64(off: usize, val: u64) {
        asm!(
            "mov qword ptr fs:[{}], {}",
            in(reg) off, in(reg) val,
            options(nostack, preserves_flags),
        );
    }
    unsafe fn write_u32(off: usize, val: u32) {
        asm!(
            "mov dword ptr fs:[{}], {:e}",
            in(reg) off, in(reg) val,
            options(nostack, preserves_flags),
        );
    }
    unsafe fn write_u16(off: usize, val: u16) {
        asm!(
            "mov word ptr fs:[{}], {:x}",
            in(reg) off, in(reg) val,
            options(nostack, preserves_flags),
        );
    }
    unsafe fn write_u8(off: usize, val: u8) {
        asm!(
            "mov byte ptr fs:[{}], {}",
            in(reg) off, in(reg_byte) val,
            options(nostack, preserves_flags),
        );
    }
}

/// TODO: Document
#[derive(Debug)]
pub struct GS(());

impl Segment for GS {
    unsafe fn get_base() -> VirtAddr {
        // SAFETY: rdfsbase always returns a canonical address
        VirtAddr::new_unsafe(rdgsbase())
    }
    unsafe fn set_base(addr: VirtAddr) {
        wrgsbase(addr.as_u64())
    }
    unsafe fn read_u64(off: usize) -> u64 {
        let val: u64;
        asm!(
            "mov {}, qword ptr gs:[{}]",
            lateout(reg) val, in(reg) off,
            options(nostack, preserves_flags, pure, readonly),
        );
        val
    }
    unsafe fn read_u32(off: usize) -> u32 {
        let val: u32;
        asm!(
            "mov {:e}, dword ptr gs:[{}]",
            lateout(reg) val, in(reg) off,
            options(nostack, preserves_flags, pure, readonly),
        );
        val
    }
    unsafe fn read_u16(off: usize) -> u16 {
        let val: u32; // Avoid partial register issues
        asm!(
            "movzx {:e}, word ptr gs:[{}]",
            lateout(reg) val, in(reg) off,
            options(nostack, preserves_flags, pure, readonly),
        );
        val as u16
    }
    unsafe fn read_u8(off: usize) -> u8 {
        let val: u32; // Avoid partial register issues
        asm!(
            "movzx {:e}, byte ptr gs:[{}]",
            lateout(reg) val, in(reg) off,
            options(nostack, preserves_flags, pure, readonly),
        );
        val as u8
    }
    unsafe fn write_u64(off: usize, val: u64) {
        asm!(
            "mov qword ptr gs:[{}], {}",
            in(reg) off, in(reg) val,
            options(nostack, preserves_flags),
        );
    }
    unsafe fn write_u32(off: usize, val: u32) {
        asm!(
            "mov dword ptr gs:[{}], {:e}",
            in(reg) off, in(reg) val,
            options(nostack, preserves_flags),
        );
    }
    unsafe fn write_u16(off: usize, val: u16) {
        asm!(
            "mov word ptr gs:[{}], {:x}",
            in(reg) off, in(reg) val,
            options(nostack, preserves_flags),
        );
    }
    unsafe fn write_u8(off: usize, val: u8) {
        asm!(
            "mov byte ptr gs:[{}], {}",
            in(reg) off, in(reg_byte) val,
            options(nostack, preserves_flags),
        );
    }
}

#[inline]
unsafe fn read_ptr<S: Segment + ?Sized>(off: usize, p: *mut u8, size: usize) {
    if size >= 8 {
        (p as *mut u64).write_unaligned(S::read_u64(off));
        read_ptr::<S>(off + 8, p.offset(8), size - 8);
    } else if size == 4 {
        (p as *mut u32).write_unaligned(S::read_u32(off));
    } else if size == 2 {
        (p as *mut u16).write_unaligned(S::read_u16(off));
    } else if size == 1 {
        p.write(S::read_u8(off));
    } else if size > 0 {
        read_cold::<S>(off, p, size);
    }
}

#[cold]
unsafe fn read_cold<S: Segment + ?Sized>(off: usize, p: *mut u8, size: usize) {
    match size {
        7 => {
            (p as *mut u32).write_unaligned(S::read_u32(off));
            (p.offset(4) as *mut u16).write_unaligned(S::read_u16(off + 4));
            p.offset(6).write(S::read_u8(off + 6));
        }
        6 => {
            (p as *mut u32).write_unaligned(S::read_u32(off));
            (p.offset(4) as *mut u16).write_unaligned(S::read_u16(off + 4));
        }
        5 => {
            (p as *mut u32).write_unaligned(S::read_u32(off));
            p.offset(4).write(S::read_u8(off + 4));
        }
        3 => {
            (p as *mut u16).write_unaligned(S::read_u16(off));
            p.offset(2).write(S::read_u8(off + 2));
        }
        _ => core::hint::unreachable_unchecked(),
    }
}

#[inline]
unsafe fn write_ptr<S: Segment + ?Sized>(off: usize, p: *const u8, size: usize) {
    if size >= 8 {
        S::write_u64(off, (p as *const u64).read_unaligned());
        write_ptr::<S>(off + 8, p.offset(8), size - 8);
    } else if size == 4 {
        S::write_u32(off, (p as *const u32).read_unaligned());
    } else if size == 2 {
        S::write_u16(off, (p as *const u16).read_unaligned());
    } else if size == 1 {
        S::write_u8(off, p.read());
    } else if size > 0 {
        write_cold::<S>(off, p, size);
    }
}

#[cold]
unsafe fn write_cold<S: Segment + ?Sized>(off: usize, p: *const u8, size: usize) {
    match size {
        7 => {
            S::write_u32(off, (p as *const u32).read_unaligned());
            S::write_u16(off + 4, (p.offset(4) as *const u16).read_unaligned());
            S::write_u8(off + 6, p.offset(6).read_unaligned());
        }
        6 => {
            S::write_u32(off, (p as *const u32).read_unaligned());
            S::write_u16(off + 4, (p.offset(4) as *const u16).read_unaligned());
        }
        5 => {
            S::write_u32(off, (p as *const u32).read_unaligned());
            S::write_u8(off + 4, p.offset(4).read_unaligned());
        }
        3 => {
            S::write_u16(off, (p as *const u16).read_unaligned());
            S::write_u8(off + 2, p.offset(2).read_unaligned());
        }
        _ => core::hint::unreachable_unchecked(),
    }
}
