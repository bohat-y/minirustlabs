use hmac::{Hmac, Mac};
use sha2::Sha256;
use std::time::Instant;

type HmacSha256 = Hmac<Sha256>;

// Collect at least this many raw bytes before first DRBG seed.
const MIN_SEED_BYTES: usize = 256;
// Reseed the DRBG after this many generate() calls.
const RESEED_INTERVAL: u64 = 100;
// Cap the pool to avoid unbounded growth between reseeds.
const MAX_POOL_BYTES: usize = 8192;

// ---------------------------------------------------------------------------
// HMAC-DRBG (NIST SP 800-90A, Section 10.1.2) with HMAC-SHA-256.
// ---------------------------------------------------------------------------

struct HmacDrbg {
    key: [u8; 32],
    v: [u8; 32],
    reseed_counter: u64,
}

impl HmacDrbg {
    fn instantiate(seed: &[u8]) -> Self {
        let mut drbg = Self {
            key: [0x00; 32],
            v: [0x01; 32],
            reseed_counter: 0,
        };
        drbg.update(seed);
        drbg.reseed_counter = 1;
        drbg
    }

    fn reseed(&mut self, entropy: &[u8]) {
        self.update(entropy);
        self.reseed_counter = 1;
    }

    fn generate(&mut self, out: &mut [u8]) {
        let mut pos = 0;
        while pos < out.len() {
            self.v = hmac_sha256(&self.key, &self.v);
            let n = (out.len() - pos).min(32);
            out[pos..pos + n].copy_from_slice(&self.v[..n]);
            pos += n;
        }
        self.update(&[]);
        self.reseed_counter += 1;
    }

    fn needs_reseed(&self) -> bool {
        self.reseed_counter > RESEED_INTERVAL
    }

    // NIST SP 800-90A Section 10.1.2.2 — HMAC_DRBG_Update.
    fn update(&mut self, data: &[u8]) {
        // Step 1: Key = HMAC(Key, V || 0x00 || data)
        self.key = hmac_sha256_concat(&self.key, &self.v, 0x00, data);
        // Step 2: V = HMAC(Key, V)
        self.v = hmac_sha256(&self.key, &self.v);

        if !data.is_empty() {
            // Step 4: Key = HMAC(Key, V || 0x01 || data)
            self.key = hmac_sha256_concat(&self.key, &self.v, 0x01, data);
            // Step 5: V = HMAC(Key, V)
            self.v = hmac_sha256(&self.key, &self.v);
        }
    }
}

fn hmac_sha256(key: &[u8; 32], data: &[u8]) -> [u8; 32] {
    let mut mac = HmacSha256::new_from_slice(key).unwrap();
    mac.update(data);
    mac.finalize().into_bytes().into()
}

/// HMAC(key, v || separator || data) — avoids allocating a concat buffer.
fn hmac_sha256_concat(key: &[u8; 32], v: &[u8; 32], sep: u8, data: &[u8]) -> [u8; 32] {
    let mut mac = HmacSha256::new_from_slice(key).unwrap();
    mac.update(v);
    mac.update(&[sep]);
    mac.update(data);
    mac.finalize().into_bytes().into()
}

// ---------------------------------------------------------------------------
// Entropy pool — collects raw entropy and drives the DRBG.
// ---------------------------------------------------------------------------

pub struct EntropyPool {
    pool: Vec<u8>,
    drbg: Option<HmacDrbg>,
    last_ts: Instant,
    total_collected: usize,
}

impl EntropyPool {
    pub fn new() -> Self {
        Self {
            pool: Vec::with_capacity(4096),
            drbg: None,
            last_ts: Instant::now(),
            total_collected: 0,
        }
    }

    /// Extract LSBs from camera frame - sensor noise lives in the low bits.
    /// Takes the lowest 2 bits of every 4th byte (one channel per pixel)
    /// to avoid correlation between adjacent BGRA channels.
    pub fn feed_camera(&mut self, frame: &[u8]) {
        if self.pool.len() >= MAX_POOL_BYTES { return; }

        let mut acc: u8 = 0;
        let mut bits: u8 = 0;

        let before = self.pool.len();
        for &byte in frame.iter().step_by(4) {
            if self.pool.len() >= MAX_POOL_BYTES { break; }
            // Pack 2 LSBs per sample, 4 samples per output byte.
            acc = (acc << 2) | (byte & 0b11);
            bits += 2;
            if bits == 8 {
                self.pool.push(acc);
                acc = 0;
                bits = 0;
            }
        }
        self.total_collected += self.pool.len() - before;
    }

    /// Capture timing jitter - low-order nanoseconds of inter-frame delta.
    pub fn feed_timing(&mut self) {
        let now = Instant::now();
        let nanos = now.duration_since(self.last_ts).as_nanos() as u64;
        self.last_ts = now;
        // Only the low bytes carry meaningful jitter.
        self.pool.extend_from_slice(&nanos.to_le_bytes()[..4]);
    }

    /// Try to produce `out.len()` random bytes. Returns false if not enough
    /// entropy has been collected yet to seed the DRBG.
    pub fn generate(&mut self, out: &mut [u8]) -> bool {
        // First seed.
        if self.drbg.is_none() {
            if self.pool.len() < MIN_SEED_BYTES {
                return false;
            }
            self.drbg = Some(HmacDrbg::instantiate(&self.pool));
            self.pool.clear();
            self.pool.shrink_to(4096);
        }

        let drbg = self.drbg.as_mut().unwrap();

        // Reseed periodically with fresh entropy from the pool.
        if drbg.needs_reseed() && !self.pool.is_empty() {
            drbg.reseed(&self.pool);
            self.pool.clear();
            self.pool.shrink_to(4096);
        }

        drbg.generate(out);
        true
    }

}
