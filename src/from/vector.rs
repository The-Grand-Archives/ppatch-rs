use std::{marker::PhantomData, ops::Deref, ops::DerefMut};

use super::allocator::{DLAllocator, DLAllocatorProxy};

#[repr(C)]
#[derive(Debug)]
pub struct DLVector<T, A: DLAllocator = DLAllocatorProxy> {
    allocator: A,
    begin: *mut T,
    end: *mut T,
    buffer_end: *mut T,
    phantom: PhantomData<[T]>,
}

impl<T, A: DLAllocator> DLVector<T, A> {
    pub fn len(&self) -> usize {
        unsafe { self.end.offset_from(self.begin) as usize }
    }

    pub fn capacity(&self) -> usize {
        unsafe { self.buffer_end.offset_from(self.begin) as usize }
    }
}

impl<T, A: DLAllocator> Deref for DLVector<T, A> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        unsafe { std::slice::from_raw_parts(self.begin, self.len()) }
    }
}

impl<T, A: DLAllocator> DerefMut for DLVector<T, A> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { std::slice::from_raw_parts_mut(self.begin, self.len()) }
    }
}
