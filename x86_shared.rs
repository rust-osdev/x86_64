
#[derive(Copy)]
pub enum Feature {
	Fpu,
	Virtual8086,
	DebugExtension,
	PageSizeExtension,
	TimeStampCounter,
	ModelSpecificRegister,
	PhysicalAddressExtension,
	MachineCheckException,
	Cx8, // CMPXCHG8
	Apic,
	SysEnter,
	MemoryTypeRange,
	PageGlobal,
	MachineCheckArchitecture,
	CMov,
	PageAttributeTable,
	PageSizeExtension36,
	ProcessorSerial,
	CacheFlush,
	DebugStore,
	Acpi,
	Mmx,
	FxSave,
	Sse,
	Sse2,
	SelfSnoop,
	HyperThreading,
	ThermalMonitor,
	Ia64,
	PendingBreak,
	Sse3,
	PclMulQdq, // what
	DebugStore64,
	Monitor,
	CplDebugStore,
	Vmx,
	SaferMode,
	EnhancedSpeedStep,
	ThermalMonitor2,
	Ssse3,
	L1ContextId,
	Fma,
	Cx16, // CMPXCHG16B
	Xtpr, // I have no idea what this is
	PerformanceMonitor,
	ProcessContextId,
	DirectCache,
	Sse41,
	Sse42,
	X2Apic,
	MovBe,
	PopulationCount,
	TscDeadline,
	AesNi,
	XSave,
	OsXSave,
	Avx,
	HalfPrecision,
	HwRandom
}

#[derive(Copy, FromPrimitive)]
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

#[derive(Copy)]
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
pub fn supports(feature: Feature) -> bool {
	let (_, _, feature_ecx, feature_edx) = cpuid(1);
	0 != match feature { // this is bad
		Feature::Fpu => feature_edx & (1 << 0),
		Feature::Virtual8086 => feature_edx & (1 << 1),
		Feature::DebugExtension => feature_edx & (1 << 2),
		Feature::PageSizeExtension => feature_edx & (1 << 3),
		Feature::TimeStampCounter => feature_edx & (1 << 4),
		Feature::ModelSpecificRegister => feature_edx & (1 << 5),
		Feature::PhysicalAddressExtension => feature_edx & (1 << 6),
		Feature::MachineCheckException => feature_edx & (1 << 7),
		Feature::Cx8 => feature_edx & (1 << 8),
		Feature::Apic => feature_edx & (1 << 9),
		Feature::SysEnter => feature_edx & (1 << 11),
		Feature::MemoryTypeRange => feature_edx & (1 << 12),
		Feature::PageGlobal => feature_edx & (1 << 13),
		Feature::MachineCheckArchitecture => feature_edx & (1 << 14),
		Feature::CMov => feature_edx & (1 << 15),
		Feature::PageAttributeTable => feature_edx & (1 << 16),
		Feature::PageSizeExtension36 => feature_edx & (1 << 17),
		Feature::ProcessorSerial => feature_edx & (1 << 18),
		Feature::CacheFlush => feature_edx & (1 << 19),
		Feature::DebugStore => feature_edx & (1 << 21),
		Feature::Acpi => feature_edx & (1 << 22),
		Feature::Mmx => feature_edx & (1 << 23),
		Feature::FxSave => feature_edx & (1 << 24),
		Feature::Sse => feature_edx & (1 << 25),
		Feature::Sse2 => feature_edx & (1 << 26),
		Feature::SelfSnoop => feature_edx & (1 << 27),
		Feature::HyperThreading => feature_edx & (1 << 28),
		Feature::ThermalMonitor => feature_edx & (1 << 29),
		Feature::Ia64 => feature_edx & (1 << 30),
		Feature::PendingBreak => feature_edx & (1 << 31),

		Feature::Sse3 => feature_ecx & (1 << 0),
		Feature::PclMulQdq => feature_ecx & (1 << 1),
		Feature::DebugStore64 => feature_ecx & (1 << 2),
		Feature::Monitor => feature_ecx & (1 << 3),
		Feature::CplDebugStore => feature_ecx & (1 << 4),
		Feature::Vmx => feature_ecx & (1 << 5),
		Feature::SaferMode => feature_ecx & (1 << 6),
		Feature::EnhancedSpeedStep => feature_ecx & (1 << 7),
		Feature::ThermalMonitor2 => feature_ecx & (1 << 8),
		Feature::Ssse3 => feature_ecx & (1 << 9),
		Feature::L1ContextId => feature_ecx & (1 << 10),
		Feature::Fma => feature_ecx & (1 << 12),
		Feature::Cx16 => feature_ecx & (1 << 13),
		Feature::Xtpr => feature_ecx & (1 << 14),
		Feature::PerformanceMonitor => feature_ecx & (1 << 15),
		Feature::ProcessContextId => feature_ecx & (1 << 17),
		Feature::DirectCache => feature_ecx & (1 << 18),
		Feature::Sse41 => feature_ecx & (1 << 19),
		Feature::Sse42 => feature_ecx & (1 << 20),
		Feature::X2Apic => feature_ecx & (1 << 21),
		Feature::MovBe => feature_ecx & (1 << 22),
		Feature::PopulationCount => feature_ecx & (1 << 23),
		Feature::TscDeadline => feature_ecx & (1 << 24),
		Feature::AesNi => feature_ecx & (1 << 25),
		Feature::XSave => feature_ecx & (1 << 26),
		Feature::OsXSave => feature_ecx & (1 << 27),
		Feature::Avx => feature_ecx & (1 << 28),
		Feature::HalfPrecision => feature_ecx & (1 << 29),
		Feature::HwRandom => feature_ecx & (1 << 30)
	}
}

#[inline(always)]
pub unsafe fn read_msr(msr: Msr) -> u64 {
	let (r1, r2): (u32, u32);
	asm!("rdmsr" : "={eax}"(r1), "={edx}"(r2) : "{ecx}"(msr as u32) :: "intel");
	r1 as u64 | ((r2 as u64) << 32)
}

#[derive(Copy)]
#[repr(C, packed)]
pub struct SegmentSelector {
	data: u16
}

impl SegmentSelector {
	#[inline(always)]
	pub fn new(index: u16, rpl: u8) -> SegmentSelector {
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
pub unsafe fn in8(port: u16) -> u8 { // unsafe since devices change state upon being read
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
