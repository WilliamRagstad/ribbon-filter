#![no_main]

use libfuzzer_sys::arbitrary;
use libfuzzer_sys::arbitrary::Arbitrary;
use libfuzzer_sys::fuzz_target;
use ribbon_filter::{Mode, Params, RibbonBuilder};
use std::collections::hash_map::DefaultHasher;
use std::hash::BuildHasherDefault;

type H = BuildHasherDefault<DefaultHasher>;

#[derive(Arbitrary, Debug)]
struct StructuredInput {
    mode_bit: bool,
    w_index: u8,
    r_index: u8,
    n_raw: u16,
    m_slack_raw: u16,
    retry_raw: u8,
    grow_raw: u8,
    seed: u64,
    key_seed: u64,
    keys: Vec<u64>,
    queries: Vec<u64>,
}

fn generate_keys(input: &StructuredInput, n: usize) -> Vec<u64> {
    let mut out = Vec::with_capacity(n);
    let mut state = input.key_seed ^ 0x9E37_79B9_7F4A_7C15;

    if input.keys.is_empty() {
        for _ in 0..n {
            state = state
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            out.push(state);
        }
        return out;
    }

    for i in 0..n {
        state = state
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        let base = input.keys[i % input.keys.len()];
        out.push(base ^ state.rotate_left((i % 63) as u32));
    }

    out
}

fuzz_target!(|input: StructuredInput| {
    let mode = if input.mode_bit {
        Mode::Homogeneous
    } else {
        Mode::Standard
    };

    let w_choices = [1usize, 2, 8, 16, 32, 64, 96, 128];
    let r_choices = [1usize, 2, 8, 12, 16, 24];
    let w = w_choices[(input.w_index as usize) % w_choices.len()];
    let r = r_choices[(input.r_index as usize) % r_choices.len()];

    let n = 1 + ((input.n_raw as usize) % 512);
    let base_m = n + ((input.m_slack_raw as usize) % (n + 1));
    let m = base_m.max(w);
    let retry_limit = 1 + (input.retry_raw as usize % 4);
    let grow_limit = input.grow_raw as usize % 3;

    let params = match Params::new(m, w, r, mode) {
        Ok(p) => p,
        Err(_) => return,
    };
    let params = match params.with_retry_policy(retry_limit, grow_limit) {
        Ok(p) => p.with_seed(input.seed),
        Err(_) => return,
    };

    let keys = generate_keys(&input, n);

    let builder = match RibbonBuilder::new(params, H::default()) {
        Ok(b) => b,
        Err(_) => return,
    };

    if let Ok(filter) = builder.build(&keys) {
        let mut scratch = filter.new_scratch();

        for key in keys.iter().take(32) {
            assert!(filter.contains_in(key, &mut scratch));
        }

        for query in input.queries.iter().take(16) {
            let a = filter.contains(query);
            let b = filter.contains_in(query, &mut scratch);
            assert_eq!(a, b);
        }
    }
});
