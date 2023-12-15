use sha1::{Digest, Sha1};

pub fn hash_sha1(input: &[u8]) -> [u8; 20] {
    let mut hasher = Sha1::new();
    hasher.update(&input);
    hasher.finalize().into()
}
