use crate::{Mode, ParamError, Params};

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
