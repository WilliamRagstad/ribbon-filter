use std::time::Instant;

use bloomz::BloomFilter as BloomzFilter;

use crate::common::{ResultRow, FP_RATE};

fn bloomz_bits(filter: &BloomzFilter) -> usize {
    let bytes = filter.to_bytes();
    let m_offset = bytes.len() - 12;
    let mut m_arr = [0u8; 8];
    m_arr.copy_from_slice(&bytes[m_offset..m_offset + 8]);
    u64::from_le_bytes(m_arr) as usize
}

pub fn measure(n: usize, q: usize) -> ResultRow {
    let keys: Vec<u64> = (0..n as u64).collect();

    let build_start = Instant::now();
    let mut filter = BloomzFilter::new_for_capacity(n, FP_RATE);
    for &key in &keys {
        filter.insert(&key);
    }
    let build_us = build_start.elapsed().as_micros();

    let bits_per_key = (bloomz_bits(&filter) as f64) / (n as f64);

    let query_start = Instant::now();
    let mut hits = 0usize;
    for i in 0..q as u64 {
        if filter.contains(&(10_000_000 + i)) {
            hits += 1;
        }
    }
    let query_us = query_start.elapsed().as_micros();

    let _ = hits;
    ResultRow {
        name: "bloomz",
        build_us,
        query_us,
        bits_per_key,
    }
}
