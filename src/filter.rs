use std::hash::{BuildHasher, Hash};

use crate::hashing::{for_each_set_bit_u64, standard_equation_w64, xor_words};
use crate::params::Params;

#[derive(Debug, Clone)]
pub struct RibbonFilter<S> {
    params: Params,
    build_hasher: S,
    z: Vec<u64>,
    stride_words: usize,
}

impl<S> RibbonFilter<S>
where
    S: BuildHasher + Clone,
{
    pub(crate) fn new(params: Params, build_hasher: S, z: Vec<u64>) -> Self {
        let stride_words = params.fingerprint_words();
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

    pub fn contains<Q: Hash + ?Sized>(&self, key: &Q) -> bool {
        let mut fingerprint = vec![0u64; self.stride_words];
        let equation = standard_equation_w64(
            &self.build_hasher,
            key,
            self.params.seed,
            self.params.m,
            self.params.w,
            &mut fingerprint,
            self.params.fingerprint_last_word_mask(),
        );

        let mut acc = vec![0u64; self.stride_words];
        for_each_set_bit_u64(equation.coeff, |offset| {
            let row = self.z_row(equation.start + offset);
            xor_words(&mut acc, row);
        });

        acc == fingerprint
    }

    fn z_row(&self, row: usize) -> &[u64] {
        let start = row * self.stride_words;
        let end = start + self.stride_words;
        &self.z[start..end]
    }
}
