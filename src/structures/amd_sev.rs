use core::sync::atomic::AtomicBool;
use core::sync::atomic::Ordering::SeqCst;

use bit_field::BitField;

use crate::registers::model_specific::Msr;
use crate::structures::paging;
use crate::structures::paging::PageTableFlags;

const MSR_AMD_SEV: Msr = Msr::new(0xc0010131);

const SEV_INITIALIZED: AtomicBool = AtomicBool::new(false);


/// Retrieves the AMD SEV status, after calling `init`
pub fn sev_state<'a>() -> Option<&'a SevState> {
	AMD_SEV_STATUS.as_ref()
}

/// Initializes AMD Secure Encrypted Virtualization, if enabled.
///
/// It is required to call this function in an SEV enabled guest, as it sets-up a dynamic page table
/// flag for memory encryption. If this is not set, page table operations may cause panics due to
/// invalid canonical addresses.
pub fn init<'a>() -> Option<&'a SevState> {
	if SEV_INITIALIZED.fetch_or(true, SeqCst) {
		return sev_state();
	}

	// https://github.com/torvalds/linux/blob/900241a5cc15e6e0709a012051cc72d224cd6a6e/arch/x86/mm/mem_encrypt_identity.c#L566
	// Check for SME support
	let (cpuid_max, _) = unsafe { core::arch::x86_64::__get_cpuid_max(0x8000_0000) };
	if cpuid_max < 0x8000_001F {
		return None;
	}

	// Check if SME is available on the current CPU then read the C-Bit position and define the mask
	let sme_features = unsafe { core::arch::x86_64::__cpuid(0x8000_001F) };
	let sme_available = (sme_features.eax & 0x3) != 0; // either SME or SEV bit is set
	if !sme_available {
		return None;
	}

	let cbit_mask = (1u64) << sme_features.ebx.get_bits(0..=5) as u16;

	// Check the MSR to see if SME is currently enabled
	let sme_status = unsafe { MSR_AMD_SEV.read() };
	let sev_enabled = (sme_status & 0x1) == 1;
	let sev_es_enabled = (sme_status & 0x2) == 0x2;
	let snp_enabled = (sme_status & 0x4) == 0x4;

	if !sev_enabled && !snp_enabled {
		return None;
	}

	let state = Some(SevState {
		sev_enabled,
		snp_enabled,
		sev_es_enabled,
		c_bit_flag: PageTableFlags::from_bits_retain(cbit_mask),
	});

	// Update the memory mask
	unsafe {
		assign_static_immutable(&paging::page_table::PHYSICAL_ADDRESS_MASK, |pa| {
			*pa &= !cbit_mask
		});
		assign_static_immutable(&AMD_SEV_STATUS, |opt| *opt = state);
	}

	sev_state()
}

unsafe fn assign_static_immutable<T, F>(attr: &'static T, op: F)
where
	F: FnOnce(&mut T),
{
	let ptr = attr as *const T;
	unsafe {
		let ptr = ptr as u64;
		let ptr = ptr as *mut T;
		op(ptr.as_mut().unwrap());
	}
}

static AMD_SEV_STATUS: Option<SevState> = None;


/// Represents the current enablement state of AMD Secure Encrypted Virtualization features
#[derive(Copy, Clone, Debug)]
pub struct SevState {
	/// True if Secure Encrypted Virtualization is enabled
	pub sev_enabled: bool,

	/// True if Secure Nested Paging is enabled.
	///
	/// Implies `sev_es_enabled` = true and `sev_enabled` = true
	pub snp_enabled: bool,

	/// True if Encrypted State is enabled.
	///
	/// Implies `sev_enabled` = true
	pub sev_es_enabled: bool,

	/// Custom flag used to set the encryption bit in page table entries
	pub c_bit_flag: PageTableFlags,
}

impl PageTableFlags {
	#[inline]
	fn c_bit_mask() -> PageTableFlags {
		sev_state().map(|s| s.c_bit_flag)
			.expect("fatal: memory encryption is not enabled")
	}

	/// Sets the encryption bit on the page table entry.
	///
	/// Requires memory encryption to be enabled, or this will panic.
	pub fn set_encrypted(&mut self, encrypted: bool) {
		self.set(Self::c_bit_mask(), encrypted);
	}

	/// Checks if the encryption bit is set on the page table entry.
	///
	/// Requires memory encryption to be enabled, or this will panic.
	pub fn is_encrypted(&self) -> bool {
		self.contains(Self::c_bit_mask())
	}
}
