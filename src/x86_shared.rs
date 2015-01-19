#![allow(non_upper_case_globals)]

bitflags!(
	flags Features: u64 {
		static Fpu = 1 << 0,
		static Virtual8086 = 1 << 1,
		static DebugExtension = 1 << 2,
		static PageSizeExtension = 1 << 3,
		static TimeStampCounter = 1 << 4,
		static ModelSpecificRegister = 1 << 5,
		static PhysicalAddressExtension = 1 << 6,
		static MachineCheckException = 1 << 7,
		static Cx8 = 1 << 8, // CMPXCHG8
		static Apic = 1 << 9,
		static SysEnter = 1 << 11,
		static MemoryTypeRange = 1 << 12,
		static PageGlobal = 1 << 13,
		static MachineCheckArchitecture = 1 << 14,
		static CMov = 1 << 15,
		static PageAttributeTable = 1 << 16,
		static PageSizeExtension36 = 1 << 17,
		static ProcessorSerial = 1 << 18,
		static CacheFlush = 1 << 19,
		static DebugStore = 1 << 21,
		static Acpi = 1 << 22,
		static Mmx = 1 << 23,
		static FxSave = 1 << 24,
		static Sse = 1 << 25,
		static Sse2 = 1 << 26,
		static SelfSnoop = 1 << 27,
		static HyperThreading = 1 << 28,
		static ThermalMonitor = 1 << 29,
		static Ia64 = 1 << 30,
		static PendingBreak = 1 << 31,

		static Sse3 = 1 << (32 + 0),
		static PclMulQdq = 1 << (32 + 1), // what
		static DebugStore64 = 1 << (32 + 2),
		static Monitor = 1 << (32 + 3),
		static CplDebugStore = 1 << (32 + 4),
		static Vmx = 1 << (32 + 5),
		static SaferMode = 1 << (32 + 6),
		static EnhancedSpeedStep = 1 << (32 + 7),
		static ThermalMonitor2 = 1 << (32 + 8),
		static Ssse3 = 1 << (32 + 9),
		static L1ContextId = 1 << (32 + 10),
		static Fma = 1 << (32 + 12),
		static Cx16 = 1 << (32 + 13), // CMPXCHG16B
		static Xtpr = 1 << (32 + 14), // I have no idea what this is
		static PerformanceMonitor = 1 << (32 + 15),
		static ProcessContextId = 1 << (32 + 17),
		static DirectCache = 1 << (32 + 18),
		static Sse41 = 1 << (32 + 19),
		static Sse42 = 1 << (32 + 20),
		static X2Apic = 1 << (32 + 21),
		static MovBe = 1 << (32 + 22),
		static PopulationCount = 1 << (32 + 23),
		static TscDeadline = 1 << (32 + 24),
		static AesNi = 1 << (32 + 25),
		static XSave = 1 << (32 + 26),
		static OsXSave = 1 << (32 + 27),
		static Avx = 1 << (32 + 28),
		static HalfPrecision = 1 << (32 + 29),
		static HwRandom = 1 << (32 + 30)
	}
);

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
