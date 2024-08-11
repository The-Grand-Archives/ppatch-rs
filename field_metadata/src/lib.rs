use num_traits::PrimInt;
use rkyv::AlignedVec;
use std::collections::HashMap;

/// Represents a portion (or superset) of a paramdef field, stored in an integer of type `N`.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct FieldBlock<N: PrimInt> {
    /// Start index of the field in the [`FieldBlock`] array.
    pub field_start: u16,
    /// The offset (as a multiple of `std::mem::size_of::<N>()`) of this field part in the struct.
    pub offset: u16,
    /// A bitmask with the bits that belong to the field set to 1.
    pub mask: N,
}

impl<N: PrimInt> rkyv::Archive for FieldBlock<N> {
    type Archived = FieldBlock<N>;
    type Resolver = FieldBlock<N>;

    unsafe fn resolve(&self, pos: usize, resolver: Self::Resolver, out: *mut Self::Archived) {
        (*out).field_start = self.field_start;
        (*out).offset = self.offset;
        (*out).mask = self.mask;
    }
}
impl<S: rkyv::ser::Serializer, N: PrimInt> rkyv::Serialize<S> for FieldBlock<N> {
    fn serialize(&self, serializer: &mut S) -> Result<Self::Resolver, S::Error> {
        Ok(*self)
    }
}

pub type Block = u32;
pub type FieldBlockRepo = HashMap<String, Vec<FieldBlock<Block>>>;
pub type ArchivedFieldBlockRepo = <FieldBlockRepo as rkyv::Archive>::Archived;

pub unsafe fn load_fb_repo(bytes: &[u8]) -> &ArchivedFieldBlockRepo {
    rkyv::archived_root::<FieldBlockRepo>(bytes)
}

pub fn serialize_fb_repo(repo: &FieldBlockRepo) -> Box<[u8]> {
    rkyv::to_bytes::<_, 4096>(repo).unwrap().into_boxed_slice()
}
