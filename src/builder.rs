use std::hash::BuildHasher;

use crate::error::{BuildError, ConstructionFailure};
use crate::filter::RibbonFilter;
use crate::hashing::{for_each_set_bit_u64, standard_equation_w64, xor_words};
use crate::params::Params;

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

        let m = self.params.m;
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
                self.params.seed,
                m,
                self.params.w,
                &mut key_fp,
                fp_last_mask,
            );

            let mut i = equation.start;
            let mut c = equation.coeff;
            let mut b = key_fp.clone();

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
                    return Err(BuildError::ConstructionFailed(
                        ConstructionFailure::InconsistentEquation {
                            key_index,
                            row_index: i,
                        },
                    ));
                }

                let shift = c.trailing_zeros() as usize;
                i += shift;
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
                let mut row_copy = vec![0u64; stride_words];
                row_copy.copy_from_slice(&z[other_start..other_end]);
                xor_words(&mut z[row_start..row_end], &row_copy);
            });

            z[row_end - 1] &= fp_last_mask;
        }

        Ok(RibbonFilter::new(self.params, self.build_hasher.clone(), z))
    }
}
