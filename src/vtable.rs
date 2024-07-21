macro_rules! vtable_entries {
    ($idx:literal => fn $n:ident(&self $(,$an:ident: $at:ty)*) -> $ret:ty;) => {
         fn $n(&self, $($an: $at),*) -> $ret {
            unsafe {
                let fptr = self.vmt().add($idx).read();
                #[cfg(all(target_os = "windows", target_pointer_width = "64"))]
                let typed: extern "fastcall" fn(&Self, $($at),*) -> $ret = core::mem::transmute(fptr);
                #[cfg(all(target_os = "windows", target_pointer_width = "32"))]
                let typed: extern "thiscall" fn(&Self, $($at),*) -> $ret = core::mem::transmute(fptr);
                typed(self, $($an),*)
            }
        }
    };

    ($idx:literal => fn $n:ident(&mut self $(,$an:ident: $at:ty)*) -> $ret:ty;) => {
        fn $n(&mut self, $($an: $at),*) -> $ret {
            unsafe {
                let fptr = self.vmt().add($idx).read();
                #[cfg(all(target_os = "windows", target_pointer_width = "64"))]
                let typed: extern "fastcall" fn(&mut Self, $($at),*) -> $ret = core::mem::transmute(fptr);
                #[cfg(all(target_os = "windows", target_pointer_width = "32"))]
                let typed: extern "thiscall" fn(&mut Self, $($at),*) -> $ret = core::mem::transmute(fptr);
                typed(self, $($an),*)
            }
        }
    };


    ($idx:literal => unsafe fn $n:ident(&self $(,$an:ident: $at:ty)*) -> $ret:ty;) => {
        unsafe fn $n(&self, $($an: $at),*) -> $ret {
            let fptr = self.vmt().add($idx).read();
            #[cfg(all(target_os = "windows", target_pointer_width = "64"))]
            let typed: extern "fastcall" fn(&Self, $($at),*) -> $ret = core::mem::transmute(fptr);
            #[cfg(all(target_os = "windows", target_pointer_width = "32"))]
            let typed: extern "thiscall" fn(&Self, $($at),*) -> $ret = core::mem::transmute(fptr);
            typed(self, $($an),*)
        }
    };

    ($idx:literal => unsafe fn $n:ident(&mut self $(,$an:ident: $at:ty)*) -> $ret:ty;) => {
        unsafe fn $n(&mut self, $($an: $at),*) -> $ret {
            let fptr = self.vmt().add($idx).read();
            #[cfg(all(target_os = "windows", target_pointer_width = "64"))]
            let typed: extern "fastcall" fn(&mut Self, $($at),*) -> $ret = core::mem::transmute(fptr);
            #[cfg(all(target_os = "windows", target_pointer_width = "32"))]
            let typed: extern "thiscall" fn(&mut Self, $($at),*) -> $ret = core::mem::transmute(fptr);
            typed(self, $($an),*)
        }
    };

    ($idx:literal => fn $n:ident(&self $(,$an:ident: $at:ty)*) -> $ret:ty; $($extra:tt)*) => {
        $crate::vtable::vtable_entries! { $idx => fn $n(&self $(,$an: $at)*) -> $ret; }
        $crate::vtable::vtable_entries! { $($extra)* }
    };

    ($idx:literal => fn $n:ident(&mut self $(,$an:ident: $at:ty)*) -> $ret:ty; $($extra:tt)*) => {
        $crate::vtable::vtable_entries! { $idx => fn $n(&mut self $(,$an: $at)*) -> $ret; }
        $crate::vtable::vtable_entries! { $($extra)* }
    };

    ($idx:literal => unsafe fn $n:ident(&self $(,$an:ident: $at:ty)*) -> $ret:ty; $($extra:tt)*) => {
        $crate::vtable::vtable_entries! { $idx => unsafe fn $n(&self $(,$an: $at)*) -> $ret; }
        $crate::vtable::vtable_entries! { $($extra)* }
    };

    ($idx:literal => unsafe fn $n:ident(&mut self $(,$an:ident: $at:ty)*) -> $ret:ty; $($extra:tt)*) => {
        $crate::vtable::vtable_entries! { $idx => unsafe fn $n(&mut self $(,$an: $at)*) -> $ret; }
        $crate::vtable::vtable_entries! { $($extra)* }
    }
}

macro_rules! vtable {
    () => {
        fn vmt(&self) -> *const fn();
    };

    ($($defs:tt)*) => {
        fn vmt(&self) -> *const fn();
        $crate::vtable::vtable_entries! { $($defs)* }
    };
}

pub(crate) use vtable;
pub(crate) use vtable_entries;

pub type VTable = *const fn();
