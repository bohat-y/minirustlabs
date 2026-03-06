// TODO: define a Hasher trait (update, finalize, digest) so Sha256 and Sha3
//       can share an interface — then make HMAC generic over it
// TODO: wrap sha256 in a Sha256 struct that implements Hasher
// TODO: implement own HMAC<H: Hasher> to replace the `hmac` crate in hrng

// SHA-256 round constants: first 32 bits of fractional parts of cube roots of first 64 primes
const K: [u32; 64] = [
    0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5,
    0x3956c25b, 0x59f111f1, 0x923f82a4, 0xab1c5ed5,
    0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3,
    0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174,
    0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc,
    0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da,
    0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7,
    0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967,
    0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13,
    0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85,
    0xa2bfe8a1, 0xa81a664b, 0xc24b8b70, 0xc76c51a3,
    0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070,
    0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5,
    0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
    0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208,
    0x90befffa, 0xa4506ceb, 0xbef9a3f7, 0xc67178f2,
];

// Initial hash values: first 32 bits of fractional parts of square roots of first 8 primes
const H_INIT: [u32; 8] = [
    0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a,
    0x510e527f, 0x9b05688c, 0x1f83d9ab, 0x5be0cd19,
];

fn pad(msg: &[u8]) -> Vec<u8> {
    let bit_len = (msg.len() as u64) * 8;
    let mut data = msg.to_vec();
    data.push(0x80);
    while (data.len() % 64) != 56 {
        data.push(0x00);
    }
    data.extend_from_slice(&bit_len.to_be_bytes());
    data
}

fn schedule(block: &[u8]) -> [u32; 64] {
    let mut w = [0u32; 64];

    // split into 16x32 bit words W[0] ... W[15]
    for i in 0..16 {
        w[i] = u32::from_be_bytes([
            block[i * 4],
            block[i * 4 + 1],
            block[i * 4 + 2],
            block[i * 4 + 3],
        ]);
    }

    // expand to W[16] ... W[63]
    // W(i) = Wⁱ⁻¹⁶ + σ⁰ + Wⁱ⁻⁷ + σ¹
    for i in 16..64 {
        // σ⁰ = (Wⁱ⁻¹⁵ ROTR⁷) XOR (Wⁱ⁻¹⁵ ROTR¹⁸) XOR (Wⁱ⁻¹⁵ SHR³)
        let s0 = w[i - 15].rotate_right(7) ^ w[i - 15].rotate_right(18) ^ (w[i - 15] >> 3);
        // σ¹ = (Wⁱ⁻² ROTR¹⁷) XOR (Wⁱ⁻² ROTR¹⁹) XOR (Wⁱ⁻² SHR¹⁰)
        let s1 = w[i - 2].rotate_right(17) ^ w[i - 2].rotate_right(19) ^ (w[i - 2] >> 10);
        w[i] = w[i - 16]
            .wrapping_add(s0)
            .wrapping_add(w[i - 7])
            .wrapping_add(s1);
    }

    w
}

fn compress(h: &mut [u32; 8], w: &[u32; 64]) {
    let [mut a, mut b, mut c, mut d, mut e, mut f, mut g, mut hh] = *h;

    for i in 0..64 {
        // ∑(E) = (E >>> 6) XOR (E >>> 11) XOR (E >>> 25)
        let sum_e = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
        // Ch(E, F, G) = (E AND F) XOR ((NOT E) AND G)
        let ch = (e & f) ^ (!e & g);
        let temp1 = hh
            .wrapping_add(sum_e)
            .wrapping_add(ch)
            .wrapping_add(K[i])
            .wrapping_add(w[i]);
        // ∑(A) = (A >>> 2) XOR (A >>> 13) XOR (A >>> 22)
        let sum_a = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
        // Ma(A, B, C) = (A AND B) XOR (A AND C) XOR (B AND C)
        let maj = (a & b) ^ (a & c) ^ (b & c);
        let temp2 = sum_a.wrapping_add(maj);

        hh = g;
        g = f;
        f = e;
        e = d.wrapping_add(temp1);
        d = c;
        c = b;
        b = a;
        a = temp1.wrapping_add(temp2);
    }

    h[0] = h[0].wrapping_add(a);
    h[1] = h[1].wrapping_add(b);
    h[2] = h[2].wrapping_add(c);
    h[3] = h[3].wrapping_add(d);
    h[4] = h[4].wrapping_add(e);
    h[5] = h[5].wrapping_add(f);
    h[6] = h[6].wrapping_add(g);
    h[7] = h[7].wrapping_add(hh);
}

pub fn sha256(msg: &[u8]) -> [u8; 32] {
    let data = pad(msg);
    let mut h = H_INIT;

    for block in data.chunks_exact(64) {
        let w = schedule(block);
        compress(&mut h, &w);
    }

    let mut out = [0u8; 32];
    for (i, &val) in h.iter().enumerate() {
        out[i * 4..i * 4 + 4].copy_from_slice(&val.to_be_bytes());
    }
    out
}

// HMAC-SHA-256 (RFC 2104)
// HMAC(K, m) = H((K' ^ opad) || H((K' ^ ipad) || m))
// where K' is the key padded/hashed to block size (64 bytes)
const BLOCK_SIZE: usize = 64;

pub fn hmac_sha256(key: &[u8], msg: &[u8]) -> [u8; 32] {
    // If key > block size, hash it first. Otherwise pad with zeros.
    let mut k_prime = [0u8; BLOCK_SIZE];
    if key.len() > BLOCK_SIZE {
        k_prime[..32].copy_from_slice(&sha256(key));
    } else {
        k_prime[..key.len()].copy_from_slice(key);
    }

    // inner: H((K' ^ ipad) || msg)
    let mut inner = Vec::with_capacity(BLOCK_SIZE + msg.len());
    for &b in &k_prime {
        inner.push(b ^ 0x36);
    }
    inner.extend_from_slice(msg);
    let inner_hash = sha256(&inner);

    // outer: H((K' ^ opad) || inner_hash)
    let mut outer = Vec::with_capacity(BLOCK_SIZE + 32);
    for &b in &k_prime {
        outer.push(b ^ 0x5c);
    }
    outer.extend_from_slice(&inner_hash);
    sha256(&outer)
}

pub fn hmac_sha256_hex(key: &[u8], msg: &[u8]) -> String {
    hmac_sha256(key, msg).iter().map(|b| format!("{:02x}", b)).collect()
}

pub fn sha256_hex(msg: &[u8]) -> String {
    sha256(msg).iter().map(|b| format!("{:02x}", b)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty() {
        assert_eq!(
            sha256_hex(b""),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
    }

    #[test]
    fn hello() {
        println!("{}", sha256_hex(b"hello"));
        assert_eq!(
            sha256_hex(b"hello"),
            "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824"
        );
    }

    #[test]
    fn hello_world() {
        assert_eq!(
            sha256_hex(b"hello world"),
            "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9"
        );
    }

    #[test]
    fn two_blocks() {
        // 64 bytes forces a second block after padding
        let msg = [0xABu8; 64];
        assert_eq!(sha256_hex(&msg).len(), 64);
    }

    // RFC 4231 test vectors for HMAC-SHA-256
    #[test]
    fn hmac_rfc4231_test1() {
        let key = [0x0bu8; 20];
        assert_eq!(
            hmac_sha256_hex(&key, b"Hi There"),
            "b0344c61d8db38535ca8afceaf0bf12b881dc200c9833da726e9376c2e32cff7"
        );
    }

    #[test]
    fn hmac_rfc4231_test2() {
        assert_eq!(
            hmac_sha256_hex(b"Jefe", b"what do ya want for nothing?"),
            "5bdcc146bf60754e6a042426089575c75a003f089d2739839dec58b964ec3843"
        );
    }

    #[test]
    fn hmac_long_key() {
        // key longer than block size (131 bytes) gets hashed first
        let key = [0xaau8; 131];
        let data = [0xddu8; 50];
        // just verify it produces 64 hex chars without panicking
        assert_eq!(hmac_sha256_hex(&key, &data).len(), 64);
    }
}
