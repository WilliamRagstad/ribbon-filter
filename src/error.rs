use core::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParamError {
    Unimplemented,
}

impl fmt::Display for ParamError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParamError::Unimplemented => write!(f, "parameter validation not implemented yet"),
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
