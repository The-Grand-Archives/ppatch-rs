use num_traits::PrimInt;

use super::base::{FieldBlock, RowPatchId, RowPatcher};
use crate::util::unaligned::{ToUnalignedSlice, Unaligned};

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Ord, Eq)]
pub struct RowDiffId(u16);

impl RowDiffId {
    pub const NONE: RowDiffId = RowDiffId(u16::MAX);
    pub const fn none() -> Self {
        Self::NONE
    }

    pub fn as_index(self) -> Option<usize> {
        (self != Self::NONE).then_some(self.0 as usize)
    }
}

impl Default for RowDiffId {
    fn default() -> Self {
        Self::NONE
    }
}

#[derive(Debug, Clone, Copy, Default)]
struct PatchedFieldRef {
    /// Row diff the PatchedField belongs to.
    diff: RowDiffId,
    /// Index of the PatchedField inside the snapshot's `patched_fields` vector.
    index: u16,
}
impl PatchedFieldRef {
    pub fn is_null(self) -> bool {
        self.diff == RowDiffId::none()
    }

    pub fn new(diff: RowDiffId, index: u16) -> Self {
        Self { diff, index }
    }

    pub fn row_diff<N: PrimInt>(self, container: &impl AsRef<[RowDiff<N>]>) -> Option<&RowDiff<N>> {
        self.diff.as_index().map(|i| &container.as_ref()[i])
    }

    pub fn row_diff_mut<N: PrimInt>(
        self,
        container: &mut impl AsMut<[RowDiff<N>]>,
    ) -> Option<&mut RowDiff<N>> {
        self.diff.as_index().map(|i| &mut container.as_mut()[i])
    }

    pub fn field_and_diff<N: PrimInt>(
        self,
        container: &impl AsRef<[RowDiff<N>]>,
    ) -> Option<(&PatchedField, &[N])> {
        self.row_diff(container).map(|rd| {
            let pf = &rd.patched_fields[self.index as usize];
            (pf, rd.block_diffs.as_slice())
        })
    }

    pub fn field_and_diff_mut<N: PrimInt>(
        self,
        container: &mut impl AsMut<[RowDiff<N>]>,
    ) -> Option<(&mut PatchedField, &mut [N])> {
        self.row_diff_mut(container).map(|rd| {
            let pf = &mut rd.patched_fields[self.index as usize];
            (pf, rd.block_diffs.as_mut_slice())
        })
    }
}

/// Stores information about the patch to an individual field.
#[derive(Debug, Default)]
struct PatchedField {
    /// Start index of the patched field in the field block array.
    field_start: u16,
    /// Start index of the diff for this field in the `block_diffs` vector.
    diff_start: u16,
    /// Patched field that is "on top" of this one in the patch stack (i.e. closer to head of the LL).
    prev: PatchedFieldRef,
    /// PatchedField that is "obscured" by this one.
    next: PatchedFieldRef,
}

#[derive(Debug, Default)]
struct RowDiff<N: PrimInt> {
    /// Stores XOR binary diff from the previous field values.
    /// Contiguous blocks that have changes will be contiguous here.
    block_diffs: Vec<N>,
    /// Start index of the saved fields in the field block array.
    patched_fields: Vec<PatchedField>,
    /// Next free slot in the diffs vector.
    next_free_slot: RowDiffId,
}

/// Row patcher which maintains per-field linked lists to resolve conflicts.
///
/// Boasts excellent time complexity and decent memory usage
/// at the cost of high overhead for maintaining the compacted
/// per-field linked lists.
///
/// ### Memory consumed per patch:
/// `56 + n_bytes_patched + 12*n_fields_patched`
///
/// ### Complexity of [`RowPatcher::create_patch`]
/// `O(n_fields + row_size)`
///
/// ### Complexity of [`RowPatcher::restore_patch`]
/// `O(n_fields_patched + n_bytes_patched)`
///
pub struct LinkedListPatcher<'a, N: PrimInt + Default = u32> {
    diffs: Vec<RowDiff<N>>,
    field_blocks: &'a [FieldBlock<N>],
    patched_field_heads: Vec<PatchedFieldRef>,
    free_list_head: RowDiffId,
}

impl<'a, N: PrimInt + Default> LinkedListPatcher<'a, N> {
    fn allocate_slot(&mut self) -> RowDiffId {
        if let Some(i) = self.free_list_head.as_index() {
            self.free_list_head = self.diffs[i].next_free_slot;
            RowDiffId(i as u16)
        }
        else if self.diffs.len() < u16::MAX as usize {
            self.diffs.push(Default::default());
            RowDiffId((self.diffs.len() - 1) as u16)
        }
        else {
            RowDiffId::none()
        }
    }

    fn reclaim_slot(&mut self, id: RowDiffId) {
        if let Some(diff_index) = id.as_index() {
            if self.free_list_head != RowDiffId::none() {
                self.diffs[diff_index].next_free_slot = self.free_list_head
            }
            self.free_list_head = id;
        }
    }

    fn pf_ll_insert(
        &mut self,
        fb: FieldBlock<N>,
        field: &mut PatchedField,
        field_ref: PatchedFieldRef,
    ) {
        let field_i = fb.field_start as usize;
        let head = &mut self.patched_field_heads[field_i];

        if let Some((pf, _)) = head.field_and_diff_mut(&mut self.diffs) {
            pf.prev = field_ref;
            field.next = *head
        }
        *head = field_ref;
    }
}

impl<'a, N: PrimInt + Default> RowPatcher<'a, N> for LinkedListPatcher<'a, N> {
    fn new(field_blocks: &'a [FieldBlock<N>], _row_size: usize) -> Self {
        Self {
            diffs: Vec::new(),
            field_blocks,
            patched_field_heads: vec![Default::default(); field_blocks.len()],
            free_list_head: Default::default(),
        }
    }

    fn create_patch(
        &mut self,
        before: &[Unaligned<N>],
        after: &[Unaligned<N>],
    ) -> Option<RowPatchId> {
        let mut diff = RowDiff::default();
        let slot = self.allocate_slot();
        if slot == RowDiffId::none() {
            return None;
        }

        let mut i = 0;
        let mut last_offset = None;
        while i < self.field_blocks.len() {
            let fb = self.field_blocks[i];
            let fb_offset = fb.offset as usize;
            if ((before[fb_offset].0 ^ after[fb_offset].0) & fb.mask) == N::zero() {
                i += 1;
                continue;
            }

            let diff_start = if last_offset.map(|x| x < fb_offset).unwrap_or(true) {
                diff.block_diffs.len()
            }
            else {
                diff.block_diffs.len() - 1
            } as u16;

            let mut pf = PatchedField {
                field_start: fb.field_start,
                diff_start,
                ..Default::default()
            };
            self.pf_ll_insert(
                fb,
                &mut pf,
                PatchedFieldRef::new(slot, diff.patched_fields.len() as u16),
            );
            diff.patched_fields.push(pf);

            i = fb.field_start as usize;
            while i < self.field_blocks.len() && self.field_blocks[i].field_start == fb.field_start
            {
                let offset = self.field_blocks[i].offset as usize;
                if last_offset.map(|x| x < offset).unwrap_or(true) {
                    diff.block_diffs.push(before[offset].0 ^ after[offset].0);
                    last_offset = Some(offset);
                }
                i += 1;
            }
        }

        self.diffs[slot.0 as usize] = diff;
        Some(slot.0 as usize)
    }

    fn restore_patch(&mut self, diff_id: RowPatchId, live_memory: &mut [Unaligned<N>]) {
        let slot = RowDiffId(diff_id as u16);
        let diff_index = slot.as_index().unwrap();
        let diff = std::mem::take(&mut self.diffs[diff_index]);

        for pf in &diff.patched_fields {
            let mut i_fb = pf.field_start as usize;
            let fb = self.field_blocks[i_fb];
            let base_offset = fb.offset as usize;

            let blocks =
                if let Some((prev_pf, blocks)) = pf.prev.field_and_diff_mut(&mut self.diffs) {
                    prev_pf.next = pf.next;
                    blocks[prev_pf.diff_start as usize..].to_unaligned_slice_mut()
                }
                else {
                    &mut live_memory[base_offset..]
                };

            let orig_blocks = &diff.block_diffs[pf.diff_start as usize..];
            while i_fb < self.field_blocks.len()
                && self.field_blocks[i_fb].field_start == fb.field_start
            {
                let offset_diff = self.field_blocks[i_fb].offset as usize - base_offset;
                blocks[offset_diff].0 =
                    blocks[offset_diff].0 ^ orig_blocks[offset_diff] & self.field_blocks[i_fb].mask;
                i_fb += 1;
            }

            if let Some((next_pf, _)) = pf.next.field_and_diff_mut(&mut self.diffs) {
                next_pf.prev = pf.prev;
            }
            if self.patched_field_heads[i_fb].diff == slot {
                self.patched_field_heads[i_fb] = pf.next;
            }
        }

        self.reclaim_slot(slot);
    }
}
