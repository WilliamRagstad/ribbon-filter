use std::hash::BuildHasher;

use crate::error::BuildError;
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
}
