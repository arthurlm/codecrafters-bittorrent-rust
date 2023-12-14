use bittorrent_starter_rust::torrent_file::InfoSingleFile;

const TEST_PIECES: [u8; 60] = [
    239, 191, 189, 118, 239, 191, 189, 122, 42, 239, 191, 189, 239, 191, 189, 239, 191, 189, 239,
    191, 189, 107, 19, 103, 38, 239, 191, 189, 15, 239, 191, 189, 239, 191, 189, 3, 2, 45, 110, 34,
    117, 239, 191, 189, 4, 239, 191, 189, 118, 102, 86, 115, 110, 239, 191, 189, 239, 191, 189, 16,
];

#[test]
fn test_info_hash() {
    let info = InfoSingleFile {
        name: "test.txt".to_string(),
        length: 296,
        piece_length: 312,
        pieces: TEST_PIECES.to_vec(),
    };

    assert_eq!(info.info_hash(), "a8d8cc6ac9e649158452dee9800c15571c491656");
}
