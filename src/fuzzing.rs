//! Useful helper functions for fuzzing.

use rand::{distributions::Standard, prelude::Rng, thread_rng, SeedableRng};
use rand_chacha::ChaCha8Rng;

/// Returns a randomly seeded `ChaCha8Rng` instance, useful for making tests reproducible.
pub fn seeded_rng() -> ChaCha8Rng {
    let mut seed: <ChaCha8Rng as SeedableRng>::Seed = Default::default();
    thread_rng().fill(&mut seed);

    // We print the seed for reproducibility.
    println!("Seed for RNG: {:?}", seed);

    // Isn't cryptographically secure but adequate enough as a general source of seeded randomness.
    ChaCha8Rng::from_seed(seed)
}

/// Returns `n` random length sets of random bytes.
pub fn random_bytes(rng: &mut ChaCha8Rng, n: usize) -> Vec<Vec<u8>> {
    (0..n)
        .map(|_| {
            let random_len: usize = rng.gen_range(1..(64 * 1024));
            let random_payload: Vec<u8> = rng.sample_iter(Standard).take(random_len).collect();

            random_payload
        })
        .collect()
}
