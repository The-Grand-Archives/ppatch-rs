use super::{resource::ParamResCap, vector::DLVector};
use crate::vtable::VTable;

#[derive(Debug)]
#[repr(C)]
pub struct CSRegulationManager {
    vtable: VTable,
    regulation_step_task: *mut (),
    param_res_caps: DLVector<ParamResCap>,
}

mod ce_ffi {
    #[link(name = "CE", kind = "raw-dylib")]
    extern "C" {
        pub static CSRegulationManager: *mut super::CSRegulationManager;
    }
}

impl CSRegulationManager {
    pub unsafe fn instance() -> &'static mut Self {
        &mut *ce_ffi::CSRegulationManager
    }
}
