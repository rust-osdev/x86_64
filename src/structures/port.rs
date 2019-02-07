//! A trait for accessing I/O ports.

/// A helper trait that implements the actual port operations.
///
/// On x86, I/O ports operate on either `u8` (via `inb`/`outb`), `u16` (via `inw`/`outw`),
/// or `u32` (via `inl`/`outl`). Therefore this trait is implemented for exactly these types.
pub trait PortReadWrite {
    /// Reads a `Self` value from the given port.
    ///
    /// This function is unsafe because the I/O port could have side effects that violate memory
    /// safety.
    unsafe fn read_from_port(port: u16) -> Self;

    /// Writes a `Self` value to the given port.
    ///
    /// This function is unsafe because the I/O port could have side effects that violate memory
    /// safety.
    unsafe fn write_to_port(port: u16, value: Self);
}
