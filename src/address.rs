use core::fmt;

/// Represents a physical memory address
#[derive(Copy, Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct PhysicalAddress(u64);

impl From<u64> for PhysicalAddress {
    fn from(value: u64) -> Self {
        PhysicalAddress(value)
    }
}

impl From<usize> for PhysicalAddress {
    fn from(value: usize) -> Self {
        PhysicalAddress(value as u64)
    }
}

impl PhysicalAddress {
    /// Convert to `u64`
    pub const fn as_u64(&self) -> u64 {
        self.0
    }
}

impl fmt::Binary for PhysicalAddress {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl fmt::Display for PhysicalAddress {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl fmt::LowerHex for PhysicalAddress {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl fmt::Octal for PhysicalAddress {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl fmt::UpperHex for PhysicalAddress {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

/// Represent a virtual (linear) memory address
#[derive(Copy, Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct VirtualAddress(usize);

impl From<usize> for VirtualAddress {
    fn from(value: usize) -> Self {
        VirtualAddress(value)
    }
}

impl<T> From<*const T> for VirtualAddress {
    fn from(pointer: *const T) -> Self {
        VirtualAddress(pointer as usize)
    }
}

impl<T> From<*mut T> for VirtualAddress {
    fn from(pointer: *mut T) -> Self {
        VirtualAddress(pointer as usize)
    }
}

impl<'a, T> From<&'a T> for VirtualAddress {
    fn from(pointer: &T) -> Self {
        VirtualAddress::from(pointer as *const _)
    }
}

impl<'a, T> From<&'a mut T> for VirtualAddress {
    fn from(pointer: &mut T) -> Self {
        VirtualAddress::from(pointer as *mut _)
    }
}


impl VirtualAddress {
    /// Convert to `usize`
    pub const fn as_usize(&self) -> usize {
        self.0
    }
}

impl fmt::Binary for VirtualAddress {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl fmt::Display for VirtualAddress {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl fmt::LowerHex for VirtualAddress {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl fmt::Octal for VirtualAddress {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl fmt::UpperHex for VirtualAddress {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}
