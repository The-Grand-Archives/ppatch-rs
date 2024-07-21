use std::ops::{Deref, DerefMut};

use super::{component::FD4ComponentBase, string::FD4BasicHashString};
use crate::vtable::VTable;

pub type FD4ResHashString = FD4BasicHashString<u16>;

#[derive(Debug)]
#[repr(C)]
pub struct FD4ResCapHolderItem {
    vtable: VTable,
    pub res_name: FD4ResHashString,
    pub repository: *const (),
    pub next_item: *mut FD4ResCapHolderItem,
    pub ref_count: usize,
}

unsafe impl FD4ComponentBase for FD4ResCapHolderItem {
    fn vmt(&self) -> *const fn() {
        return self.vtable;
    }
}

#[derive(Debug)]
#[repr(C)]
pub struct FD4ResCap {
    pub res_cap_holder_item: FD4ResCapHolderItem,
    pub is_debug: bool,
    pub unk_61: bool,
    pub debug_menu_item: *mut (),
    pub unk_70: bool,
}

impl Deref for FD4ResCap {
    type Target = FD4ResCapHolderItem;
    fn deref(&self) -> &Self::Target {
        &self.res_cap_holder_item
    }
}
impl DerefMut for FD4ResCap {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.res_cap_holder_item
    }
}

#[derive(Debug)]
#[repr(C)]
pub struct FD4ParamResCap {
    rescap: FD4ResCap,
    file_size: usize,
    file: *mut u8,
}

impl Deref for FD4ParamResCap {
    type Target = FD4ResCap;
    fn deref(&self) -> &Self::Target {
        &self.rescap
    }
}
impl DerefMut for FD4ParamResCap {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.rescap
    }
}

#[derive(Debug)]
#[repr(C)]
pub struct CSParamResCap {
    rescap: FD4ResCap,
    unk_78: u32,
    fd4_res_cap: *mut FD4ParamResCap,
}

impl Deref for CSParamResCap {
    type Target = FD4ResCap;
    fn deref(&self) -> &Self::Target {
        &self.rescap
    }
}
impl DerefMut for CSParamResCap {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.rescap
    }
}
