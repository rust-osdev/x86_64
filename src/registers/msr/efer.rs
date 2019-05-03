pub use x86_64_types::registers::Efer;

const MSR: super::Msr = super::Msr(0xC0000080);

impl super::super::RegReader for Efer {
    fn read() -> Self {
        Efer::from_bits_truncate(MSR.read())
    }
}

impl super::super::RegWriter for Efer {
    unsafe fn write(self) {
        MSR.write(self.bits())
    }
}
