#![allow(non_upper_case_globals)]

use core::prelude::*;
use core::mem::size_of;

bitflags!(
	flags EFlags: u32 {
		static CarryFlag = 1 << 0,
		static ParityFlag = 1 << 2,
		static AdjustFlag = 1 << 4,
		static ZeroFlag = 1 << 6,
		static SignFlag = 1 << 7,
		static TrapFlag = 1 << 8,
		static InterruptFlag = 1 << 9,
		static DirectionFlag = 1 << 10,
		static OverflowFlag = 1 << 11,
		static Iopl1 = 1 << 12,
		static Iopl2 = 1 << 13,
		static NestedTaskFlag = 1 << 14,
		static ResumeFlag = 1 << 16,
		static Virtual8086Flag = 1 << 17,
		static AlignmentFlag = 1 << 18,
		static VirtualInterruptFlag = 1 << 19,
		static VirtualInterruptPending = 1 << 20,
		static CpuIdFlag = 1 << 21
	}
);

bitflags!(
	flags Cr0Flags: u32 {
		static ProtectedMode = 1 << 0,
		static MonitorCoprocessor = 1 << 1,
		static EmulateCoprocessor = 1 << 2,
		static TaskSwitched = 1 << 3,
		static ExtensionType = 1 << 4,
		static NumericError = 1 << 5,
		static WriteProtect = 1 << 16,
		static AlignmentMask = 1 << 18,
		static NotWriteThrough = 1 << 29,
		static CacheDisable = 1 << 30,
		static EnablePaging = 1 << 31
	}
);

bitflags!(
	flags Cr4Flags: u32 {
		static EnableVme = 1 << 0,
		static VirtualInterrupts = 1 << 1,
		static TimeStampDisable = 1 << 2,
		static DebuggingExtensions = 1 << 3,
		static EnablePse = 1 << 4,
		static EnablePae = 1 << 5,
		static EnableMachineCheck = 1 << 6,
		static EnableGlobalPages = 1 << 7,
		static EnablePpmc = 1 << 8,
		static EnableSse = 1 << 9,
		static UnmaskedSse = 1 << 10,
		static EnableVmx = 1 << 13,
		static EnableSmx = 1 << 14,
		static EnablePcid = 1 << 17,
		static EnableOsXSave = 1 << 18,
		static EnableSmep = 1 << 20,
		static EnableSmap = 1 << 21
	}
);

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

	pub fn new(base: u32, limit: usize, access: u8) -> GdtEntry {
		let (limit, flags) = if limit < 0x100000 {
			((limit & 0xFFFF) as u16, ((limit & 0xF0000) >> 16) as u8 | 0x40u8)
		} else {
			if ((limit - 0xFFF) & 0xFFF) > 0 {
				panic!("bad segment limit for GDT entry");
			}
			(((limit & 0xFFFF000) >> 12) as u16, ((limit & 0xF0000000) >> 28) as u8 | 0xC0u8)
		};
		GdtEntry {
			base1: (base & 0xFFFF) as u16,
			base2: ((base & 0xFF0000) >> 16) as u8,
			base3: ((base & 0xFF000000) >> 24) as u8,
			access: access,
			limit: limit,
			flags: flags
		}
	}
}

pub const NULL_IDT_ENTRY: IdtEntry = IdtEntry { offset1: 0, selector: 0, reserved: 0, flags: 0, offset2: 0 };

impl IdtEntry {
	pub fn new(f: unsafe extern "C" fn(), dpl: usize, block: bool) -> IdtEntry {
		IdtEntry {
			offset1: ((f as usize) & 0xFFFF) as u16,
			offset2: (((f as usize) & 0xFFFF0000) >> 16) as u16,
			selector: 8,
			reserved: 0,
			flags: if block { 0x8E } else { 0x8F } | ((dpl as u8 & 3) << 5)
		}
	}
}

#[derive(Copy)]
#[repr(C, packed)]
pub struct Tss {
	pub link: u16,
	pub reserved0: u16,
	pub esp0: u32,
	pub ss0: u16,
	pub reserved1: u16,
	pub esp1: u32,
	pub ss1: u16,
	pub reserved2: u16,
	pub esp2: u32,
	pub ss2: u16,
	pub reserved3: u16,

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
	pub reserved4: u16,
	pub cs: u16,
	pub reserved5: u16,
	pub ss: u16,
	pub reserved6: u16,
	pub ds: u16,
	pub reserved7: u16,
	pub fs: u16,
	pub reserved8: u16,
	pub gs: u16,
	pub reserved9: u16,
	pub ldtr: u16,
	pub reserved10: u32,
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
pub unsafe fn jump_stack(stack: *mut (), ip: *const ()) -> ! {
	asm!("mov esp, $0; jmp $1" :: "rg"(stack), "r"(ip) :: "volatile", "intel");
	loop { }
}
