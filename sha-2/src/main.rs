use sha_2::sha256_hex;

fn main() {
    let input = std::env::args().nth(1).unwrap_or_else(|| "hello world".into());
    println!("{}", sha256_hex(input.as_bytes()));
}
