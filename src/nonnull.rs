use std::mem;

#[repr(C)]
pub struct NonNull<T: ?Sized> {
    pointer: *const T,
}

impl<T> NonNull<T> {
    #[inline]
    const unsafe fn new_unchecked(ptr: *mut T) -> Self {
        // SAFETY: the caller must guarantee that `ptr` is non-null.
        // unsafe {
        // assert_unsafe_precondition!("NonNull::new_unchecked requires that the pointer is non-null", [T: ?Sized](ptr: *mut T) => !ptr.is_null());
        NonNull { pointer: ptr as _ }
        // }
    }

    #[inline]
    pub fn new(ptr: *mut T) -> Option<Self> {
        if !ptr.is_null() {
            // SAFETY: The pointer is already checked and is not null
            Some(unsafe { Self::new_unchecked(ptr) })
        } else {
            None
        }
    }

    #[inline]
    pub const fn dangling() -> Self {
        // SAFETY: mem::align_of() returns a non-zero usize which is then casted
        // to a *mut T. Therefore, `ptr` is not null and the conditions for
        // calling new_unchecked() are respected.
        unsafe {
            let ptr = sptr::invalid_mut::<T>(mem::align_of::<T>());
            NonNull::new_unchecked(ptr)
        }
    }

    #[inline(always)]
    pub const fn as_ptr(self) -> *mut T {
        self.pointer as *mut T
    }

    #[allow(dead_code)]
    #[must_use]
    #[inline(always)]
    pub unsafe fn as_ref<'a>(&self) -> &'a T {
        // SAFETY: the caller must guarantee that `self` meets all the
        // requirements for a reference.
        unsafe { &*self.as_ptr() }
    }
}

impl<T: ?Sized> Clone for NonNull<T> {
    #[inline(always)]
    fn clone(&self) -> Self {
        *self
    }
}

impl<T: ?Sized> Copy for NonNull<T> {}
