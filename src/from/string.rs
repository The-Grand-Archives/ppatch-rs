use std::ops::{Deref, DerefMut};

use super::allocator::{DLAllocator, DLAllocatorProxy};
use crate::vtable::VTable;

mod private {
    pub trait Sealed {}
}

pub trait IStringStorage<C: Copy> {
    unsafe fn get(&self, len: usize) -> &[C];
    unsafe fn get_mut(&mut self, len: usize) -> &mut [C];
}

#[repr(C)]
pub union StringStorage<C: Copy, const N: usize> {
    in_place: [C; N],
    ptr: *mut C,
}

impl<C: Copy, const N: usize> IStringStorage<C> for StringStorage<C, N> {
    unsafe fn get(&self, len: usize) -> &[C] {
        let p = if len >= N {
            self.ptr
        }
        else {
            self.in_place.as_ptr()
        };
        std::slice::from_raw_parts(p, len)
    }

    unsafe fn get_mut(&mut self, len: usize) -> &mut [C] {
        let p = if len >= N {
            self.ptr
        }
        else {
            self.in_place.as_mut_ptr()
        };
        std::slice::from_raw_parts_mut(p, len)
    }
}

pub trait Char: private::Sealed + Copy {
    type Storage: IStringStorage<Self>;
}
impl private::Sealed for u8 {}
impl Char for u8 {
    type Storage = StringStorage<Self, 16>;
}
impl private::Sealed for u16 {}
impl Char for u16 {
    type Storage = StringStorage<Self, 8>;
}

#[derive(fmt_derive::Debug)]
#[repr(C)]
pub struct DLString<C: Char = u8, A: DLAllocator = DLAllocatorProxy> {
    #[cfg(not(feature = "ds3"))]
    allocator: A,
    storage: C::Storage,
    len: usize,
    capacity: usize,
    #[cfg(feature = "ds3")]
    allocator: A,
}

impl<C: Char, A: DLAllocator> DLString<C, A> {
    pub fn len(&self) -> usize {
        self.len
    }

    pub fn capacity(&self) -> usize {
        self.capacity
    }
}

impl<C: Char, A: DLAllocator> Deref for DLString<C, A> {
    type Target = [C];
    fn deref(&self) -> &Self::Target {
        unsafe { self.storage.get(self.len) }
    }
}

impl<C: Char, A: DLAllocator> DerefMut for DLString<C, A> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { self.storage.get_mut(self.len) }
    }
}

pub type DLWString<A = DLAllocatorProxy> = DLString<u16, A>;

#[derive(Debug)]
#[repr(C)]
pub struct FD4BasicHashString<C: Char, A: DLAllocator = DLAllocatorProxy> {
    vtable: VTable,
    string: DLString<C, A>,
    unk_08: usize,
    hash: u32,
    requires_rehash: bool,
}

impl<C: Char, A: DLAllocator> FD4BasicHashString<C, A> {
    pub fn hash(&self) -> u32 {
        self.hash // Here, we would recompute the hash if requires_rehash was true
    }

    pub fn requires_rehash(&self) -> bool {
        self.requires_rehash
    }
}

impl<C: Char, A: DLAllocator> Deref for FD4BasicHashString<C, A> {
    type Target = DLString<C, A>;
    fn deref(&self) -> &Self::Target {
        &self.string
    }
}

impl<C: Char, A: DLAllocator> DerefMut for FD4BasicHashString<C, A> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.string
    }
}
