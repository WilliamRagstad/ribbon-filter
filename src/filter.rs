use std::hash::BuildHasher;

use crate::params::Params;

#[derive(Debug, Clone)]
pub struct RibbonFilter<S> {
    params: Params,
    build_hasher: S,
}

impl<S> RibbonFilter<S>
where
    S: BuildHasher + Clone,
{
    pub fn params(&self) -> Params {
        self.params
    }

    pub fn hasher(&self) -> &S {
        &self.build_hasher
    }
}
