use super::{resource::CSParamResCap, vector::DLVector};
use crate::vtable::VTable;

#[derive(Debug)]
#[repr(C)]
pub struct CSRegulationManager {
    vtable: VTable,
    #[cfg(not(feature = "ds3"))]
    regulation_step_task: *mut (),
    param_res_caps: DLVector<CSParamResCap>,
}
