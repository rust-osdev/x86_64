#![cfg(target_arch = "x86_64")]

//! Special x86_64 instructions.

pub mod interrupts;
pub mod port;
pub mod segmentation;
pub mod tables;
pub mod tlb;

/// Cause a breakpoint exception by invoking the `int3` instruction.
pub fn int3() {
    unsafe {
        asm!("int3" :::: "volatile");
    }
}

/// Halts the CPU until the next interrupt arrives.
#[inline(always)]
pub fn hlt() {
    unsafe {
        asm!("hlt" :::: "volatile");
    }
}

/// Uniformly sampled Some(u64) if RdRand opcode is supported, None otherwise
#[inline]
pub fn rdrand() -> Option<u64> {
	let cpuid = raw_cpuid::CpuId::new();

	let has_rdrand = match cpuid.get_feature_info() {
		Some(finfo) => finfo.has_rdrand(),
		None => false
	};

	if has_rdrand {
		let res: u64;

		unsafe {
			asm!("rdrand %rax" : "={rax}"(res) ::: "volatile");
		}

		Some(res)
	} else {
		None
	}
}
