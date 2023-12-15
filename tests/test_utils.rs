use bittorrent_starter_rust::utils::hash_sha1;
use hex::ToHex;

#[test]
fn test_hash_sha1() {
    fn check(input: &[u8], expected: &str) {
        assert_eq!(hash_sha1(input).encode_hex::<String>(), expected);
    }

    check(&[], "da39a3ee5e6b4b0d3255bfef95601890afd80709");
    check(b"hello", "aaf4c61ddcc5e8a2dabede0f3b482cd9aea9434d");
}
