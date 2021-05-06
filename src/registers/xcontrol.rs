//! Access to various extended system registers
use bitflags::bitflags;

/// Extended feature enable mask register
#[derive(Debug)]
pub struct XCr0;

bitflags! {
    /// Configuration flags of the XCr0 register.
    pub struct XCr0Flags: u64 {
        /// Enables x87 FPU
        const X87 = 1;
        /// Enables 128-bit (legacy) SSE
        /// Must be set to enable AVX and YMM
        const SSE = 1<<1;
        /// Enables 256-bit SSE
        /// Must be set to enable AVX
        const YMM = 1<<2;
        /// When set, PKRU state management is supported by
        /// ZSAVE/XRSTOR
        const MPK = 1<<9;
        /// When set the Lightweight Profiling extensions are enabled
        const LWP = 1<<62;
    }
}

#[cfg(all(feature = "instructions", feature = "inline_asm"))]
mod x86_64 {
    use super::*;
    impl XCr0 {
        /// Read the current set of XCR0 flags.
        #[inline]
        pub fn read() -> XCr0Flags {
            XCr0Flags::from_bits_truncate(Self::read_raw())
        }

        /// Read the current raw XCR0 value.
        #[inline]
        pub fn read_raw() -> u64 {
            let (low, high): (u32, u32);
            unsafe {
                asm!(
                    "xgetbv",
                    in("ecx") 0,
                    out("rax") low, out("rdx") high,
                    options(nomem, nostack, preserves_flags),
                );
            }
            (high as u64) << 32 | (low as u64)
        }

        /// Write XCR0 flags.
        ///
        /// Preserves the value of reserved fields.
        ///
        /// ## Safety
        ///
        /// This function is unsafe because it's possible to
        /// enable features that are not supported by the architecture
        #[inline]
        pub unsafe fn write(flags: XCr0Flags) {
            let old_value = Self::read_raw();
            let reserved = old_value & !(XCr0Flags::all().bits());
            let new_value = reserved | flags.bits();

            Self::write_raw(new_value);
        }

        /// Write raw XCR0 flags.
        ///
        /// Does _not_ preserve any values, including reserved fields.
        ///
        /// ## Safety
        ///
        /// This function is unsafe because it's possible to
        /// enable features that are not supported by the architecture
        #[inline]
        pub unsafe fn write_raw(value: u64) {
            let low = value as u32;
            let high = (value >> 32) as u32;
            asm!(
                "xsetbv",
                in("ecx") 0,
                in("rax") low, in("rdx") high,
                options(nomem, nostack, preserves_flags),
            );
        }
    }
}
