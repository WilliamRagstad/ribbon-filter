use core::hash::{Hash, Hasher};
use std::hash::BuildHasher;

use crate::params::Mode;

const MIX_CONST: u64 = 0x9E37_79B9_7F4A_7C15;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct StandardEquation {
    pub(crate) start: usize,
    pub(crate) coeff: u64,
}

#[inline]
pub(crate) fn xor_words(dst: &mut [u64], rhs: &[u64]) {
    for (d, r) in dst.iter_mut().zip(rhs.iter()) {
        *d ^= *r;
    }
}

#[inline]
pub(crate) fn for_each_set_bit_u64(mut mask: u64, mut f: impl FnMut(usize)) {
    while mask != 0 {
        let bit = mask.trailing_zeros() as usize;
        f(bit);
        mask &= mask - 1;
    }
}

#[derive(Debug, Clone, Copy)]
struct SplitMix64 {
    state: u64,
}

impl SplitMix64 {
    fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(MIX_CONST);
        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }
}

fn hash64<S: BuildHasher, Q: Hash + ?Sized>(build_hasher: &S, key: &Q) -> u64 {
    let mut hasher = build_hasher.build_hasher();
    key.hash(&mut hasher);
    hasher.finish()
}

fn fastrange_u64(x: u64, range: usize) -> usize {
    ((x as u128 * range as u128) >> 64) as usize
}

pub(crate) fn derive_attempt_seed(base_seed: u64, attempt_index: u64) -> u64 {
    let mut sm = SplitMix64::new(base_seed ^ attempt_index.wrapping_mul(MIX_CONST));
    sm.next_u64().wrapping_mul(MIX_CONST)
}

pub(crate) fn standard_equation_w64<S: BuildHasher, Q: Hash + ?Sized>(
    build_hasher: &S,
    key: &Q,
    seed: u64,
    m: usize,
    w: usize,
    mode: Mode,
    fingerprint: &mut [u64],
    last_word_mask: u64,
) -> StandardEquation {
    let base_hash = hash64(build_hasher, key);
    let stream_seed = (base_hash ^ seed).wrapping_mul(MIX_CONST);
    let mut stream = SplitMix64::new(stream_seed);

    let start = fastrange_u64(stream.next_u64(), m - w + 1);

    let width_mask = if w == 64 { u64::MAX } else { (1u64 << w) - 1 };
    let coeff = (stream.next_u64() & width_mask) | 1;

    if matches!(mode, Mode::Homogeneous) {
        fingerprint.fill(0);
    } else {
        for word in fingerprint.iter_mut() {
            *word = stream.next_u64();
        }
        if let Some(last) = fingerprint.last_mut() {
            *last &= last_word_mask;
        }
    }

    StandardEquation { start, coeff }
}
