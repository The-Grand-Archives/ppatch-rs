#[repr(C, packed)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Unaligned<N>(pub N);

pub trait ToUnaligned {
    fn to_unaligned(&self) -> &Unaligned<Self>
    where
        Self: Sized;

    fn to_unaligned_mut(&mut self) -> &mut Unaligned<Self>
    where
        Self: Sized;
}
impl<T> ToUnaligned for T {
    fn to_unaligned(&self) -> &Unaligned<Self>
    where
        Self: Sized,
    {
        // SAFETY: the layout of T is valid for Unaligned<T>
        unsafe { std::mem::transmute(self) }
    }

    fn to_unaligned_mut(&mut self) -> &mut Unaligned<Self>
    where
        Self: Sized,
    {
        // SAFETY: the layout of T is valid for Unaligned<T>
        unsafe { std::mem::transmute(self) }
    }
}

pub trait ToUnalignedSlice<T> {
    fn to_unaligned_slice(&self) -> &[Unaligned<T>];
    fn to_unaligned_slice_mut(&mut self) -> &mut [Unaligned<T>];
}
impl<T> ToUnalignedSlice<T> for [T] {
    fn to_unaligned_slice(&self) -> &[Unaligned<T>] {
        unsafe { std::slice::from_raw_parts(self.as_ptr() as *const Unaligned<T>, self.len()) }
    }

    fn to_unaligned_slice_mut(&mut self) -> &mut [Unaligned<T>] {
        unsafe {
            std::slice::from_raw_parts_mut(self.as_mut_ptr() as *mut Unaligned<T>, self.len())
        }
    }
}

pub trait ToUnalignedArray<T: Copy, const N: usize> {
    fn to_unaligned_array(self) -> [Unaligned<T>; N]
    where
        Self: Sized;
    fn to_unaligned_fixed_slice(&self) -> &[Unaligned<T>; N];
    fn to_unaligned_fixed_slice_mut(&mut self) -> &mut [Unaligned<T>; N];
}
impl<T: Copy, const N: usize> ToUnalignedArray<T, N> for [T; N] {
    fn to_unaligned_array(self) -> [Unaligned<T>; N]
    where
        Self: Sized,
    {
        unsafe { std::mem::transmute_copy(&self) }
    }

    fn to_unaligned_fixed_slice(&self) -> &[Unaligned<T>; N] {
        unsafe { std::mem::transmute(self) }
    }

    fn to_unaligned_fixed_slice_mut(&mut self) -> &mut [Unaligned<T>; N] {
        unsafe { unsafe { std::mem::transmute(self) } }
    }
}
