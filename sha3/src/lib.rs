// TODO: define a Hasher trait (update, finalize, digest) so Sha256 and Sha3
//       can share an interface. Then make HMAC generic over it
// TODO: wrap sha3 in a Sha3 struct that implements Hasher

// SHA-3 (Keccak) round constants: precomputed for 24 rounds of iota step
const RC: [u64; 24] = [
    0x0000000000000001, 0x0000000000008082, 0x800000000000808A,
    0x8000000080008000, 0x000000000000808B, 0x0000000080000001,
    0x8000000080008081, 0x8000000000008009, 0x000000000000008A,
    0x0000000000000088, 0x0000000080008009, 0x000000008000000A,
    0x000000008000808B, 0x800000000000008B, 0x8000000000008089,
    0x8000000000008003, 0x8000000000008002, 0x8000000000000080,
    0x000000000000800A, 0x800000008000000A, 0x8000000080008081,
    0x8000000000008080, 0x0000000080000001, 0x8000000080008008,
];

// ROT[x][y] — rotation offset for lane (x, y) in rho step
const ROT: [[u32; 5]; 5] = [
    [ 0, 36,  3, 41, 18],
    [ 1, 44, 10, 45,  2],
    [62,  6, 43, 15, 61],
    [28, 55, 25, 21, 56],
    [27, 20, 39,  8, 14],
];

type State = [[u64; 5]; 5];

fn keccak_f(state: &mut State) {
    for round in 0..24 {
        theta(state);
        rho(state);
        pi(state);
        chi(state);
        iota(state, round);
    }
}

// θ: C[x] = A[x,0..4], D[x] = C[x-1] ^ rot(C[x+1], 1), A[x,y] ^= D[x]
fn theta(state: &mut State) {
    let mut c = [0u64; 5];
    for x in 0..5 {
        c[x] = state[x][0] ^ state[x][1] ^ state[x][2] ^ state[x][3] ^ state[x][4];
    }
    let mut d = [0u64; 5];
    for x in 0..5 {
        d[x] = c[(x + 4) % 5] ^ c[(x + 1) % 5].rotate_left(1);
    }
    for x in 0..5 {
        for y in 0..5 {
            state[x][y] ^= d[x];
        }
    }
}

// ρ: rotate each lane by its fixed offset
fn rho(state: &mut State) {
    for x in 0..5 {
        for y in 0..5 {
            state[x][y] = state[x][y].rotate_left(ROT[x][y]);
        }
    }
}

// π: B[y, 2x+3y mod 5] = A[x, y]
fn pi(state: &mut State) {
    let old = *state;
    for x in 0..5 {
        for y in 0..5 {
            state[y][(2 * x + 3 * y) % 5] = old[x][y];
        }
    }
}

// χ: A[x,y] = A[x,y] ^ (!A[x+1,y] & A[x+2,y])
fn chi(state: &mut State) {
    let old = *state;
    for x in 0..5 {
        for y in 0..5 {
            state[x][y] = old[x][y] ^ (!old[(x + 1) % 5][y] & old[(x + 2) % 5][y]);
        }
    }
}

// ι: A[0,0] ^= RC[round]
fn iota(state: &mut State, round: usize) {
    state[0][0] ^= RC[round];
}

fn state_to_bytes(state: &State) -> [u8; 200] {
    let mut out = [0u8; 200];
    for x in 0..5 {
        for y in 0..5 {
            let offset = 8 * (5 * y + x);
            out[offset..offset + 8].copy_from_slice(&state[x][y].to_le_bytes());
        }
    }
    out
}

fn bytes_to_state(bytes: &[u8]) -> State {
    let mut state = [[0u64; 5]; 5];
    for x in 0..5 {
        for y in 0..5 {
            let offset = 8 * (5 * y + x);
            state[x][y] = u64::from_le_bytes(bytes[offset..offset + 8].try_into().unwrap());
        }
    }
    state
}

fn xor_block(state: &mut State, block: &[u8], rate_bytes: usize) {
    let mut state_bytes = state_to_bytes(state);
    for i in 0..rate_bytes.min(block.len()) {
        state_bytes[i] ^= block[i];
    }
    *state = bytes_to_state(&state_bytes);
}

// pad10*1: append domain byte, zeros, then 0x80
// SHA-3 domain = 0x06, SHAKE domain = 0x1F
fn pad(msg: &[u8], rate_bytes: usize, domain: u8) -> Vec<u8> {
    let mut data = msg.to_vec();
    data.push(domain);
    while data.len() % rate_bytes != rate_bytes - 1 {
        data.push(0x00);
    }
    data.push(0x80);
    data
}

fn sponge(msg: &[u8], rate_bytes: usize, output_len: usize, domain: u8) -> Vec<u8> {
    let data = pad(msg, rate_bytes, domain);
    let mut state: State = [[0u64; 5]; 5];

    // absorb
    for block in data.chunks(rate_bytes) {
        xor_block(&mut state, block, rate_bytes);
        keccak_f(&mut state);
    }

    // squeeze
    let mut hash = Vec::with_capacity(output_len);
    while hash.len() < output_len {
        let state_bytes = state_to_bytes(&state);
        let take = rate_bytes.min(output_len - hash.len());
        hash.extend_from_slice(&state_bytes[..take]);
        if hash.len() < output_len {
            keccak_f(&mut state);
        }
    }
    hash.truncate(output_len);
    hash
}

// SHA3-224: r=1152 (144 bytes), output=28 bytes
pub fn sha3_224(msg: &[u8]) -> [u8; 28] {
    sponge(msg, 144, 28, 0x06).try_into().unwrap()
}

// SHA3-256: r=1088 (136 bytes), output=32 bytes
pub fn sha3_256(msg: &[u8]) -> [u8; 32] {
    sponge(msg, 136, 32, 0x06).try_into().unwrap()
}

// SHA3-384: r=832 (104 bytes), output=48 bytes
pub fn sha3_384(msg: &[u8]) -> [u8; 48] {
    sponge(msg, 104, 48, 0x06).try_into().unwrap()
}

// SHA3-512: r=576 (72 bytes), output=64 bytes
pub fn sha3_512(msg: &[u8]) -> [u8; 64] {
    sponge(msg, 72, 64, 0x06).try_into().unwrap()
}

// SHAKE128: r=1344 (168 bytes), extendable output
pub fn shake128(msg: &[u8], output_len: usize) -> Vec<u8> {
    sponge(msg, 168, output_len, 0x1F)
}

// SHAKE256: r=1088 (136 bytes), extendable output
pub fn shake256(msg: &[u8], output_len: usize) -> Vec<u8> {
    sponge(msg, 136, output_len, 0x1F)
}

pub fn sha3_256_hex(msg: &[u8]) -> String {
    sha3_256(msg).iter().map(|b| format!("{:02x}", b)).collect()
}

pub fn sha3_512_hex(msg: &[u8]) -> String {
    sha3_512(msg).iter().map(|b| format!("{:02x}", b)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hex(bytes: &[u8]) -> String {
        bytes.iter().map(|b| format!("{:02x}", b)).collect()
    }

    #[test]
    fn empty() {
        assert_eq!(
            sha3_256_hex(b""),
            "a7ffc6f8bf1ed76651c14756a061d662f580ff4de43b49fa82d80a4b80f8434a"
        );
    }

    #[test]
    fn abc() {
        assert_eq!(
            sha3_256_hex(b"abc"),
            "3a985da74fe225b2045c172d6bd390bd855f086e3e9d525b46bfe24511431532"
        );
    }

    #[test]
    fn two_blocks() {
        let input = b"abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq";
        assert_eq!(
            sha3_256_hex(input),
            "41c0dba2a9d6240849100376a8235e2c82e1b9998a999e21db32dd97496d3376"
        );
    }

    #[test]
    fn sha3_512_empty() {
        assert_eq!(
            sha3_512_hex(b""),
            "a69f73cca23a9ac5c8b567dc185a756e97c982164fe25859e0d1dcc1475c80a6\
             15b2123af1f5f94c11e3e9402c3ac558f500199d95b6d3e301758586281dcd26"
        );
    }

    #[test]
    fn sha3_512_abc() {
        assert_eq!(
            sha3_512_hex(b"abc"),
            "b751850b1a57168a5693cd924b6b096e08f621827444f70d884f5d0240d2712e\
             10e116e9192af3c91a7ec57647e3934057340b4cf408d5a56592f8274eec53f0"
        );
    }

    #[test]
    fn sha3_224_empty() {
        assert_eq!(
            hex(&sha3_224(b"")),
            "6b4e03423667dbb73b6e15454f0eb1abd4597f9a1b078e3f5b5a6bc7"
        );
    }

    #[test]
    fn sha3_384_empty() {
        assert_eq!(
            hex(&sha3_384(b"")),
            "0c63a75b845e4f7d01107d852e4c2485c51a50aaaa94fc61995e71bbee983a2a\
             c3713831264adb47fb6bd1e058d5f004"
        );
    }

    #[test]
    fn shake128_empty() {
        assert_eq!(
            hex(&shake128(b"", 32)),
            "7f9c2ba4e88f827d616045507605853ed73b8093f6efbc88eb1a6eacfa66ef26"
        );
    }

    #[test]
    fn shake256_empty() {
        assert_eq!(
            hex(&shake256(b"", 32)),
            "46b9dd2b0ba88d13233b3feb743eeb243fcd52ea62b81b82b50c27646ed5762f"
        );
    }
}
