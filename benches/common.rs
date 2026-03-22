use criterion::{BenchmarkGroup, measurement::WallTime};

pub const FP_RATE: f64 = 0.01;
pub const QUERY_COUNT: usize = 1_000_000;

#[derive(Debug, Clone, Copy)]
pub struct Scenario {
    pub n: usize,
    pub w: usize,
    pub r: usize,
    pub seed: u64,
}

impl Scenario {
    pub fn id(self) -> String {
        format!("n={};w={};r={};seed={}", self.n, self.w, self.r, self.seed)
    }
}

pub type Group<'a> = BenchmarkGroup<'a, WallTime>;

pub const SCENARIOS: [Scenario; 3] = [
    Scenario {
        n: 10_000,
        w: 16,
        r: 8,
        seed: 42,
    },
    Scenario {
        n: 100_000,
        w: 16,
        r: 8,
        seed: 42,
    },
    Scenario {
        n: 100_000,
        w: 96,
        r: 10,
        seed: 777,
    },
];
