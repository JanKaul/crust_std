use std::marker::PhantomData;
use std::ops::Deref;
use std::ptr::drop_in_place;
use std::sync::atomic::{self, AtomicUsize, Ordering};

use crate::nonnull::NonNull;

#[repr(C)]
pub struct Arc<T> {
    ptr: NonNull<ArcInner<T>>,
    phantom: PhantomData<ArcInner<T>>,
}

#[repr(C)]
pub struct ArcInner<T> {
    rc: AtomicUsize,
    data: T,
}

impl<T> Arc<T> {
    pub fn new(data: T) -> Arc<T> {
        // We start the reference count at 1, as that first reference is the
        // current pointer.
        let boxed = Box::new(ArcInner {
            rc: AtomicUsize::new(1),
            data,
        });
        Arc {
            // It is okay to call `.unwrap()` here as we get a pointer from
            // `Box::into_raw` which is guaranteed to not be null.
            ptr: NonNull::new(Box::into_raw(boxed)).unwrap(),
            phantom: PhantomData,
        }
    }
}

unsafe impl<T: Sync + Send> Send for Arc<T> {}
unsafe impl<T: Sync + Send> Sync for Arc<T> {}

impl<T> Deref for Arc<T> {
    type Target = T;

    fn deref(&self) -> &T {
        let inner = unsafe { self.ptr.as_ref() };
        &inner.data
    }
}

impl<T> Clone for Arc<T> {
    fn clone(&self) -> Arc<T> {
        let inner = unsafe { self.ptr.as_ref() };
        // Using a relaxed ordering is alright here as we don't need any atomic
        // synchronization here as we're not modifying or accessing the inner
        // data.
        let old_rc = inner.rc.fetch_add(1, Ordering::Relaxed);

        if old_rc >= isize::MAX as usize {
            std::process::abort();
        }

        Self {
            ptr: self.ptr,
            phantom: PhantomData,
        }
    }
}

impl<T> Drop for Arc<T> {
    fn drop(&mut self) {
        let inner = unsafe { self.ptr.as_ref() };
        if inner.rc.fetch_sub(1, Ordering::Release) != 1 {
            return;
        }
        // This fence is needed to prevent reordering of the use and deletion
        // of the data.
        atomic::fence(Ordering::Acquire);
        // This is safe as we know we have the last pointer to the `ArcInner`
        // and that its pointer is valid.
        unsafe {
            drop(Box::from_raw(self.ptr.as_ptr()));
        }
    }
}

type Opaque<T> = Arc<T>;

#[no_mangle]
pub unsafe extern "C" fn crust_arc_get<Void>(arc: *const Opaque<Void>) -> *const Void {
    (&*arc).deref()
}

#[no_mangle]
pub unsafe extern "C" fn crust_arc_clone<Void>(arc: *const Opaque<Void>) -> Arc<Void> {
    (&*arc).clone()
}

#[no_mangle]
pub unsafe extern "C" fn crust_arc_free<Void>(arc: *mut Opaque<Void>) {
    drop_in_place(arc);
}