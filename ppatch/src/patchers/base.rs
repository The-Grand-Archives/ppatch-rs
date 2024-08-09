use num_traits::PrimInt;

use crate::util::unaligned::Unaligned;

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

/// Type representing an ID for a given row patch.
///
/// A[`RowPatchId`] is only guaranteed to be unique for a specific instance of [`RowPatcher`].
pub type RowPatchId = usize;

/// Trait representing a data structure for creating and restoring
/// patches to a single param row, working in blocks of `N`.
pub trait RowPatcher<'a, N: PrimInt = u32> {
    fn new(field_blocks: &'a [FieldBlock<N>], row_size: usize) -> Self;

    fn create_patch(
        &mut self,
        before: &[Unaligned<N>],
        after: &[Unaligned<N>],
    ) -> Option<RowPatchId>;

    fn restore_patch(&mut self, id: RowPatchId, live_memory: &mut [Unaligned<N>]);
}
