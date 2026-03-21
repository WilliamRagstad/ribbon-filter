#![no_main]

use libfuzzer_sys::fuzz_target;
use ribbon_filter::{Mode, Params, RibbonBuilder};
use std::collections::hash_map::DefaultHasher;
use std::hash::BuildHasherDefault;

type H = BuildHasherDefault<DefaultHasher>;

fn read_u64(data: &[u8], index: usize) -> u64 {
    let mut bytes = [0u8; 8];
    if data.len() >= index + 8 {
        bytes.copy_from_slice(&data[index..index + 8]);
    }
    u64::from_le_bytes(bytes)
}

fuzz_target!(|data: &[u8]| {
    if data.len() < 32 {
        return;
    }

    let mode = if (data[0] & 1) == 0 {
        Mode::Standard
    } else {
        Mode::Homogeneous
    };

    let w_choices = [1usize, 2, 8, 16, 32, 64, 96, 128];
    let w = w_choices[(data[1] as usize) % w_choices.len()];
    let r_choices = [1usize, 2, 8, 12, 16, 24];
    let r = r_choices[(data[2] as usize) % r_choices.len()];

    let n = 1 + (data[3] as usize % 256);
    let base_m = n + (data[4] as usize % (n + 1));
    let m = base_m.max(w);
    let retry_limit = 1 + (data[5] as usize % 4);
    let grow_limit = data[6] as usize % 3;
    let seed = read_u64(data, 8);

    let params = match Params::new(m, w, r, mode) {
        Ok(p) => p,
        Err(_) => return,
    };
    let params = match params.with_retry_policy(retry_limit, grow_limit) {
        Ok(p) => p.with_seed(seed),
        Err(_) => return,
    };

    let mut keys = Vec::with_capacity(n);
    let mut state = read_u64(data, 16) ^ 0x9E37_79B9_7F4A_7C15;
    for _ in 0..n {
        state = state
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        keys.push(state);
    }

    let builder = match RibbonBuilder::new(params, H::default()) {
        Ok(b) => b,
        Err(_) => return,
    };

    if let Ok(filter) = builder.build(&keys) {
        let mut scratch = filter.new_scratch();
        for key in keys.iter().take(16) {
            let _ = filter.contains_in(key, &mut scratch);
        }

        let q = read_u64(data, 24);
        let _ = filter.contains_in(&q, &mut scratch);
    }
});
