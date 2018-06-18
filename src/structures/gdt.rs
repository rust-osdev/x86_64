//! Types for the Global Descriptor Table and segment selectors.

use bit_field::BitField;
use core::fmt;
use structures::tss::TaskStateSegment;
use PrivilegeLevel;

/// Specifies which element to load into a segment from
/// descriptor tables (i.e., is a index to LDT or GDT table
/// with some additional flags).
///
/// See Intel 3a, Section 3.4.2 "Segment Selectors"
#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct SegmentSelector(pub u16);

impl SegmentSelector {
    /// Creates a new SegmentSelector
    ///
    /// # Arguments
    ///  * `index`: index in GDT or LDT array (not the offset)
    ///  * `rpl`: the requested privilege level
    pub const fn new(index: u16, rpl: PrivilegeLevel) -> SegmentSelector {
        SegmentSelector(index << 3 | (rpl as u16))
    }

    /// Returns the GDT index.
    pub fn index(&self) -> u16 {
        self.0 >> 3
    }

    /// Returns the requested privilege level.
    pub fn rpl(&self) -> PrivilegeLevel {
        PrivilegeLevel::from_u16(self.0.get_bits(0..2))
    }
}

impl fmt::Debug for SegmentSelector {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut s = f.debug_struct("SegmentSelector");
        s.field("index", &self.index());
        s.field("rpl", &self.rpl());
        s.finish()
    }
}

/// A 64-bit mode global descriptor table (GDT).
///
/// In 64-bit mode, segmentation is not supported. The GDT is used nonetheless, for example for
/// switching between user and kernel mode or for loading a TSS.
///
/// The GDT has a fixed size of 8 entries, trying to add more entries will panic.
pub struct GlobalDescriptorTable {
    table: [u64; 8],
    next_free: usize,
}

impl GlobalDescriptorTable {
    /// Creates an empty GDT.
    pub fn new() -> GlobalDescriptorTable {
        GlobalDescriptorTable {
            table: [0; 8],
            next_free: 1,
        }
    }

    /// Adds the given segment descriptor to the GDT, returning the segment selector.
    ///
    /// Panics if the GDT has no free entries left.
    pub fn add_entry(&mut self, entry: Descriptor) -> SegmentSelector {
        let index = match entry {
            Descriptor::UserSegment(value) => self.push(value),
            Descriptor::SystemSegment(value_low, value_high) => {
                let index = self.push(value_low);
                self.push(value_high);
                index
            }
        };
        SegmentSelector::new(index as u16, PrivilegeLevel::Ring0)
    }

    /// Loads the GDT in the CPU using the `lgdt` instruction.
    pub fn load(&'static self) {
        use core::mem::size_of;
        use instructions::tables::{lgdt, DescriptorTablePointer};

        let ptr = DescriptorTablePointer {
            base: self.table.as_ptr() as u64,
            limit: (self.table.len() * size_of::<u64>() - 1) as u16,
        };

        unsafe { lgdt(&ptr) };
    }

    fn push(&mut self, value: u64) -> usize {
        if self.next_free < self.table.len() {
            let index = self.next_free;
            self.table[index] = value;
            self.next_free += 1;
            index
        } else {
            panic!("GDT full");
        }
    }
}

/// A 64-bit mode segment descriptor.
///
/// Segmentation is no longer supported in 64-bit mode, so most of the descriptor
/// contents are ignored.
pub enum Descriptor {
    /// Descriptor for a code or data segment.
    ///
    /// Since segmentation is no longer supported in 64-bit mode, almost all of
    /// code and data descriptors is ignored. Only some flags are still used.
    UserSegment(u64),
    /// A system segment descriptor such as a LDT or TSS descriptor.
    SystemSegment(u64, u64),
}

bitflags! {
    /// Flags for a GDT descriptor. Not all flags are valid for all descriptor types.
    pub struct DescriptorFlags: u64 {
        /// Marks a code segment as “conforming”. This influences the privilege checks that
        /// occur on control transfers.
        const CONFORMING        = 1 << 42;
        /// This flag must be set for code segments.
        const EXECUTABLE        = 1 << 43;
        /// This flag must be set for user segments (in contrast to system segments).
        const USER_SEGMENT      = 1 << 44;
        /// Must be set for any segment, causes a segment not present exception if not set.
        const PRESENT           = 1 << 47;
        /// Must be set for long mode code segments.
        const LONG_MODE         = 1 << 53;
    }
}

impl Descriptor {
    /// Creates a segment descriptor for a long mode kernel code segment.
    pub fn kernel_code_segment() -> Descriptor {
        use self::DescriptorFlags as Flags;

        let flags = Flags::USER_SEGMENT | Flags::PRESENT | Flags::EXECUTABLE | Flags::LONG_MODE;
        Descriptor::UserSegment(flags.bits())
    }

    /// Creates a TSS system descriptor for the given TSS.
    pub fn tss_segment(tss: &'static TaskStateSegment) -> Descriptor {
        use self::DescriptorFlags as Flags;
        use bit_field::BitField;
        use core::mem::size_of;

        let ptr = tss as *const _ as u64;

        let mut low = Flags::PRESENT.bits();
        // base
        low.set_bits(16..40, ptr.get_bits(0..24));
        low.set_bits(56..64, ptr.get_bits(24..32));
        // limit (the `-1` in needed since the bound is inclusive)
        low.set_bits(0..16, (size_of::<TaskStateSegment>() - 1) as u64);
        // type (0b1001 = available 64-bit tss)
        low.set_bits(40..44, 0b1001);

        let mut high = 0;
        high.set_bits(0..32, ptr.get_bits(32..64));

        Descriptor::SystemSegment(low, high)
    }
}
