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
    Unimplemented,
}

impl fmt::Display for ConstructionFailure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConstructionFailure::Unimplemented => {
                write!(f, "construction failure details not implemented yet")
            }
        }
    }
}

impl std::error::Error for ConstructionFailure {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BuildError {
    InvalidParams(ParamError),
    ConstructionFailed(ConstructionFailure),
}

impl fmt::Display for BuildError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BuildError::InvalidParams(err) => write!(f, "invalid parameters: {err}"),
            BuildError::ConstructionFailed(err) => write!(f, "construction failed: {err}"),
        }
    }
}

impl std::error::Error for BuildError {}
