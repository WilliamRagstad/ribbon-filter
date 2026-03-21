use crate::{Mode, ParamError, Params};
use std::collections::hash_map::DefaultHasher;
use std::hash::BuildHasherDefault;

use crate::hashing::standard_equation_w64;

type DefaultBuildHasher = BuildHasherDefault<DefaultHasher>;

#[test]
fn params_rejects_zero_m() {
    let err = Params::new(0, 4, 8, Mode::Standard).expect_err("m=0 should fail");
    assert_eq!(err, ParamError::ZeroM);
}

#[test]
fn params_rejects_zero_w() {
    let err = Params::new(10, 0, 8, Mode::Standard).expect_err("w=0 should fail");
    assert_eq!(err, ParamError::ZeroWidth);
}

#[test]
fn params_rejects_zero_r() {
    let err = Params::new(10, 4, 0, Mode::Standard).expect_err("r=0 should fail");
    assert_eq!(err, ParamError::ZeroFingerprintBits);
}

#[test]
fn params_rejects_w_greater_than_m() {
    let err = Params::new(7, 8, 8, Mode::Standard).expect_err("w>m should fail");
    assert_eq!(err, ParamError::WidthExceedsM { m: 7, w: 8 });
}

#[test]
fn params_rejects_zero_retry_limit() {
    let params = Params::new(16, 8, 8, Mode::Standard).expect("base params should be valid");
    let err = params
        .with_retry_limit(0)
        .expect_err("retry_limit=0 should fail");
    assert_eq!(err, ParamError::ZeroRetryLimit);
}

#[test]
fn params_accepts_valid_values() {
    let params = Params::new(16, 8, 12, Mode::Standard).expect("valid params should pass");
    assert_eq!(params.m, 16);
    assert_eq!(params.w, 8);
    assert_eq!(params.r, 12);
}

#[test]
fn hash_pipeline_start_in_range_and_pivot_forced() {
    let hasher = DefaultBuildHasher::default();
    let params = Params::new(128, 17, 13, Mode::Standard).expect("params must be valid");
    let mut fp = vec![0u64; params.fingerprint_words()];

    let eq = standard_equation_w64(
        &hasher,
        &"hello-key",
        42,
        params.m,
        params.w,
        &mut fp,
        params.fingerprint_last_word_mask(),
    );

    assert!(eq.start < params.start_range());
    assert_eq!(eq.coeff & 1, 1);
}

#[test]
fn hash_pipeline_masks_fingerprint_to_r_bits() {
    let hasher = DefaultBuildHasher::default();
    let params = Params::new(64, 8, 9, Mode::Standard).expect("params must be valid");
    let mut fp = vec![0u64; params.fingerprint_words()];

    let _ = standard_equation_w64(
        &hasher,
        &12345u64,
        7,
        params.m,
        params.w,
        &mut fp,
        params.fingerprint_last_word_mask(),
    );

    assert_eq!(fp[0] & !params.fingerprint_last_word_mask(), 0);
}

#[test]
fn hash_pipeline_is_deterministic_for_seed_and_key() {
    let hasher = DefaultBuildHasher::default();
    let params = Params::new(96, 16, 20, Mode::Standard).expect("params must be valid");
    let mut fp_a = vec![0u64; params.fingerprint_words()];
    let mut fp_b = vec![0u64; params.fingerprint_words()];

    let eq_a = standard_equation_w64(
        &hasher,
        &"deterministic-key",
        999,
        params.m,
        params.w,
        &mut fp_a,
        params.fingerprint_last_word_mask(),
    );
    let eq_b = standard_equation_w64(
        &hasher,
        &"deterministic-key",
        999,
        params.m,
        params.w,
        &mut fp_b,
        params.fingerprint_last_word_mask(),
    );

    assert_eq!(eq_a, eq_b);
    assert_eq!(fp_a, fp_b);
}
