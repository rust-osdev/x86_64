use super::*;
use core::{cmp::Ordering, fmt, ops::Range};

#[cfg(not(target_arch = "x86_64"))]
use core::marker::PhantomData;

#[cfg(target_arch = "x86_64")]
type PtrValue<T> = *mut T;
#[cfg(not(target_arch = "x86_64"))]
type PtrValue<T> = (u64, PhantomData<*mut T>);

/// A Virtual Address representing a pointer to a type of T.
pub struct VirtPtr<T>(PtrValue<T>);

impl<T> VirtPtr<T> {
    /// Create a Virtual Address pointer from a plain [`VirtAddr`].
    pub const fn new(addr: VirtAddr) -> Self {
        // TODO: Use ptr::from_exposed_addr() when it's stable
        #[cfg(target_arch = "x86_64")]
        let v = addr.0 as *mut T;
        #[cfg(not(target_arch = "x86_64"))]
        let v = (addr.0, PhantomData);
        Self(v)
    }
    /// Create a null Virtual Address pointer.
    pub const fn null() -> Self {
        Self::new(VirtAddr::zero())
    }
    /// Return the underlying [`VirtAddr`] for this pointer.
    pub fn addr(self) -> VirtAddr {
        // TODO: Use ptr::expose_addr() when it's stable
        #[cfg(target_arch = "x86_64")]
        let a = self.0 as u64;
        #[cfg(not(target_arch = "x86_64"))]
        let a = self.0 .0;
        VirtAddr::new(a)
    }
}

#[cfg(target_arch = "x86_64")]
impl<T> VirtPtr<T> {
    /// Tries to create a new `VirtPtr` from a normal pointer.
    ///
    /// Returns an error if the pointer is not a valid virtual address.
    #[allow(clippy::not_unsafe_ptr_arg_deref)]
    pub fn from_ptr(ptr: *const T) -> Result<Self, VirtAddrNotValid> {
        // TODO: Use ptr::expose_addr() when it's stable
        VirtAddr::try_new(ptr as u64)?;
        Ok(unsafe { Self::from_ptr_unchecked(ptr) })
    }
    /// Create a `VirtPtr` from a shared reference.
    ///
    /// This can be safe and infallible as references are required to be in-bounds.
    pub const fn from_ref(r: &T) -> Self {
        unsafe { Self::from_ptr_unchecked(r) }
    }
    /// Create a `VirtPtr` range from a slice
    ///
    /// This can be safe and infallible as references are required to be in-bounds.
    pub const fn from_slice(r: &[T]) -> Range<Self> {
        let ptrs = r.as_ptr_range();
        Range {
            start: unsafe { Self::from_ptr_unchecked(ptrs.start) },
            end: unsafe { Self::from_ptr_unchecked(ptrs.end) },
        }
    }
    /// Create a new `VirtPtr` from a normal pointer.
    ///
    /// # Safety
    ///
    /// The provided pointer must be a valid virtual address.
    pub const unsafe fn from_ptr_unchecked(ptr: *const T) -> Self {
        Self(ptr as *mut T)
    }
    /// Convert into a normal pointer.
    pub const fn as_ptr(self) -> *mut T {
        self.0
    }
}

// We have to write these impls (instead of deriving them) to avoid a trait
// bound on T.
impl<T> Copy for VirtPtr<T> {}
impl<T> Clone for VirtPtr<T> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<T> PartialEq for VirtPtr<T> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}
impl<T> Eq for VirtPtr<T> {}
impl<T> PartialOrd for VirtPtr<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.0.partial_cmp(&other.0)
    }
}
impl<T> Ord for VirtPtr<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.cmp(&other.0)
    }
}

impl<T> fmt::Debug for VirtPtr<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("VirtPtr")
            .field(&format_args!("{:#018x}", self.addr()))
            .finish()
    }
}

unsafe impl<T> Send for VirtPtr<T> {}
unsafe impl<T> Sync for VirtPtr<T> {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn check_null() {
        assert_eq!(VirtPtr::<u8>(0 as *mut _), VirtPtr::<u8>(1 as *mut _));
    }
}
