use crate::vtable::{vtable, vtable_entries, VTable};

pub unsafe trait DLAllocator {
    vtable! {
        0 => unsafe fn destruct(&mut self) -> ();
        1 => fn allocator_id(&self) -> i32;
        // 2 unknown
        3 => fn heap_flags(&self) -> u8;
        4 => fn heap_capacity(&self) -> usize;
        5 => fn heap_size(&self) -> usize;
        6 => fn backing_heap_capacity(&self) -> usize;
        7 => fn heap_allocation_count(&self) -> usize;
        8 => fn block_size(&self, memory: *const ()) -> usize;
        9 => fn allocate(&mut self, cb: usize) -> * mut ();
        10 => fn allocate_aligned(&mut self, cb: usize, align: usize) -> *mut ();
        11 => fn reallocate(&mut self, cb: usize) -> *mut ();
        12 => fn reallocate_aligned(&mut self, cb: usize, align: usize) -> *mut ();
        13 => fn deallocate(&mut self, ptr: *mut ()) -> ();
        // 14 unknown
    }
}

pub unsafe trait DLBackAllocator: DLAllocator {
    vtable_entries! {
        15 => fn allocate_second(&mut self, cb: usize) -> * mut ();
        16 => fn allocate_second_aligned(&mut self, cb: usize, align: usize) -> *mut ();
        17 => fn reallocate_second(&mut self, cb: usize) -> *mut ();
        18 => fn reallocate_second_aligned(&mut self, cb: usize, align: usize) -> *mut ();
        19 => fn deallocate_second(&mut self, ptr: *mut ()) -> ();
        // 20 unknown
        21 => fn belongs_to_first(&self, ptr: *const ()) -> bool;
        22 => fn belongs_to_second(&self, ptr: *const ()) -> bool;
        23 => fn lock(&mut self) -> ();
        24 => fn unlock(&mut self) -> ();
        25 => fn block_base(&self, ptr: *const ()) -> *const ();
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct DLAllocatorProxy {
    instance_ptr: &'static VTable,
}
unsafe impl DLAllocator for DLAllocatorProxy {
    fn vmt(&self) -> *const fn() {
        return *self.instance_ptr;
    }
}
