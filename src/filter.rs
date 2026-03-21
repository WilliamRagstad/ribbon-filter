use bitvec::prelude::{BitVec, Lsb0};
use std::hash::{BuildHasher, Hash};

use crate::builder::Scratch;
use crate::hashing::{for_each_set_bit_u128_parts, standard_equation_w64, xor_words};
use crate::params::Params;

#[derive(Debug, Clone)]
pub struct RibbonFilter<S> {
    params: Params,
    build_hasher: S,
    z: BitVec<u64, Lsb0>,
    stride_words: usize,
}

impl<S> RibbonFilter<S>
where
    S: BuildHasher + Clone,
{
    pub(crate) fn new(params: Params, build_hasher: S, z: Vec<u64>) -> Self {
        let stride_words = params.fingerprint_words();
        let z = BitVec::<u64, Lsb0>::from_vec(z);
        Self {
            params,
            build_hasher,
            z,
            stride_words,
        }
    }

    pub fn params(&self) -> Params {
        self.params
    }

    pub fn new_scratch(&self) -> Scratch {
        Scratch::new(self.stride_words)
    }

    pub fn contains<Q: Hash + ?Sized>(&self, key: &Q) -> bool {
        let mut scratch = self.new_scratch();
        self.contains_in(key, &mut scratch)
    }

    pub fn contains_in<Q: Hash + ?Sized>(&self, key: &Q, scratch: &mut Scratch) -> bool {
        debug_assert_eq!(scratch.fingerprint.len(), self.stride_words);
        debug_assert_eq!(scratch.acc.len(), self.stride_words);
        scratch.reset();

        let equation = standard_equation_w64(
            &self.build_hasher,
            key,
            self.params.seed,
            &self.params,
            &mut scratch.fingerprint,
        );

        for_each_set_bit_u128_parts(equation.coeff_lo, equation.coeff_hi, |offset| {
            let row = self.z_row(equation.start + offset);
            xor_words(&mut scratch.acc, row);
        });

        scratch.acc == scratch.fingerprint
    }

    fn z_row(&self, row: usize) -> &[u64] {
        let start = row * self.stride_words;
        let end = start + self.stride_words;
        &self.z.as_raw_slice()[start..end]
    }
}
