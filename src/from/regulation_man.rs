use super::{resource::ParamResCap, vector::DLVector};
use crate::vtable::VTable;

#[derive(Debug)]
#[repr(C)]
pub struct CSRegulationManager {
    vtable: VTable,
    regulation_step_task: *mut (),
    param_res_caps: DLVector<ParamResCap>,
}
