use crate::vtable::vtable;

pub unsafe trait FD4ComponentBase {
    vtable! {
        0 => unsafe fn destruct(&mut self) -> ();
        1 => fn runtime_class(&self) -> *const ();
    }
}
