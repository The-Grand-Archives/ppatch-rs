use criterion::{criterion_group, criterion_main, Criterion};
use num_traits::PrimInt;
use ppatch::patchers::{self, base::FieldBlock};
use rand::distributions::WeightedIndex;
use rand::prelude::*;

pub fn gen_field_blocks<N: PrimInt>(
    row_size: usize,
    packed: bool,
    field_size_weights: &[usize],
) -> Vec<FieldBlock<N>> {
    let mut rng = thread_rng();
    let mut blocks = Vec::new();

    let size_dist = WeightedIndex::new(field_size_weights).unwrap();
    let bit_offset = 0;
    loop {
        let sz = 1 << size_dist.sample(&mut rng);
        let mask = N::max_value() >> (8 * sz) << bit_offset;
    }

    blocks
}

pub fn test_patch_register(c: &mut Criterion) {}

criterion_group!(benches, test_patch_register);
criterion_main!(benches);
