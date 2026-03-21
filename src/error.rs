use core::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParamError {
    ZeroM,
    ZeroWidth,
    ZeroFingerprintBits,
    WidthExceedsM { m: usize, w: usize },
    ZeroRetryLimit,
}

impl fmt::Display for ParamError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParamError::ZeroM => write!(f, "m must be greater than zero"),
            ParamError::ZeroWidth => write!(f, "w must be greater than zero"),
            ParamError::ZeroFingerprintBits => write!(f, "r must be greater than zero"),
            ParamError::WidthExceedsM { m, w } => {
                write!(f, "w ({w}) must be less than or equal to m ({m})")
            }
            ParamError::ZeroRetryLimit => write!(f, "retry_limit must be greater than zero"),
        }
    }
}

impl std::error::Error for ParamError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConstructionFailure {
    InconsistentEquation { key_index: usize, row_index: usize },
}

impl fmt::Display for ConstructionFailure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConstructionFailure::InconsistentEquation { key_index, row_index } => write!(
                f,
                "inconsistent equation while inserting key at index {key_index} near row {row_index}"
            ),
        }
    }
}

impl std::error::Error for ConstructionFailure {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BuildError {
    InvalidParams(ParamError),
    ConstructionFailed {
        final_m: usize,
        attempts: usize,
        last_failure: ConstructionFailure,
    },
}

impl fmt::Display for BuildError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BuildError::InvalidParams(err) => write!(f, "invalid parameters: {err}"),
            BuildError::ConstructionFailed {
                final_m,
                attempts,
                last_failure,
            } => write!(
                f,
                "construction failed after {attempts} attempt(s) at m={final_m}: {last_failure}"
            ),
        }
    }
}

impl std::error::Error for BuildError {}
