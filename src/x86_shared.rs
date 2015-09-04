#![allow(non_upper_case_globals)]

bitflags! {
	flags Flags: usize {
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
	flags Cr0: usize {
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
	flags Cr4: usize {
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

bitflags!(
	flags Features: u64 {
		const Fpu = 1 << 0,
		const Virtual8086 = 1 << 1,
		const DebugExtension = 1 << 2,
		const PageSizeExtension = 1 << 3,
		const TimeStampCounter = 1 << 4,
		const ModelSpecificRegister = 1 << 5,
		const PhysicalAddressExtension = 1 << 6,
		const MachineCheckException = 1 << 7,
		const Cx8 = 1 << 8, // CMPXCHG8
		const Apic = 1 << 9,
		const SysEnter = 1 << 11,
		const MemoryTypeRange = 1 << 12,
		const PageGlobal = 1 << 13,
		const MachineCheckArchitecture = 1 << 14,
		const CMov = 1 << 15,
		const PageAttributeTable = 1 << 16,
		const PageSizeExtension36 = 1 << 17,
		const ProcessorSerial = 1 << 18,
		const CacheFlush = 1 << 19,
		const DebugStore = 1 << 21,
		const Acpi = 1 << 22,
		const Mmx = 1 << 23,
		const FxSave = 1 << 24,
		const Sse = 1 << 25,
		const Sse2 = 1 << 26,
		const SelfSnoop = 1 << 27,
		const HyperThreading = 1 << 28,
		const ThermalMonitor = 1 << 29,
		const Ia64 = 1 << 30,
		const PendingBreak = 1 << 31,

		const Sse3 = 1 << (32 + 0),
		const PclMulQdq = 1 << (32 + 1), // what
		const DebugStore64 = 1 << (32 + 2),
		const Monitor = 1 << (32 + 3),
		const CplDebugStore = 1 << (32 + 4),
		const Vmx = 1 << (32 + 5),
		const SaferMode = 1 << (32 + 6),
		const EnhancedSpeedStep = 1 << (32 + 7),
		const ThermalMonitor2 = 1 << (32 + 8),
		const Ssse3 = 1 << (32 + 9),
		const L1ContextId = 1 << (32 + 10),
		const Fma = 1 << (32 + 12),
		const Cx16 = 1 << (32 + 13), // CMPXCHG16B
		const Xtpr = 1 << (32 + 14), // I have no idea what this is
		const PerformanceMonitor = 1 << (32 + 15),
		const ProcessContextId = 1 << (32 + 17),
		const DirectCache = 1 << (32 + 18),
		const Sse41 = 1 << (32 + 19),
		const Sse42 = 1 << (32 + 20),
		const X2Apic = 1 << (32 + 21),
		const MovBe = 1 << (32 + 22),
		const PopulationCount = 1 << (32 + 23),
		const TscDeadline = 1 << (32 + 24),
		const AesNi = 1 << (32 + 25),
		const XSave = 1 << (32 + 26),
		const OsXSave = 1 << (32 + 27),
		const Avx = 1 << (32 + 28),
		const HalfPrecision = 1 << (32 + 29),
		const HwRandom = 1 << (32 + 30)
	}
);

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum Exception {
	DivisionByZero = 0,
	Debug = 1,
	Nmi = 2,
	Breakpoint = 3,
	Overflow = 4,
	Bounds = 5,
	InvalidOpcode = 6,
	NotAvailable = 7,
	DoubleFault = 8,
	CoprocessorSegment = 9,
	Tss = 10,
	NotPresent = 11,
	StackSegment = 12,
	GeneralProtection = 13,
	PageFault = 14,
	Fpu = 16,
	Alignment = 17,
	MachineCheck = 18,
	Simd = 19,
	Virtualization = 20,
	Security = 30
}

impl Exception {
	pub fn from_code(code: u32) -> Option<Exception> {
		Some(match code {
			0 => Exception::DivisionByZero,
			1 => Exception::Debug,
			2 => Exception::Nmi,
			3 => Exception::Breakpoint,
			4 => Exception::Overflow,
			5 => Exception::Bounds,
			6 => Exception::InvalidOpcode,
			7 => Exception::NotAvailable,
			8 => Exception::DoubleFault,
			9 => Exception::CoprocessorSegment,
			10 => Exception::Tss,
			11 => Exception::NotPresent,
			12 => Exception::StackSegment,
			13 => Exception::GeneralProtection,
			14 => Exception::PageFault,
			16 => Exception::Fpu,
			17 => Exception::Alignment,
			18 => Exception::MachineCheck,
			19 => Exception::Simd,
			20 => Exception::Virtualization,
			30 => Exception::Security,
			
			_ => return None
		})
	}
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum Msr {
	ApicBase = 0x1B
}

#[inline(always)]
pub fn cpuid(function: u32) -> (u32, u32, u32, u32) {
	unsafe {
		let (eax, ebx, ecx, edx): (u32, u32, u32, u32);
		asm!("cpuid" : "={eax}"(eax), "={ebx}"(ebx), "={ecx}"(ecx), "={edx}"(edx) : "{eax}"(function));
		(eax, ebx, ecx, edx)
	}
}

#[inline(always)]
pub fn supports() -> Features {
	let (_, _, feature_ecx, feature_edx) = cpuid(1);
	Features {
		bits: ((feature_ecx as u64) << 32) | (feature_edx as u64)
	}
}

#[inline(always)]
pub unsafe fn read_msr(msr: Msr) -> u64 {
	let (r1, r2): (u32, u32);
	asm!("rdmsr" : "={eax}"(r1), "={edx}"(r2) : "{ecx}"(msr as u32) :: "intel");
	r1 as u64 | ((r2 as u64) << 32)
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum PrivilegeLevel {
	Ring0 = 0,
	Ring1 = 1,
	Ring2 = 2,
	Ring3 = 3,
}

#[derive(Copy, Clone, PartialEq, Eq)]
#[repr(C, packed)]
pub struct SegmentSelector {
	data: u16
}

impl SegmentSelector {
	#[inline(always)]
	pub fn new(index: u16, rpl: PrivilegeLevel) -> SegmentSelector {
		SegmentSelector {
			data: index << 3 | rpl as u16
		}
	}

	pub fn bits(&self) -> u16 {
		self.data
	}
}

#[inline(always)]
pub unsafe fn set_tr(selector: SegmentSelector) {
	asm!("ltr $0" :: "r"(selector.bits()) :: "volatile", "intel");
}

#[inline(always)]
pub unsafe fn set_ss(selector: SegmentSelector) {
	asm!("mov ss, $0" :: "r"(selector.bits()) :: "volatile", "intel");
}

#[inline(always)]
pub unsafe fn set_ds(selector: SegmentSelector) {
	asm!("mov ds, $0" :: "r"(selector.bits()) :: "volatile", "intel");
}

#[inline(always)]
pub unsafe fn set_es(selector: SegmentSelector) {
	asm!("mov es, $0" :: "r"(selector.bits()) :: "volatile", "intel");
}

#[inline(always)]
pub unsafe fn set_gs(selector: SegmentSelector) {
	asm!("mov gs, $0" :: "r"(selector.bits()) :: "volatile", "intel");
}

#[inline(always)]
pub unsafe fn set_fs(selector: SegmentSelector) {
	asm!("mov fs, $0" :: "r"(selector.bits()) :: "volatile", "intel");
}

#[inline(always)]
pub unsafe fn set_cs(selector: SegmentSelector) {
	asm!("push $0;
		push $$1f
		lret;
		1:" :: "ri"(selector.bits() as usize) :: "volatile");
}

#[inline(always)]
pub fn get_cr0() -> Cr0 {
	unsafe {
		let r: usize;
		asm!("mov $0, cr0" : "=r"(r) ::: "intel");
		Cr0::from_bits_truncate(r)
	}
}

#[inline(always)]
pub fn get_cr2() -> usize {
	unsafe {
		let r: usize;
		asm!("mov $0, cr2" : "=r"(r) ::: "intel");
		r
	}
}

#[inline(always)]
pub fn get_cr3() -> usize {
	unsafe {
		let r: usize;
		asm!("mov $0, cr3" : "=r"(r) ::: "intel");
		r
	}
}

#[inline(always)]
pub fn get_cr4() -> Cr4 {
	unsafe {
		let r: usize;
		asm!("mov $0, cr4" : "=r"(r) ::: "intel");
		Cr4::from_bits_truncate(r)
	}
}

#[inline(always)]
pub unsafe fn set_cr0(flags: Cr0) {
	asm!("mov cr0, $0" :: "r"(flags.bits()) :: "volatile", "intel");
}

#[inline(always)]
pub unsafe fn set_cr3(val: usize) {
	asm!("mov cr3, $0" :: "r"(val) :: "volatile", "intel");
}

#[inline(always)]
pub unsafe fn set_cr4(flags: Cr4) {
	asm!("mov cr4, $0" :: "r"(flags.bits()) :: "volatile", "intel");
}

#[inline(always)]
pub unsafe fn enable_interrupts() {
	asm!("sti" :::: "volatile", "intel");
}

#[inline(always)]
pub unsafe fn disable_interrupts() {
	asm!("cli" :::: "volatile", "intel");
}

#[inline(always)]
pub unsafe fn halt() {
	asm!("hlt" :::: "volatile", "intel");
}

#[inline(always)]
pub unsafe fn out8(port: u16, value: u8) {
	asm!("out $0, $1" :: "{dx}"(port), "{al}"(value) :: "volatile", "intel");
}

#[inline(always)]
pub unsafe fn out16(port: u16, value: u16) {
	asm!("out $0, $1" :: "{dx}"(port), "{ax}"(value) :: "volatile", "intel");
}

#[inline(always)]
pub unsafe fn out32(port: u16, value: u32) {
	asm!("out $0, $1" :: "{dx}"(port), "{eax}"(value) :: "volatile", "intel");
}

#[inline(always)]
pub unsafe fn outs8(port: u16, buf: &[u8]) {
	asm!("rep outsb dx, [esi]" :: "{ecx}"(buf.len()), "{dx}"(port), "{esi}"(buf.as_ptr()) : "ecx", "edi" : "intel");
}

#[inline(always)]
pub unsafe fn outs16(port: u16, buf: &[u16]) {
	asm!("rep outsw dx, [esi]" :: "{ecx}"(buf.len()), "{dx}"(port), "{esi}"(buf.as_ptr()) : "ecx", "edi" : "intel");
}

#[inline(always)]
pub unsafe fn outs32(port: u16, buf: &[u32]) {
	asm!("rep outsd dx, [esi]" :: "{ecx}"(buf.len()), "{dx}"(port), "{esi}"(buf.as_ptr()) : "ecx", "edi" : "intel");
}


#[inline(always)]
pub unsafe fn in8(port: u16) -> u8 {
	let r: u8;
	asm!("in $0, $1" : "={al}"(r) : "{dx}"(port) :: "intel");
	r
}

#[inline(always)]
pub unsafe fn in16(port: u16) -> u16 {
	let r: u16;
	asm!("in $0, $1" : "={ax}"(r) : "{dx}"(port) :: "intel");
	r
}

#[inline(always)]
pub unsafe fn in32(port: u16) -> u32 {
	let r: u32;
	asm!("in $0, $1" : "={eax}"(r) : "{dx}"(port) :: "intel");
	r
}

#[inline(always)]
pub unsafe fn ins8(port: u16, buf: &mut [u8]) {
	asm!("rep insb [edi], dx" :: "{ecx}"(buf.len()), "{dx}"(port), "{edi}"(buf.as_ptr()) : "ecx", "edi" : "intel");
}

#[inline(always)]
pub unsafe fn ins16(port: u16, buf: &mut [u16]) {
	asm!("rep insw [edi], dx" :: "{ecx}"(buf.len()), "{dx}"(port), "{edi}"(buf.as_ptr()) : "ecx", "edi" : "intel");
}

#[inline(always)]
pub unsafe fn ins32(port: u16, buf: &mut [u32]) {
	asm!("rep insd [edi], dx" :: "{ecx}"(buf.len()), "{dx}"(port), "{edi}"(buf.as_ptr()) : "ecx", "edi" : "intel");
}
