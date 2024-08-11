use crate::util::unaligned::Unaligned;
pub use field_metadata::FieldBlock;
use num_traits::PrimInt;

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
