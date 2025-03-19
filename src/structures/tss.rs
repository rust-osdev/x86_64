//! Provides a type for the task state segment structure.

use crate::VirtAddr;
use core::{
    fmt::{self, Display},
    mem::size_of,
};

/// In 64-bit mode the TSS holds information that is not
/// directly related to the task-switch mechanism,
/// but is used for stack switching when an interrupt or exception occurs.
#[derive(Debug, Clone, Copy)]
#[repr(C, packed(4))]
pub struct TaskStateSegment {
    reserved_1: u32,
    /// The full 64-bit canonical forms of the stack pointers (RSP) for privilege levels 0-2.
    /// The stack pointers used when a privilege level change occurs from a lower privilege level to a higher one.
    pub privilege_stack_table: [VirtAddr; 3],
    reserved_2: u64,
    /// The full 64-bit canonical forms of the interrupt stack table (IST) pointers.
    /// The stack pointers used when an entry in the Interrupt Descriptor Table has an IST value other than 0.
    pub interrupt_stack_table: [VirtAddr; 7],
    reserved_3: u64,
    reserved_4: u16,
    /// The 16-bit offset to the I/O permission bit map from the 64-bit TSS base. It must not
    /// exceed `0xDFFF`.
    pub iomap_base: u16,
}

impl TaskStateSegment {
    /// Creates a new TSS with zeroed privilege and interrupt stack table and an
    /// empty I/O-Permission Bitmap.
    ///
    /// As we always set the TSS segment limit to
    /// `size_of::<TaskStateSegment>() - 1`, this means that `iomap_base` is
    /// initialized to `size_of::<TaskStateSegment>()`.
    #[inline]
    pub const fn new() -> TaskStateSegment {
        TaskStateSegment {
            privilege_stack_table: [VirtAddr::zero(); 3],
            interrupt_stack_table: [VirtAddr::zero(); 7],
            iomap_base: size_of::<TaskStateSegment>() as u16,
            reserved_1: 0,
            reserved_2: 0,
            reserved_3: 0,
            reserved_4: 0,
        }
    }
}

impl Default for TaskStateSegment {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

/// The given IO permissions bitmap is invalid.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum InvalidIoMap {
    /// The IO permissions bitmap is before the TSS. It must be located after the TSS.
    IoMapBeforeTss,
    /// The IO permissions bitmap is too far from the TSS. It must be within `0xdfff` bytes of the
    /// start of the TSS. Note that if the IO permissions bitmap is located before the TSS, then
    /// `IoMapBeforeTss` will be returned instead.
    TooFarFromTss {
        /// The distance of the IO permissions bitmap from the beginning of the TSS.
        distance: usize,
    },
    /// The final byte of the IO permissions bitmap was not 0xff
    InvalidTerminatingByte {
        /// The byte found at the end of the IO permissions bitmap.
        byte: u8,
    },
    /// The IO permissions bitmap exceeds the maximum length (8193).
    TooLong {
        /// The length of the IO permissions bitmap.
        len: usize,
    },
    /// The `iomap_base` in the `TaskStateSegment` struct was not what was expected.
    InvalidBase {
        /// The expected `iomap_base` to be set in the `TaskStateSegment` struct.
        expected: u16,
        /// The actual `iomap_base` set in the `TaskStateSegment` struct.
        got: u16,
    },
}

impl Display for InvalidIoMap {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            InvalidIoMap::IoMapBeforeTss => {
                write!(f, "the IO permissions bitmap is before the TSS")
            }
            InvalidIoMap::TooFarFromTss { distance } => write!(
                f,
                "the IO permissions bitmap is too far from the TSS (distance {distance})"
            ),
            InvalidIoMap::InvalidTerminatingByte { byte } => write!(
                f,
                "The final byte of the IO permissions bitmap was not 0xff ({byte}"
            ),
            InvalidIoMap::TooLong { len } => {
                write!(
                    f,
                    "The IO permissions bitmap exceeds the maximum length ({len} > 8193)"
                )
            }
            InvalidIoMap::InvalidBase { expected, got } => write!(
                f,
                "the `iomap_base` in the `TaskStateSegment` struct was not what was expected (expected {expected}, got {got})"
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn check_tss_size() {
        // Per the SDM, the minimum size of a TSS is 0x68 bytes, giving a
        // minimum limit of 0x67.
        assert_eq!(size_of::<TaskStateSegment>(), 0x68);
    }
}
