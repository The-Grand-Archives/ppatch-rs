use std::u32;

use num_traits::PrimInt;

use super::base::{FieldBlock, RowPatchId, RowPatcher};
use crate::util::unaligned::Unaligned;

#[derive(Debug, Clone, Default)]
struct PatchedBlock<N: PrimInt> {
    /// XOR bitwise diff of the changes made to the block.
    diff: N,
    /// Bitmask where all bits of affected fields are set to 1.
    mask: N,
    /// Offset of block from the start of the param row.
    offset: u32,
}

#[derive(Debug, Clone, Default)]
struct RowDiff<N: PrimInt> {
    /// Array of 4-byte blocks that were patched, in ascending order.
    blocks: Box<[PatchedBlock<N>]>,
    // The ID of this patch.
    id: RowPatchId,
}

impl<N: PrimInt> RowDiff<N> {
    /// Merge this row diff with another in linear time using a mergesort like strategy.
    pub fn merge(&mut self, rd: &RowDiff<N>) {
        let mut new_blocks = Vec::with_capacity(self.blocks.len() + rd.blocks.len());

        let (mut i, mut j) = (0, 0);
        while i < self.blocks.len() && j < rd.blocks.len() {
            let a = &self.blocks[i];
            let b = &rd.blocks[j];
            let merged = if a.offset < b.offset {
                i += 1;
                a.clone()
            } else if a.offset > b.offset {
                j += 1;
                b.clone()
            } else {
                i += 1;
                j += 1;
                PatchedBlock {
                    diff: a.diff ^ b.diff,
                    mask: a.mask | b.mask,
                    offset: a.offset,
                }
            };
            if merged.mask.is_zero() {
                new_blocks.push(merged);
            }
        }
        new_blocks.extend_from_slice(&self.blocks[i..]);
        new_blocks.extend_from_slice(&rd.blocks[j..]);

        self.blocks = new_blocks.into_boxed_slice();
    }
}

#[derive(Debug, Clone, Default)]
struct MaskBlock<N: PrimInt + Default> {
    /// Value of the combined field mask for this block.
    value: N,
    /// The restore "step" ID, used to tag which blocks have been written to.
    step: u32,
}

/// Row patcher which maintains a stack of sparse patch arrays.
///
/// Uses a custom field block format enabling extremely fast
/// `create_patch` execution, especially for sparse patches.
///
/// `restore_patch` will also be very fast for top-of-stack (recently created)
/// patches, but has poor asymptotic complexity for patches at the bottom of the
/// stack.
///
///
/// ### Memory consumed per patch
/// `24 + 3*n_bytes_patched`
///
/// ### Complexity of [`RowPatcher::create_patch`]
/// `O(n_fields + row_size)`
///
/// ### Complexity of [`RowPatcher::restore_patch`]
/// O(sum of number of bytes patched for all patches above and including the restored patch)
///
#[derive(Debug, Clone)]
struct SparseArrayPatcher<N: PrimInt + Default = u32> {
    diff_stack: Vec<RowDiff<N>>,
    combined_mask: Box<[MaskBlock<N>]>,
    field_blocks: Box<[N]>,
    id_counter: usize,
    step_counter: u32,
}

impl<'a, N: PrimInt + Default> RowPatcher<'a, N> for SparseArrayPatcher<N> {
    fn new(field_blocks: &'a [FieldBlock<N>], row_size: usize) -> Self {
        // Convert "standard" field block format into optimized bit format
        let mut bin_fb: Vec<N> = Vec::new();
        let max_bit = !(N::max_value() >> 1);

        for (i, fb) in field_blocks.iter().enumerate() {
            let mut b = if fb.mask.is_zero() {
                N::zero()
            } else {
                N::from(max_bit >> fb.mask.leading_zeros() as usize).unwrap()
            };
            if let Some(next) = field_blocks.get(i + 1) {
                if next.field_start == fb.field_start {
                    b = N::zero();
                }
            }
            if let Some(prev) = field_blocks.get(i - 1) {
                if prev.offset == fb.offset {
                    let last = bin_fb.last_mut().unwrap();
                    *last = *last | b;
                    continue;
                }
                // Field skipping is not supported
                assert!(fb.offset == prev.offset + 1);
            }
            bin_fb.push(b);
        }

        Self {
            diff_stack: Vec::new(),
            combined_mask: vec![MaskBlock::default(); row_size / std::mem::size_of::<N>()]
                .into_boxed_slice(),
            field_blocks: bin_fb.into_boxed_slice(),
            id_counter: 0,
            step_counter: 0,
        }
    }

    fn create_patch(
        &mut self,
        before: &[Unaligned<N>],
        after: &[Unaligned<N>],
    ) -> Option<RowPatchId> {
        // This assert passing will give info to the compiler
        assert!(self.field_blocks.len() == before.len() && self.field_blocks.len() == after.len());

        let mut rd_blocks: Vec<PatchedBlock<N>> = Vec::new();

        let mut i = 0;
        let mut last_block = 0;
        let mut last_acc = N::max_value();
        let mut last_field_done = true;

        while i < self.field_blocks.len() {
            let diff = before[i].0 ^ after[i].0;
            let mut fields = self.field_blocks[i];

            if diff.is_zero() && last_field_done {
                match fields.leading_zeros() {
                    32 => (),
                    n => {
                        last_acc = N::max_value() >> n as usize;
                        last_block = i as u32;
                    }
                }
                i += 1;
                continue;
            }
            if last_acc != N::max_value() {
                if let Some(blk) = rd_blocks.last_mut() {
                    blk.mask = blk.mask | !last_acc;
                } else {
                    rd_blocks.push(PatchedBlock {
                        offset: last_block,
                        diff: N::zero(),
                        mask: !last_acc,
                    });
                }
            }
            rd_blocks.extend((last_block + 1..i as u32).map(|o| PatchedBlock {
                diff: N::zero(),
                mask: N::max_value(),
                offset: o as u32,
            }));

            let mut mask = N::zero();
            let mut acc = N::zero();
            while !fields.is_zero() {
                let fm = (fields ^ (fields - N::one())) & !acc;
                if !(fm & diff).is_zero() {
                    mask = mask | fm;
                }
                acc = acc | fm;
                fields = fields & (fields - N::one());
            }

            rd_blocks.push(PatchedBlock {
                offset: i as u32,
                diff,
                mask,
            });
            last_field_done = acc == N::max_value();
            last_acc = acc;
            last_block = i as u32;
            i += 1;
        }

        self.id_counter += 1;
        self.diff_stack.push(RowDiff {
            blocks: rd_blocks.into_boxed_slice(),
            id: self.id_counter,
        });
        Some(self.id_counter)
    }

    fn restore_patch(&mut self, id: RowPatchId, live_memory: &mut [Unaligned<N>]) {
        // Find and remove the row diff from the stack
        let mut i = self.diff_stack.len();
        loop {
            i = i.checked_sub(1).unwrap();
            if self.diff_stack[i].id == id {
                break;
            }
        }
        let mut rd = self.diff_stack.remove(i);

        // If the step overflows, we have to reset the combined mask!
        self.step_counter = match self.step_counter.wrapping_add(1) {
            0 => {
                self.combined_mask.fill(MaskBlock::default());
                1
            }
            n => n,
        };

        // Compute bitmask of "obscured" changes
        for rd in &self.diff_stack[i..] {
            for b in rd.blocks.iter() {
                let m = &mut self.combined_mask[b.offset as usize];
                m.value = m.value | b.mask;
                m.step = self.step_counter;
            }
        }

        // Apply only "visible changes" and remove them from the restored RowDiff
        for b in rd.blocks.iter_mut() {
            let ofs = b.offset as usize;
            let m = &self.combined_mask[ofs];
            let hidden = if m.step == self.step_counter { m.value } else { N::zero() };
            live_memory[ofs].0 = live_memory[ofs].0 ^ (b.diff & !hidden);
            b.mask = b.mask & hidden;
            b.diff = b.diff & hidden;
        }

        // Merge with next patch if necessary
        self.diff_stack.get_mut(i).map(|next| next.merge(&rd));
    }
}
