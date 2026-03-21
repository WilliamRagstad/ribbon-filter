use std::hash::BuildHasher;

use crate::error::{BuildError, ConstructionFailure};
use crate::filter::RibbonFilter;
use crate::hashing::{derive_attempt_seed, for_each_set_bit_u64, standard_equation_w64, xor_words};
use crate::params::{Mode, Params};

#[derive(Debug, Clone)]
pub struct Scratch {
    pub(crate) fingerprint: Vec<u64>,
    pub(crate) acc: Vec<u64>,
}

impl Scratch {
    pub(crate) fn new(stride_words: usize) -> Self {
        Self {
            fingerprint: vec![0; stride_words],
            acc: vec![0; stride_words],
        }
    }

    pub(crate) fn reset(&mut self) {
        self.fingerprint.fill(0);
        self.acc.fill(0);
    }
}

#[derive(Debug, Clone)]
pub struct RibbonBuilder<S> {
    params: Params,
    build_hasher: S,
}

impl<S> RibbonBuilder<S>
where
    S: BuildHasher + Clone,
{
    pub fn new(params: Params, build_hasher: S) -> Result<Self, BuildError> {
        params.validate().map_err(BuildError::InvalidParams)?;
        Ok(Self {
            params,
            build_hasher,
        })
    }

    pub fn params(&self) -> Params {
        self.params
    }

    pub fn hasher(&self) -> &S {
        &self.build_hasher
    }

    pub fn build<K: std::hash::Hash>(&self, keys: &[K]) -> Result<RibbonFilter<S>, BuildError> {
        self.params.validate().map_err(BuildError::InvalidParams)?;

        let mut attempts = 0usize;
        let mut current_m = self.params.m;
        let mut last_failure = None;

        for grow_step in 0..=self.params.grow_limit {
            for retry_step in 0..self.params.retry_limit {
                attempts += 1;
                let attempt_index = ((grow_step as u64) << 32) | retry_step as u64;
                let seed = derive_attempt_seed(self.params.seed, attempt_index);

                match self.build_once(keys, current_m, seed) {
                    Ok(filter) => return Ok(filter),
                    Err(err) => last_failure = Some(err),
                }

                if matches!(self.params.mode, Mode::Homogeneous) {
                    break;
                }
            }

            if matches!(self.params.mode, Mode::Homogeneous) {
                break;
            }

            if grow_step < self.params.grow_limit {
                let w = self.params.w;
                current_m = (current_m * (w + 1)).div_ceil(w);
                debug_assert!(current_m >= self.params.w);
            }
        }

        Err(BuildError::ConstructionFailed {
            final_m: current_m,
            attempts,
            last_failure: last_failure.unwrap_or(ConstructionFailure::InconsistentEquation {
                key_index: 0,
                row_index: 0,
            }),
        })
    }

    fn build_once<K: std::hash::Hash>(
        &self,
        keys: &[K],
        m: usize,
        seed: u64,
    ) -> Result<RibbonFilter<S>, ConstructionFailure> {
        debug_assert!(m >= self.params.w);

        let stride_words = self.params.fingerprint_words();
        let fp_last_mask = self.params.fingerprint_last_word_mask();

        let mut occupied = vec![false; m];
        let mut coeff = vec![0u64; m];
        let mut rhs = vec![0u64; m * stride_words];

        let mut key_fp = vec![0u64; stride_words];

        for (key_index, key) in keys.iter().enumerate() {
            key_fp.fill(0);
            let equation = standard_equation_w64(
                &self.build_hasher,
                key,
                seed,
                m,
                self.params.w,
                self.params.mode,
                &mut key_fp,
                fp_last_mask,
            );

            let mut i = equation.start;
            let mut c = equation.coeff;
            let mut b = key_fp.clone();

            debug_assert!(i < m);

            loop {
                if !occupied[i] {
                    occupied[i] = true;
                    coeff[i] = c;
                    rhs[i * stride_words..(i + 1) * stride_words].copy_from_slice(&b);
                    break;
                }

                c ^= coeff[i];
                xor_words(&mut b, &rhs[i * stride_words..(i + 1) * stride_words]);

                if c == 0 {
                    if b.iter().all(|&x| x == 0) {
                        break;
                    }
                    return Err(ConstructionFailure::InconsistentEquation {
                        key_index,
                        row_index: i,
                    });
                }

                let shift = c.trailing_zeros() as usize;
                i += shift;
                debug_assert!(i < m);
                c >>= shift;
            }
        }

        let mut z = vec![0u64; m * stride_words];
        for i in (0..m).rev() {
            if !occupied[i] {
                continue;
            }

            let row_start = i * stride_words;
            let row_end = row_start + stride_words;

            z[row_start..row_end].copy_from_slice(&rhs[row_start..row_end]);

            let upper_coeff = coeff[i] & !1u64;
            for_each_set_bit_u64(upper_coeff, |offset| {
                let other_start = (i + offset) * stride_words;
                let other_end = other_start + stride_words;
                debug_assert!(i + offset < m);
                let mut row_copy = vec![0u64; stride_words];
                row_copy.copy_from_slice(&z[other_start..other_end]);
                xor_words(&mut z[row_start..row_end], &row_copy);
            });

            z[row_end - 1] &= fp_last_mask;
        }
        let mut built_params = self.params;
        built_params.m = m;
        built_params.seed = seed;

        Ok(RibbonFilter::new(
            built_params,
            self.build_hasher.clone(),
            z,
        ))
    }
}
