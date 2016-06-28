#![allow(non_upper_case_globals)]

pub mod irq;
pub mod task;

pub use shared::{
    Flags,
    PrivilegeLevel,
};
pub use self::irq::IdtEntry;

use core::mem::size_of;

bitflags! {
    pub flags GdtAccess: u8 {
        const Accessed = 1 << 0,
        const Writable = 1 << 1,
        const Direction = 1 << 2,
        const Executable = 1 << 3,
        const NotTss = 1 << 4,
    }
}

#[derive(Copy, Clone, Debug)]
#[repr(C, packed)]
pub struct GdtEntry {
    limit: u16,
    base1: u16,
    base2: u8,
    access: u8,
    flags: u8,
    base3: u8,
}

impl GdtEntry {
    pub const NULL: GdtEntry = GdtEntry {
        base1: 0,
        base2: 0,
        base3: 0,
        access: 0,
        limit: 0,
        flags: 0
    };

    pub fn new(base: *const (), limit: usize, access: GdtAccess, dpl: PrivilegeLevel) -> GdtEntry {
        let (limit, flags) = if limit < 0x100000 {
            ((limit & 0xFFFF) as u16, ((limit & 0xF0000) >> 16) as u8 | 0x40u8)
        } else {
            if ((limit - 0xFFF) & 0xFFF) > 0 {
                panic!("bad segment limit for GDT entry");
            }
            (((limit & 0xFFFF000) >> 12) as u16, ((limit & 0xF0000000) >> 28) as u8 | 0xC0u8)
        };
        GdtEntry {
            base1: base as u16,
            base2: ((base as usize & 0xFF0000) >> 16) as u8,
            base3: ((base as usize & 0xFF000000) >> 24) as u8,
            access: access.bits() | ((dpl as u8) << 5) | 0x80,
            limit: limit,
            flags: flags
        }
    }
}

#[inline(always)]
pub fn get_flags() -> Flags {
    unsafe {
        let r: usize;
        asm!("pushfd; pop $0" : "=r"(r) ::: "intel");
        Flags::from_bits_truncate(r)
    }
}

#[inline(always)]
pub unsafe fn set_flags(val: Flags) {
    asm!("push $0; popfd" :: "r"(val.bits()) : "flags" : "volatile", "intel");
}

#[inline(always)]
pub unsafe fn set_gdt(gdt: &[GdtEntry]) {
    #[repr(C, packed)]
    struct GDTR {
        limit: u16,
        ptr: *const GdtEntry,
    }
    asm!("lgdtl $0" :: "*m"(&GDTR { ptr: gdt.as_ptr(), limit: (gdt.len()*size_of::<GdtEntry>() - 1) as u16 }) :: "volatile");
}

#[inline(always)]
pub unsafe fn set_idt(idt: &[IdtEntry]) {
    #[repr(C, packed)]
    struct IDTR {
        limit: u16,
        ptr: *const IdtEntry,
    }
    asm!("lidtl $0" :: "*m"(&IDTR { ptr: idt.as_ptr(), limit: idt.len() as u16 * 8 }) :: "volatile");
}

#[inline(always)]
pub unsafe fn stack_jmp(stack: *mut (), ip: *const ()) -> ! {
    asm!("mov esp, $0; jmp $1" :: "rg"(stack), "r"(ip) :: "volatile", "intel");
    loop { }
}
