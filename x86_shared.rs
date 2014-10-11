pub use self::feature::Feature;
pub use self::exception::Exception;
pub use self::msr::Msr;

pub mod feature {
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
}

pub mod exception {
	#[deriving(FromPrimitive)]
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
}

pub mod msr {
	pub enum Msr {
		ApicBase = 0x1B
	}
}

#[inline(always)]
pub fn cpuid(function: u32) -> (u32, u32, u32, u32) {
	unsafe {
		let (eax, ebx, ecx, edx): (u32, u32, u32, u32);
		asm!("cpuid" : "={eax}"(eax), "={ebx}"(ebx), "={ecx}"(ecx), "={edx}"(edx) : "{eax}"(function))
		(eax, ebx, ecx, edx)
	}
}

#[inline(always)]
pub fn supports(feature: Feature) -> bool {
	let (_, _, feature_ecx, feature_edx) = cpuid(1);
	match feature { // this is bad
		feature::Fpu => feature_edx & (1 << 0) > 0,
		feature::Virtual8086 => feature_edx & (1 << 1) > 0,
		feature::DebugExtension => feature_edx & (1 << 2) > 0,
		feature::PageSizeExtension => feature_edx & (1 << 3) > 0,
		feature::TimeStampCounter => feature_edx & (1 << 4) > 0,
		feature::ModelSpecificRegister => feature_edx & (1 << 5) > 0,
		feature::PhysicalAddressExtension => feature_edx & (1 << 6) > 0,
		feature::MachineCheckException => feature_edx & (1 << 7) > 0,
		feature::Cx8 => feature_edx & (1 << 8) > 0,
		feature::Apic => feature_edx & (1 << 9) > 0,
		feature::SysEnter => feature_edx & (1 << 11) > 0,
		feature::MemoryTypeRange => feature_edx & (1 << 12) > 0,
		feature::PageGlobal => feature_edx & (1 << 13) > 0,
		feature::MachineCheckArchitecture => feature_edx & (1 << 14) > 0,
		feature::CMov => feature_edx & (1 << 15) > 0,
		feature::PageAttributeTable => feature_edx & (1 << 16) > 0,
		feature::PageSizeExtension36 => feature_edx & (1 << 17) > 0,
		feature::ProcessorSerial => feature_edx & (1 << 18) > 0,
		feature::CacheFlush => feature_edx & (1 << 19) > 0,
		feature::DebugStore => feature_edx & (1 << 21) > 0,
		feature::Acpi => feature_edx & (1 << 22) > 0,
		feature::Mmx => feature_edx & (1 << 23) > 0,
		feature::FxSave => feature_edx & (1 << 24) > 0,
		feature::Sse => feature_edx & (1 << 25) > 0,
		feature::Sse2 => feature_edx & (1 << 26) > 0,
		feature::SelfSnoop => feature_edx & (1 << 27) > 0,
		feature::HyperThreading => feature_edx & (1 << 28) > 0,
		feature::ThermalMonitor => feature_edx & (1 << 29) > 0,
		feature::Ia64 => feature_edx & (1 << 30) > 0,
		feature::PendingBreak => feature_edx & (1 << 31) > 0,

		feature::Sse3 => feature_ecx & (1 << 0) > 0,
		feature::PclMulQdq => feature_ecx & (1 << 1) > 0,
		feature::DebugStore64 => feature_ecx & (1 << 2) > 0,
		feature::Monitor => feature_ecx & (1 << 3) > 0,
		feature::CplDebugStore => feature_ecx & (1 << 4) > 0,
		feature::Vmx => feature_ecx & (1 << 5) > 0,
		feature::SaferMode => feature_ecx & (1 << 6) > 0,
		feature::EnhancedSpeedStep => feature_ecx & (1 << 7) > 0,
		feature::ThermalMonitor2 => feature_ecx & (1 << 8) > 0,
		feature::Ssse3 => feature_ecx & (1 << 9) > 0,
		feature::L1ContextId => feature_ecx & (1 << 10) > 0,
		feature::Fma => feature_ecx & (1 << 12) > 0,
		feature::Cx16 => feature_ecx & (1 << 13) > 0,
		feature::Xtpr => feature_ecx & (1 << 14) > 0,
		feature::PerformanceMonitor => feature_ecx & (1 << 15) > 0,
		feature::ProcessContextId => feature_ecx & (1 << 17) > 0,
		feature::DirectCache => feature_ecx & (1 << 18) > 0,
		feature::Sse41 => feature_ecx & (1 << 19) > 0,
		feature::Sse42 => feature_ecx & (1 << 20) > 0,
		feature::X2Apic => feature_ecx & (1 << 21) > 0,
		feature::MovBe => feature_ecx & (1 << 22) > 0,
		feature::PopulationCount => feature_ecx & (1 << 23) > 0,
		feature::TscDeadline => feature_ecx & (1 << 24) > 0,
		feature::AesNi => feature_ecx & (1 << 25) > 0,
		feature::XSave => feature_ecx & (1 << 26) > 0,
		feature::OsXSave => feature_ecx & (1 << 27) > 0,
		feature::Avx => feature_ecx & (1 << 28) > 0,
		feature::HalfPrecision => feature_ecx & (1 << 29) > 0,
		feature::HwRandom => feature_ecx & (1 << 30) > 0
	}
}

#[inline(always)]
pub unsafe fn read_msr(msr: Msr) -> u64 {
	let (r1, r2): (u32, u32);
	asm!("rdmsr" : "={eax}"(r1), "={edx}"(r2) : "{ecx}"(msr as u32) :: "intel")
	r1 as u64 | (r2 as u64 << 32)
}

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
	asm!("ltr $0" :: "r"(selector.bits()) :: "volatile", "intel")
}

#[inline(always)]
pub unsafe fn set_ss(selector: SegmentSelector) {
	asm!("mov ss, $0" :: "r"(selector.bits()) :: "volatile", "intel")
}

#[inline(always)]
pub unsafe fn set_ds(selector: SegmentSelector) {
	asm!("mov ds, $0" :: "r"(selector.bits()) :: "volatile", "intel")
}

#[inline(always)]
pub unsafe fn set_es(selector: SegmentSelector) {
	asm!("mov es, $0" :: "r"(selector.bits()) :: "volatile", "intel")
}

#[inline(always)]
pub unsafe fn set_gs(selector: SegmentSelector) {
	asm!("mov gs, $0" :: "r"(selector.bits()) :: "volatile", "intel")
}

#[inline(always)]
pub unsafe fn set_fs(selector: SegmentSelector) {
	asm!("mov fs, $0" :: "r"(selector.bits()) :: "volatile", "intel")
}

#[inline(always)]
pub unsafe fn enable_interrupts() {
	asm!("sti" :::: "volatile", "intel")
}

#[inline(always)]
pub unsafe fn disable_interrupts() {
	asm!("cli" :::: "volatile", "intel")
}

#[inline(always)]
pub unsafe fn halt() {
	asm!("hlt" :::: "volatile", "intel")
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
