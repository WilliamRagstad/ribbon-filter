use core::fmt;

use crate::error::ParamError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Standard,
}

impl fmt::Display for Mode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Mode::Standard => write!(f, "standard"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Params {
    pub m: usize,
    pub w: usize,
    pub r: usize,
    pub mode: Mode,
    pub seed: u64,
    pub retry_limit: usize,
}

impl Params {
    pub fn new(m: usize, w: usize, r: usize, mode: Mode) -> Result<Self, ParamError> {
        let params = Self {
            m,
            w,
            r,
            mode,
            seed: 0,
            retry_limit: 1,
        };
        params.validate()?;
        Ok(params)
    }

    pub fn with_seed(mut self, seed: u64) -> Self {
        self.seed = seed;
        self
    }

    pub fn with_retry_limit(mut self, retry_limit: usize) -> Result<Self, ParamError> {
        self.retry_limit = retry_limit;
        self.validate()?;
        Ok(self)
    }

    pub fn validate(&self) -> Result<(), ParamError> {
        if self.m == 0 || self.w == 0 || self.r == 0 || self.retry_limit == 0 {
            return Err(ParamError::Unimplemented);
        }
        if self.w > self.m {
            return Err(ParamError::Unimplemented);
        }
        Ok(())
    }

    pub fn start_range(&self) -> usize {
        self.m - self.w + 1
    }

    pub fn fingerprint_words(&self) -> usize {
        self.r.div_ceil(64)
    }

    pub fn fingerprint_last_word_mask(&self) -> u64 {
        let rem = self.r % 64;
        if rem == 0 {
            u64::MAX
        } else {
            (1u64 << rem) - 1
        }
    }
}
