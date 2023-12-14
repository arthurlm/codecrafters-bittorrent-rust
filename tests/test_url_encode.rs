use bittorrent_starter_rust::url_encode::url_encode;

#[test]
fn test_url_encode() {
    assert_eq!(url_encode(&[]), "");
    assert_eq!(url_encode(b"hello"), "%68%65%6c%6c%6f");
    assert_eq!(url_encode(&[0, 1, 2, 3]), "%00%01%02%03");
}
