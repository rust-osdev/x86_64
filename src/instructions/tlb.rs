//! Functions to flush the translation lookaside buffer (TLB).

use crate::VirtAddr;

/// Invalidate the given address in the TLB using the `invlpg` instruction.
#[inline]
pub fn flush(addr: VirtAddr) {
    #[cfg(feature = "inline_asm")]
    unsafe {
        llvm_asm!("invlpg ($0)" :: "r" (addr.as_u64()) : "memory")
    };

    #[cfg(not(feature = "inline_asm"))]
    unsafe {
        crate::asm::x86_64_asm_invlpg(addr.as_u64())
    };
}

/// Invalidate the TLB completely by reloading the CR3 register.
#[inline]
pub fn flush_all() {
    use crate::registers::control::Cr3;
    let (frame, flags) = Cr3::read();
    unsafe { Cr3::write(frame, flags) }
}

/// The Invalidate PCID Command to execute.
#[derive(Debug)]
pub enum InvPicdCommand {
    /// The logical processor invalidates mappings—except global translations—for the linear address and PCID specified.
    IndividualAddressInvalidation(VirtAddr, Pcid),

    /// The logical processor invalidates all mappings—except global translations—associated with the PCID.
    SingleContextInvalidation(Pcid),

    /// The logical processor invalidates all mappings—including global translations—associated with any PCID.
    AllContextInvalidationIncludeGlobal,

    /// The logical processor invalidates all mappings—except global translations—associated with any PCID.
    AllContextInvalidationExcludeGlobal,
}

/// The INVPCID descriptor comprises 128 bits and consists of a PCID and a linear address.
/// For INVPCID type 0, the processor uses the full 64 bits of the linear address even outside 64-bit mode; the linear address is not used for other INVPCID types.
#[repr(u128)]
#[derive(Debug)]
struct InvpcidDescriptor {
    address: u64,
    pcid: u64,
}

/// Structure of a PCID. A PCID has to be <= 4096 for x86_64.
#[repr(transparent)]
#[derive(Debug)]
pub struct Pcid(u64);

impl Pcid {
    /// Create a new PCID. Will result in a failure if the value of
    /// PCID is out of expected bounds.
    pub fn new(pcid: u16) -> Result<Pcid, &'static str> {
        if pcid >= 4096 {
            Err("PCID should be < 4096.")
        } else {
            Ok(Pcid(pcid as u64))
        }
    }

    /// Get the value of the current PCID.
    pub fn value(&self) -> u16 {
        self.0 as u16
    }
}

/// Invalidate the given address in the TLB using the `invpcid` instruction.
#[inline]
pub fn flush_pcid(command: InvPicdCommand) {
    let mut desc = InvpcidDescriptor {
        address: 0,
        pcid: 0,
    };

    let kind;
    match command {
        InvPicdCommand::IndividualAddressInvalidation(addr, pcid) => {
            kind = 0;
            desc.pcid = pcid.value() as u64;
            desc.address = addr.as_u64()
        }
        InvPicdCommand::SingleContextInvalidation(pcid) => {
            kind = 1;
            desc.pcid = pcid.0
        }
        InvPicdCommand::AllContextInvalidationIncludeGlobal => kind = 2,
        InvPicdCommand::AllContextInvalidationExcludeGlobal => kind = 3,
    }

    #[cfg(feature = "inline_asm")]
    unsafe {
        llvm_asm!("invpcid ($0), $1" :: "r" (&desc as *const InvpcidDescriptor as u64), "r" (kind) : "memory")
    };

    #[cfg(not(feature = "inline_asm"))]
    unsafe {
        crate::asm::x86_64_asm_invpcid(kind, &desc as *const InvpcidDescriptor as u64)
    };
}
