use bitvec::prelude::{BitVec, Lsb0};
use std::hash::{BuildHasher, Hash};

use crate::builder::Scratch;
use crate::hashing::{for_each_set_bit_u128_parts, standard_equation_w64, xor_words};
use crate::params::Params;

#[cfg(feature = "serde")]
const RIBBON_FILTER_FORMAT_VERSION: u8 = 1;

#[cfg(feature = "serde")]
#[derive(serde::Serialize, serde::Deserialize)]
struct RibbonFilterRepr<S> {
    version: u8,
    params: Params,
    build_hasher: S,
    z_words: Vec<u64>,
}

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
            let row_index = equation.start + offset;
            if row_index < self.params.m {
                let row = self.z_row(row_index);
                xor_words(&mut scratch.acc, row);
            }
        });

        scratch.acc == scratch.fingerprint
    }

    fn z_row(&self, row: usize) -> &[u64] {
        let start = row * self.stride_words;
        let end = start + self.stride_words;
        &self.z.as_raw_slice()[start..end]
    }
}

#[cfg(feature = "serde")]
impl<S> serde::Serialize for RibbonFilter<S>
where
    S: BuildHasher + Clone + serde::Serialize,
{
    fn serialize<Ser>(&self, serializer: Ser) -> Result<Ser::Ok, Ser::Error>
    where
        Ser: serde::Serializer,
    {
        RibbonFilterRepr {
            version: RIBBON_FILTER_FORMAT_VERSION,
            params: self.params,
            build_hasher: self.build_hasher.clone(),
            z_words: self.z.as_raw_slice().to_vec(),
        }
        .serialize(serializer)
    }
}

#[cfg(feature = "serde")]
impl<'de, S> serde::Deserialize<'de> for RibbonFilter<S>
where
    S: BuildHasher + Clone + serde::Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let repr = RibbonFilterRepr::<S>::deserialize(deserializer)?;

        if repr.version != RIBBON_FILTER_FORMAT_VERSION {
            return Err(serde::de::Error::custom(format!(
                "unsupported RibbonFilter version {}, expected {}",
                repr.version, RIBBON_FILTER_FORMAT_VERSION
            )));
        }

        repr.params.validate().map_err(serde::de::Error::custom)?;

        let stride_words = repr.params.fingerprint_words();
        let expected_len = repr
            .params
            .m
            .checked_mul(stride_words)
            .ok_or_else(|| serde::de::Error::custom("z_words length overflow"))?;

        if repr.z_words.len() != expected_len {
            return Err(serde::de::Error::custom(format!(
                "invalid z_words length {}; expected {}",
                repr.z_words.len(),
                expected_len
            )));
        }

        Ok(Self::new(repr.params, repr.build_hasher, repr.z_words))
    }
}
