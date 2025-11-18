use std::ptr;
use std::sync::Arc;
use std::sync::atomic::{AtomicPtr, Ordering};

pub struct SmartAtomicPointer<T>
{
    inner: AtomicPtr<T>
}

impl<T> SmartAtomicPointer<T>
{
    pub fn new(p: *mut T) -> Self {
        return Self {
            inner: AtomicPtr::new(p),
        };
    }
}

// Delegations
impl<T> SmartAtomicPointer<T>
{

}

// Custom Drop implementation for freeing memory
impl<T> Drop for SmartAtomicPointer<T>
{
    fn drop(&mut self)
    {
        let raw = self.inner.swap(ptr::null_mut(), Ordering::AcqRel);
        if !raw.is_null() {
            unsafe {
                drop(Arc::from_raw(raw));
            }
        }
    }
}
