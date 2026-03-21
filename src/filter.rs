use std::hash::{BuildHasher, Hash};

use crate::builder::Scratch;
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
            self.params.m,
            self.params.w,
            self.params.mode,
            &mut scratch.fingerprint,
            self.params.fingerprint_last_word_mask(),
        );

        for_each_set_bit_u64(equation.coeff, |offset| {
            let row = self.z_row(equation.start + offset);
            xor_words(&mut scratch.acc, row);
        });

        scratch.acc == scratch.fingerprint
    }

    fn z_row(&self, row: usize) -> &[u64] {
        let start = row * self.stride_words;
        let end = start + self.stride_words;
        &self.z[start..end]
    }
}
