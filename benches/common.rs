pub const FP_RATE: f64 = 0.01;
pub const QUERY_COUNT: usize = 1_000_000;

pub struct ResultRow {
    pub name: &'static str,
    pub build_us: u128,
    pub query_us: u128,
    pub bits_per_key: f64,
}
