#![allow(non_upper_case_globals)]

pub use self::x86_shared::*;

use core::prelude::*;
use core::mem::size_of;

mod x86_shared;

bitflags! {
	flags EFlags: u32 {
		const CarryFlag = 1 << 0,
		const ParityFlag = 1 << 2,
		const AdjustFlag = 1 << 4,
		const ZeroFlag = 1 << 6,
		const SignFlag = 1 << 7,
		const TrapFlag = 1 << 8,
		const InterruptFlag = 1 << 9,
		const DirectionFlag = 1 << 10,
		const OverflowFlag = 1 << 11,
		const Iopl1 = 1 << 12,
		const Iopl2 = 1 << 13,
		const NestedTaskFlag = 1 << 14,
		const ResumeFlag = 1 << 16,
		const Virtual8086Flag = 1 << 17,
		const AlignmentFlag = 1 << 18,
		const VirtualInterruptFlag = 1 << 19,
		const VirtualInterruptPending = 1 << 20,
		const CpuIdFlag = 1 << 21
	}
}

bitflags! {
	flags Cr0Flags: u32 {
		const ProtectedMode = 1 << 0,
		const MonitorCoprocessor = 1 << 1,
		const EmulateCoprocessor = 1 << 2,
		const TaskSwitched = 1 << 3,
		const ExtensionType = 1 << 4,
		const NumericError = 1 << 5,
		const WriteProtect = 1 << 16,
		const AlignmentMask = 1 << 18,
		const NotWriteThrough = 1 << 29,
		const CacheDisable = 1 << 30,
		const EnablePaging = 1 << 31
	}
}

bitflags! {
	flags Cr4Flags: u32 {
		const EnableVme = 1 << 0,
		const VirtualInterrupts = 1 << 1,
		const TimeStampDisable = 1 << 2,
		const DebuggingExtensions = 1 << 3,
		const EnablePse = 1 << 4,
		const EnablePae = 1 << 5,
		const EnableMachineCheck = 1 << 6,
		const EnableGlobalPages = 1 << 7,
		const EnablePpmc = 1 << 8,
		const EnableSse = 1 << 9,
		const UnmaskedSse = 1 << 10,
		const EnableVmx = 1 << 13,
		const EnableSmx = 1 << 14,
		const EnablePcid = 1 << 17,
		const EnableOsXSave = 1 << 18,
		const EnableSmep = 1 << 20,
		const EnableSmap = 1 << 21
	}
}

bitflags! {
	flags GdtAccess: u8 {
		const Accessed = 1 << 0,
		const Writable = 1 << 1,
		const Direction = 1 << 2,
		const Executable = 1 << 3,
		const NotTss = 1 << 4,
	}
}

#[derive(Copy)]
#[repr(C, packed)]
pub struct GdtEntry {
	limit: u16,
	base1: u16,
	base2: u8,
	access: u8,
	flags: u8,
	base3: u8,
}

#[derive(Copy)]
#[repr(C, packed)]
pub struct IdtEntry {
	offset1: u16,
	selector: u16,
	reserved: u8,
	flags: u8,
	offset2: u16
}

impl GdtEntry {
	pub fn null() -> GdtEntry {
		GdtEntry {
			base1: 0,
			base2: 0,
			base3: 0,
			access: 0,
			limit: 0,
			flags: 0
		}
	}

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

pub const NULL_IDT_ENTRY: IdtEntry = IdtEntry { offset1: 0, selector: 0, reserved: 0, flags: 0, offset2: 0 };

impl IdtEntry {
	pub fn new(f: unsafe extern "C" fn(), dpl: PrivilegeLevel, block: bool) -> IdtEntry {
		IdtEntry {
			offset1: f as u16,
			offset2: ((f as usize & 0xFFFF0000) >> 16) as u16,
			selector: 8,
			reserved: 0,
			flags: if block { 0x8E } else { 0x8F } | ((dpl as u8) << 5)
		}
	}
}

#[derive(Copy)]
#[repr(C, packed)]
pub struct Tss {
	pub link: u16,
	reserved0: u16,
	pub esp0: u32,
	pub ss0: u16,
	reserved1: u16,
	pub esp1: u32,
	pub ss1: u16,
	reserved2: u16,
	pub esp2: u32,
	pub ss2: u16,
	reserved3: u16,

	pub cr3: u32,
	pub eip: u32,
	pub eflags: u32,

	pub eax: u32,
	pub ecx: u32,
	pub edx: u32,
	pub ebx: u32,
	pub esp: u32,
	pub ebp: u32,
	pub esi: u32,
	pub edi: u32,

	pub es: u16,
	reserved4: u16,
	pub cs: u16,
	reserved5: u16,
	pub ss: u16,
	reserved6: u16,
	pub ds: u16,
	reserved7: u16,
	pub fs: u16,
	reserved8: u16,
	pub gs: u16,
	reserved9: u16,
	pub ldtr: u16,
	reserved10: u32,
	pub iobp_offset: u16
}

impl Tss {
	pub fn new() -> Tss {
		Tss {
			link: 0,
			reserved0: 0,
			esp0: 0,
			ss0: 0,
			reserved1: 0,
			esp1: 0,
			ss1: 0,
			reserved2: 0,
			esp2: 0,
			ss2: 0,
			reserved3: 0,
			cr3: 0,
			eip: 0,
			eflags: 0,
			eax: 0,
			ecx: 0,
			edx: 0,
			ebx: 0,
			esp: 0,
			ebp: 0,
			esi: 0,
			edi: 0,
			es: 0,
			reserved4: 0,
			cs: 0,
			reserved5: 0,
			ss: 0,
			reserved6: 0,
			ds: 0,
			reserved7: 0,
			fs: 0,
			reserved8: 0,
			gs: 0,
			reserved9: 0,
			ldtr: 0,
			reserved10: 0,
			iobp_offset: size_of::<Tss>() as u16
		}
	}
}

#[inline(always)]
pub fn get_cr0() -> Cr0Flags {
	unsafe {
		let mut r: u32;
		asm!("mov $0, cr0" : "=r"(r) ::: "intel");
		Cr0Flags::from_bits_truncate(r)
	}
}

#[inline(always)]
pub fn get_cr2() -> u32 {
	unsafe {
		let mut r: u32;
		asm!("mov $0, cr2" : "=r"(r) ::: "intel");
		r
	}
}

#[inline(always)]
pub fn get_cr3() -> u32 {
	unsafe {
		let mut r: u32;
		asm!("mov $0, cr3" : "=r"(r) ::: "intel");
		r
	}
}

#[inline(always)]
pub fn get_cr4() -> Cr4Flags {
	unsafe {
		let mut r: u32;
		asm!("mov $0, cr4" : "=r"(r) ::: "intel");
		Cr4Flags::from_bits_truncate(r)
	}
}

#[inline(always)]
pub fn get_flags() -> EFlags {
	unsafe {
		let mut r: u32;
		asm!("pushfd; pop $0" : "=r"(r) ::: "intel");
		EFlags::from_bits_truncate(r)
	}
}


#[inline(always)]
pub unsafe fn set_cr0(flags: Cr0Flags) {
	asm!("mov cr0, $0" :: "r"(flags.bits()) :: "volatile", "intel");
}

#[inline(always)]
pub unsafe fn set_cr3(val: u32) {
	asm!("mov cr3, $0" :: "r"(val) :: "volatile", "intel");
}

#[inline(always)]
pub unsafe fn set_cr4(flags: Cr4Flags) {
	asm!("mov cr4, $0" :: "r"(flags.bits()) :: "volatile", "intel");
}

#[inline(always)]
pub unsafe fn set_gdt(gdt: &[GdtEntry]) {
	#[repr(C, packed)]
	struct GDTR {
		limit: u16,
		ptr: *const GdtEntry,
	}
	asm!("lgdt $0" :: "*m"(&GDTR { ptr: gdt.as_ptr(), limit: (gdt.len()*8 - 1) as u16 }) :: "volatile", "intel");
}

#[inline(always)]
pub unsafe fn set_idt(idt: &[IdtEntry]) {
	#[repr(C, packed)]
	struct IDTR {
		limit: u16,
		ptr: *const IdtEntry,
	}
	asm!("lidt $0" :: "*m"(&IDTR { ptr: idt.as_ptr(), limit: idt.len() as u16 * 8 }) :: "volatile", "intel");
}

#[inline(always)]
pub unsafe fn set_flags(val: EFlags) {
	asm!("push $0; popfd" :: "r"(val.bits()) : "flags" : "volatile", "intel");
}

#[inline(always)]
pub unsafe fn stack_jmp(stack: *mut (), ip: *const ()) -> ! {
	asm!("mov esp, $0; jmp $1" :: "rg"(stack), "r"(ip) :: "volatile", "intel");
	loop { }
}
