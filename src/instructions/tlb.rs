//! Functions to flush the translation lookaside buffer (TLB).

use bit_field::BitField;

use crate::{
    instructions::segmentation::{Segment, CS},
    structures::paging::{
        page::{NotGiantPageSize, PageRange},
        Page, PageSize, Size2MiB, Size4KiB,
    },
    PrivilegeLevel, VirtAddr,
};
use core::{arch::asm, cmp, convert::TryFrom, fmt};

/// Invalidate the given address in the TLB using the `invlpg` instruction.
#[inline]
pub fn flush(addr: VirtAddr) {
    unsafe {
        asm!("invlpg [{}]", in(reg) addr.as_u64(), options(nostack, preserves_flags));
    }
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
    Address(VirtAddr, Pcid),

    /// The logical processor invalidates all mappings—except global translations—associated with the PCID.
    Single(Pcid),

    /// The logical processor invalidates all mappings—including global translations—associated with any PCID.
    All,

    /// The logical processor invalidates all mappings—except global translations—associated with any PCID.
    AllExceptGlobal,
}

/// The INVPCID descriptor comprises 128 bits and consists of a PCID and a linear address.
/// For INVPCID type 0, the processor uses the full 64 bits of the linear address even outside 64-bit mode; the linear address is not used for other INVPCID types.
#[repr(C)]
#[derive(Debug)]
struct InvpcidDescriptor {
    address: u64,
    pcid: u64,
}

/// Structure of a PCID. A PCID has to be <= 4096 for x86_64.
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Pcid(u16);

impl Pcid {
    /// Create a new PCID. Will result in a failure if the value of
    /// PCID is out of expected bounds.
    pub const fn new(pcid: u16) -> Result<Pcid, &'static str> {
        if pcid >= 4096 {
            Err("PCID should be < 4096.")
        } else {
            Ok(Pcid(pcid))
        }
    }

    /// Get the value of the current PCID.
    pub const fn value(&self) -> u16 {
        self.0
    }
}

/// Invalidate the given address in the TLB using the `invpcid` instruction.
///
/// ## Safety
///
/// This function is unsafe as it requires CPUID.(EAX=07H, ECX=0H):EBX.INVPCID to be 1.
#[inline]
pub unsafe fn flush_pcid(command: InvPicdCommand) {
    let mut desc = InvpcidDescriptor {
        address: 0,
        pcid: 0,
    };

    let kind: u64;
    match command {
        InvPicdCommand::Address(addr, pcid) => {
            kind = 0;
            desc.pcid = pcid.value().into();
            desc.address = addr.as_u64()
        }
        InvPicdCommand::Single(pcid) => {
            kind = 1;
            desc.pcid = pcid.0.into()
        }
        InvPicdCommand::All => kind = 2,
        InvPicdCommand::AllExceptGlobal => kind = 3,
    }

    unsafe {
        asm!("invpcid {0}, [{1}]", in(reg) kind, in(reg) &desc, options(nostack, preserves_flags));
    }
}

/// Used to broadcast flushes to all logical processors.
///
/// ```no_run
/// use x86_64::VirtAddr;
/// use x86_64::structures::paging::Page;
/// use x86_64::instructions::tlb::Invlpgb;
///
/// // Check that `invlpgb` and `tlbsync` are supported.
/// let invlpgb = Invlpgb::new().unwrap();
///
/// // Broadcast flushing some pages to all logical processors.
/// let start: Page = Page::from_start_address(VirtAddr::new(0xf000_0000)).unwrap();
/// let pages = Page::range(start, start + 3);
/// invlpgb.build().pages(pages).include_global().flush();
///
/// // Wait for all logical processors to respond.
/// invlpgb.tlbsync();
/// ```
#[derive(Debug, Clone, Copy)]
pub struct Invlpgb {
    invlpgb_count_max: u16,
    tlb_flush_nested: bool,
    nasid: u32,
}

impl Invlpgb {
    /// Check that `invlpgb` and `tlbsync` are supported and query limits.
    ///
    /// # Panics
    ///
    /// Panics if the CPL is not 0.
    pub fn new() -> Option<Self> {
        let cs = CS::get_reg();
        assert_eq!(cs.rpl(), PrivilegeLevel::Ring0);

        // Check if the `INVLPGB` and `TLBSYNC` instruction are supported.
        let cpuid = unsafe { core::arch::x86_64::__cpuid(0x8000_0008) };
        if !cpuid.ebx.get_bit(3) {
            return None;
        }

        let tlb_flush_nested = cpuid.ebx.get_bit(21);
        let invlpgb_count_max = cpuid.edx.get_bits(0..=15) as u16;

        // Figure out the number of supported ASIDs.
        let cpuid = unsafe { core::arch::x86_64::__cpuid(0x8000_000a) };
        let nasid = cpuid.ebx;

        Some(Self {
            tlb_flush_nested,
            invlpgb_count_max,
            nasid,
        })
    }

    /// Returns the maximum count of pages to be flushed supported by the processor.
    #[inline]
    pub fn invlpgb_count_max(&self) -> u16 {
        self.invlpgb_count_max
    }

    /// Returns whether the processor supports flushing translations used for guest translation.
    #[inline]
    pub fn tlb_flush_nested(&self) -> bool {
        self.tlb_flush_nested
    }

    /// Returns the number of available address space identifiers.
    #[inline]
    pub fn nasid(&self) -> u32 {
        self.nasid
    }

    /// Create a `InvlpgbFlushBuilder`.
    pub fn build(&self) -> InvlpgbFlushBuilder<'_> {
        InvlpgbFlushBuilder {
            invlpgb: self,
            page_range: None,
            pcid: None,
            asid: None,
            include_global: false,
            final_translation_only: false,
            include_nested_translations: false,
        }
    }

    /// Wait for all previous `invlpgb` instruction executed on the current
    /// logical processor to be acknowledged by all other logical processors.
    #[inline]
    pub fn tlbsync(&self) {
        unsafe {
            asm!("tlbsync", options(nomem, preserves_flags));
        }
    }
}

/// A builder struct to construct the parameters for the `invlpgb` instruction.
#[derive(Debug, Clone)]
#[must_use]
pub struct InvlpgbFlushBuilder<'a, S = Size4KiB>
where
    S: NotGiantPageSize,
{
    invlpgb: &'a Invlpgb,
    page_range: Option<PageRange<S>>,
    pcid: Option<Pcid>,
    asid: Option<u16>,
    include_global: bool,
    final_translation_only: bool,
    include_nested_translations: bool,
}

impl<'a, S> InvlpgbFlushBuilder<'a, S>
where
    S: NotGiantPageSize,
{
    /// Flush a range of pages.
    ///
    /// If the range doesn't fit within `invlpgb_count_max`, `invlpgb` is
    /// executed multiple times.
    pub fn pages<T>(self, page_range: PageRange<T>) -> InvlpgbFlushBuilder<'a, T>
    where
        T: NotGiantPageSize,
    {
        InvlpgbFlushBuilder {
            invlpgb: self.invlpgb,
            page_range: Some(page_range),
            pcid: self.pcid,
            asid: self.asid,
            include_global: self.include_global,
            final_translation_only: self.final_translation_only,
            include_nested_translations: self.include_nested_translations,
        }
    }

    /// Only flush TLB entries with the given PCID.
    ///
    /// # Safety
    ///
    /// The caller has to ensure that PCID is enabled in CR4 when the flush is executed.
    pub unsafe fn pcid(&mut self, pcid: Pcid) -> &mut Self {
        self.pcid = Some(pcid);
        self
    }

    /// Only flush TLB entries with the given ASID.
    ///
    /// # Safety
    ///
    /// The caller has to ensure that SVM is enabled in EFER when the flush is executed.
    // FIXME: Make ASID a type and remove error type.
    pub unsafe fn asid(&mut self, asid: u16) -> Result<&mut Self, AsidOutOfRangeError> {
        if u32::from(asid) >= self.invlpgb.nasid {
            return Err(AsidOutOfRangeError {
                asid,
                nasid: self.invlpgb.nasid,
            });
        }

        self.asid = Some(asid);
        Ok(self)
    }

    /// Also flush global pages.
    pub fn include_global(&mut self) -> &mut Self {
        self.include_global = true;
        self
    }

    /// Only flush the final translation and not the cached upper level TLB entries.
    pub fn final_translation_only(&mut self) -> &mut Self {
        self.final_translation_only = true;
        self
    }

    /// Also flush nestred translations that could be used for guest translation.
    pub fn include_nested_translations(mut self) -> Self {
        assert!(
            self.invlpgb.tlb_flush_nested,
            "flushing all nested translations is not supported"
        );

        self.include_nested_translations = true;
        self
    }

    /// Execute the flush.
    pub fn flush(&self) {
        if let Some(mut pages) = self.page_range {
            while !pages.is_empty() {
                // Calculate out how many pages we still need to flush.
                let count = Page::<S>::steps_between_impl(&pages.start, &pages.end).unwrap();

                // Make sure that we never jump the gap in the address space when flushing.
                let second_half_start =
                    Page::<S>::containing_address(VirtAddr::new(0xffff_8000_0000_0000));
                let count = if pages.start < second_half_start {
                    let count_to_second_half =
                        Page::steps_between_impl(&pages.start, &second_half_start).unwrap();
                    cmp::min(count, count_to_second_half)
                } else {
                    count
                };

                // We can flush at most u16::MAX pages at once.
                let count = u16::try_from(count).unwrap_or(u16::MAX);

                // Cap the count by the maximum supported count of the processor.
                let count = cmp::min(count, self.invlpgb.invlpgb_count_max);

                unsafe {
                    flush_broadcast(
                        Some((pages.start, count)),
                        self.pcid,
                        self.asid,
                        self.include_global,
                        self.final_translation_only,
                        self.include_nested_translations,
                    );
                }

                // Even if the count is zero, one page is still flushed and so
                // we need to advance by at least one.
                let inc_count = cmp::max(count, 1);
                pages.start =
                    Page::forward_checked_impl(pages.start, usize::from(inc_count)).unwrap();
            }
        } else {
            unsafe {
                flush_broadcast::<S>(
                    None,
                    self.pcid,
                    self.asid,
                    self.include_global,
                    self.final_translation_only,
                    self.include_nested_translations,
                );
            }
        }
    }
}

/// An error returned when trying to use an invalid ASID.
#[derive(Debug)]
pub struct AsidOutOfRangeError {
    /// The requested ASID.
    pub asid: u16,
    /// The number of valid ASIDS.
    pub nasid: u32,
}

impl fmt::Display for AsidOutOfRangeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} is out of the range of available ASIDS ({})",
            self.asid, self.nasid
        )
    }
}

/// See `INVLPGB` in AMD64 Architecture Programmer's Manual Volume 3
#[inline]
unsafe fn flush_broadcast<S>(
    va_and_count: Option<(Page<S>, u16)>,
    pcid: Option<Pcid>,
    asid: Option<u16>,
    include_global: bool,
    final_translation_only: bool,
    include_nested_translations: bool,
) where
    S: NotGiantPageSize,
{
    let mut rax = 0;
    let mut ecx = 0;
    let mut edx = 0;

    if let Some((va, count)) = va_and_count {
        rax.set_bit(0, true);
        rax.set_bits(12.., va.start_address().as_u64().get_bits(12..));

        ecx.set_bits(0..=15, u32::from(count));
        ecx.set_bit(31, S::SIZE == Size2MiB::SIZE);
    }

    if let Some(pcid) = pcid {
        rax.set_bit(1, true);
        edx.set_bits(16..=27, u32::from(pcid.value()));
    }

    if let Some(asid) = asid {
        rax.set_bit(2, true);
        edx.set_bits(0..=15, u32::from(asid));
    }

    rax.set_bit(3, include_global);
    rax.set_bit(4, final_translation_only);
    rax.set_bit(5, include_nested_translations);

    unsafe {
        asm!(
            "invlpgb",
            in("rax") rax,
            in("ecx") ecx,
            in("edx") edx,
            options(nostack, preserves_flags),
        );
    }
}
