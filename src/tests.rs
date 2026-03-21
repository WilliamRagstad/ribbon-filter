use crate::{BuildError, Mode, ParamError, Params, RibbonBuilder};
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
fn params_rejects_zero_n_in_expected_items() {
    let err =
        Params::from_expected_items(0, 0.1, 4, 8, Mode::Standard).expect_err("n=0 should fail");
    assert_eq!(err, ParamError::ZeroN);
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
fn params_r_from_fpr_rounding_and_range() {
    assert_eq!(Params::r_from_fpr(0.5).expect("valid fpr"), 1);
    assert_eq!(Params::r_from_fpr(0.1).expect("valid fpr"), 4);
    assert!(matches!(
        Params::r_from_fpr(0.0),
        Err(ParamError::InvalidFalsePositiveRate { .. })
    ));
}

#[test]
fn params_from_expected_items_computes_m() {
    let p = Params::from_expected_items(1000, 0.2, 16, 8, Mode::Standard)
        .expect("params should be valid");
    assert_eq!(p.m, 1200);
    assert_eq!(p.w, 16);
    assert_eq!(p.r, 8);
}

#[test]
fn params_from_expected_items_rejects_overhead_out_of_range() {
    let err = Params::from_expected_items(1000, 10.1, 16, 8, Mode::Standard)
        .expect_err("overhead > 10 should fail");
    assert!(matches!(err, ParamError::InvalidOverhead { .. }));
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
        Mode::Standard,
        &mut fp,
        params.fingerprint_last_word_mask(),
    );

    assert!(eq.start < params.start_range());
    assert_eq!(eq.coeff_lo & 1, 1);
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
        Mode::Standard,
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
        Mode::Standard,
        &mut fp_a,
        params.fingerprint_last_word_mask(),
    );
    let eq_b = standard_equation_w64(
        &hasher,
        &"deterministic-key",
        999,
        params.m,
        params.w,
        Mode::Standard,
        &mut fp_b,
        params.fingerprint_last_word_mask(),
    );

    assert_eq!(eq_a, eq_b);
    assert_eq!(fp_a, fp_b);
}

#[test]
fn standard_builder_has_no_false_negatives_1k() {
    let hasher = DefaultBuildHasher::default();
    let params = Params::new(3000, 16, 12, Mode::Standard).expect("params should be valid");
    let builder = RibbonBuilder::new(params.with_seed(11), hasher).expect("builder should build");

    let keys: Vec<u64> = (0..1000).collect();
    let filter = builder.build(&keys).expect("construction should succeed");

    for key in &keys {
        assert!(filter.contains(key), "false negative for key {key}");
    }
}

#[test]
fn standard_builder_has_no_false_negatives_10k() {
    let hasher = DefaultBuildHasher::default();
    let params = Params::new(30000, 16, 10, Mode::Standard).expect("params should be valid");
    let builder = RibbonBuilder::new(params.with_seed(13), hasher).expect("builder should build");

    let keys: Vec<u64> = (0..10000).collect();
    let filter = builder.build(&keys).expect("construction should succeed");

    for key in &keys {
        assert!(filter.contains(key), "false negative for key {key}");
    }
}

#[test]
fn standard_builder_reports_inconsistent_equation_failure() {
    let hasher = DefaultBuildHasher::default();
    let params = Params::new(16, 16, 8, Mode::Standard)
        .expect("params should be valid")
        .with_seed(5);
    let builder = RibbonBuilder::new(params, hasher).expect("builder should build");

    let keys: Vec<u64> = (0..200).collect();
    let result = builder.build(&keys);

    match result {
        Err(BuildError::ConstructionFailed { last_failure, .. }) => {
            assert!(matches!(
                last_failure,
                crate::ConstructionFailure::InconsistentEquation { .. }
            ));
        }
        Err(other) => panic!("expected construction failure, got {other}"),
        Ok(_) => panic!("expected failure, got success"),
    }
}

#[test]
fn standard_builder_is_deterministic_for_same_input() {
    let hasher = DefaultBuildHasher::default();
    let params = Params::new(3000, 16, 9, Mode::Standard)
        .expect("params should be valid")
        .with_seed(99);

    let builder_a = RibbonBuilder::new(params, hasher.clone()).expect("builder should build");
    let builder_b = RibbonBuilder::new(params, hasher).expect("builder should build");

    let keys: Vec<u64> = (1000..2000).collect();
    let filter_a = builder_a.build(&keys).expect("first build should succeed");
    let filter_b = builder_b.build(&keys).expect("second build should succeed");

    for probe in 990..2010u64 {
        assert_eq!(
            filter_a.contains(&probe),
            filter_b.contains(&probe),
            "non-deterministic result for key {probe}"
        );
    }
}

#[derive(Default, Clone)]
struct ConstantBuildHasher;

impl std::hash::BuildHasher for ConstantBuildHasher {
    type Hasher = ConstantHasher;

    fn build_hasher(&self) -> Self::Hasher {
        ConstantHasher::default()
    }
}

#[derive(Default, Clone)]
struct ConstantHasher;

impl std::hash::Hasher for ConstantHasher {
    fn finish(&self) -> u64 {
        0
    }

    fn write(&mut self, _bytes: &[u8]) {}
}

#[test]
fn builder_supports_custom_buildhasher() {
    let hasher = ConstantBuildHasher;
    let params = Params::new(3000, 16, 9, Mode::Standard)
        .expect("params should be valid")
        .with_seed(88);
    let builder = RibbonBuilder::new(params, hasher).expect("builder should build");

    let keys: Vec<u64> = (0..200).collect();
    let filter = builder.build(&keys).expect("build should succeed");

    let mut scratch = filter.new_scratch();
    for key in &keys {
        assert!(filter.contains_in(key, &mut scratch));
    }
}

#[test]
fn contains_and_contains_in_are_equivalent() {
    let hasher = DefaultBuildHasher::default();
    let params = Params::new(3000, 16, 9, Mode::Standard)
        .expect("params should be valid")
        .with_seed(77);
    let builder = RibbonBuilder::new(params, hasher).expect("builder should build");
    let keys: Vec<u64> = (1000..2000).collect();
    let filter = builder.build(&keys).expect("build should succeed");

    let mut scratch = filter.new_scratch();
    for probe in 900..2100u64 {
        assert_eq!(
            filter.contains(&probe),
            filter.contains_in(&probe, &mut scratch),
            "contains mismatch at key {probe}"
        );
    }
}

#[test]
fn retry_path_is_exercised_and_eventually_succeeds() {
    let hasher = DefaultBuildHasher::default();
    let keys: Vec<u64> = (0..500).collect();
    let params = Params::new(16, 16, 8, Mode::Standard)
        .expect("params valid")
        .with_seed(1)
        .with_retry_policy(3, 0)
        .expect("retry policy valid");
    let builder = RibbonBuilder::new(params, hasher).expect("builder valid");

    match builder.build(&keys) {
        Err(BuildError::ConstructionFailed {
            final_m,
            attempts,
            last_failure,
        }) => {
            assert_eq!(final_m, 16);
            assert_eq!(attempts, 3);
            assert!(matches!(
                last_failure,
                crate::ConstructionFailure::InconsistentEquation { .. }
            ));
        }
        other => panic!("expected retry-exhausted failure, got {other:?}"),
    }
}

#[test]
fn growth_path_is_exercised_and_reports_grown_m() {
    let hasher = DefaultBuildHasher::default();
    let keys: Vec<u64> = (0..500).collect();
    let params = Params::new(16, 16, 8, Mode::Standard)
        .expect("params valid")
        .with_seed(1)
        .with_retry_policy(2, 2)
        .expect("retry policy valid");
    let builder = RibbonBuilder::new(params, hasher).expect("builder valid");

    match builder.build(&keys) {
        Err(BuildError::ConstructionFailed {
            final_m,
            attempts,
            last_failure,
        }) => {
            assert_eq!(attempts, 6);
            assert_eq!(final_m, 19);
            assert!(matches!(
                last_failure,
                crate::ConstructionFailure::InconsistentEquation { .. }
            ));
        }
        other => panic!("expected growth-exhausted failure, got {other:?}"),
    }
}

#[test]
fn terminal_failure_reports_attempts_and_final_m() {
    let hasher = DefaultBuildHasher::default();
    let params = Params::new(16, 16, 8, Mode::Standard)
        .expect("params valid")
        .with_seed(1)
        .with_retry_policy(2, 2)
        .expect("retry policy valid");
    let builder = RibbonBuilder::new(params, hasher).expect("builder valid");
    let keys: Vec<u64> = (0..500).collect();

    match builder.build(&keys) {
        Err(BuildError::ConstructionFailed {
            final_m,
            attempts,
            last_failure,
        }) => {
            assert_eq!(attempts, 6);
            assert_eq!(final_m, 19);
            assert!(matches!(
                last_failure,
                crate::ConstructionFailure::InconsistentEquation { .. }
            ));
        }
        other => panic!("expected terminal construction failure, got {other:?}"),
    }
}

#[test]
fn successful_build_persists_selected_attempt_seed() {
    let hasher = DefaultBuildHasher::default();
    let base_seed = 123u64;
    let params = Params::new(3000, 16, 9, Mode::Standard)
        .expect("params valid")
        .with_seed(base_seed)
        .with_retry_policy(1, 0)
        .expect("retry policy valid");
    let builder = RibbonBuilder::new(params, hasher).expect("builder valid");
    let keys: Vec<u64> = (0..1000).collect();

    let filter = builder.build(&keys).expect("build should succeed");
    assert_eq!(
        filter.params().seed,
        crate::hashing::derive_attempt_seed(base_seed, 0)
    );
}

#[test]
fn homogeneous_build_succeeds_and_has_no_false_negatives() {
    let hasher = DefaultBuildHasher::default();
    let params = Params::new(4000, 16, 8, Mode::Homogeneous)
        .expect("params valid")
        .with_seed(55);
    let builder = RibbonBuilder::new(params, hasher).expect("builder valid");
    let keys: Vec<u64> = (0..1000).collect();

    let filter = builder
        .build(&keys)
        .expect("homogeneous build should succeed");
    let mut scratch = filter.new_scratch();
    for key in &keys {
        assert!(filter.contains_in(key, &mut scratch));
    }
}

#[test]
fn homogeneous_pipeline_has_zero_fingerprint() {
    let hasher = DefaultBuildHasher::default();
    let params = Params::new(128, 16, 9, Mode::Homogeneous).expect("params must be valid");
    let mut fp = vec![0u64; params.fingerprint_words()];

    let _ = standard_equation_w64(
        &hasher,
        &"h-key",
        11,
        params.m,
        params.w,
        Mode::Homogeneous,
        &mut fp,
        params.fingerprint_last_word_mask(),
    );

    assert!(fp.iter().all(|&w| w == 0));
}

#[test]
fn width_128_pipeline_sets_bits_in_both_halves() {
    let hasher = DefaultBuildHasher::default();
    let params = Params::new(400, 128, 8, Mode::Standard).expect("params valid");

    let mut saw_hi = false;
    for seed in 0..500u64 {
        let mut fp = vec![0u64; params.fingerprint_words()];
        let eq = standard_equation_w64(
            &hasher,
            &"w128-key",
            seed,
            params.m,
            params.w,
            Mode::Standard,
            &mut fp,
            params.fingerprint_last_word_mask(),
        );

        if eq.coeff_hi != 0 {
            saw_hi = true;
            break;
        }
    }

    assert!(
        saw_hi,
        "expected at least one seed with high-half coefficient bits"
    );
}

#[test]
fn builder_supports_width_above_64() {
    let hasher = DefaultBuildHasher::default();
    let params = Params::new(4000, 96, 10, Mode::Standard)
        .expect("params should be valid")
        .with_seed(303)
        .with_retry_policy(4, 1)
        .expect("retry policy valid");
    let builder = RibbonBuilder::new(params, hasher).expect("builder should build");
    let keys: Vec<u64> = (0..800).collect();
    let filter = builder.build(&keys).expect("construction should succeed");
    let mut scratch = filter.new_scratch();

    for key in &keys {
        assert!(filter.contains_in(key, &mut scratch));
    }
}

#[test]
fn bitpacked_storage_maintains_membership_behavior() {
    let hasher = DefaultBuildHasher::default();
    let params = Params::new(3000, 16, 12, Mode::Standard)
        .expect("params should be valid")
        .with_seed(1234);
    let builder = RibbonBuilder::new(params, hasher).expect("builder should build");
    let keys: Vec<u64> = (0..1000).collect();
    let filter = builder.build(&keys).expect("build should succeed");

    let mut scratch = filter.new_scratch();
    for key in &keys {
        assert!(filter.contains_in(key, &mut scratch));
    }
}
