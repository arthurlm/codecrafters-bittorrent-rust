use bittorrent_starter_rust::torrent_file::{InfoSingleFile, MetaInfoFile};
use serde_json::json;

const TEST_PIECES: [u8; 60] = [
    239, 191, 189, 118, 239, 191, 189, 122, 42, 239, 191, 189, 239, 191, 189, 239, 191, 189, 239,
    191, 189, 107, 19, 103, 38, 239, 191, 189, 15, 239, 191, 189, 239, 191, 189, 3, 2, 45, 110, 34,
    117, 239, 191, 189, 4, 239, 191, 189, 118, 102, 86, 115, 110, 239, 191, 189, 239, 191, 189, 16,
];

#[test]
fn test_derive() {
    // Deserialize
    let meta_info: MetaInfoFile = serde_json::from_value(json!({
        "announce": "http://test.torrent.com",
        "info": {
            "name": "test.txt",
            "length": 296,
            "piece length": 312,
            "pieces": [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20],
        },
        "created_by": null,
        "comment": null,

    }))
    .unwrap();

    // Debug
    assert_eq!(format!("{meta_info:?}"), "MetaInfoFile { announce: \"http://test.torrent.com\", info: InfoSingleFile { name: \"test.txt\", length: 296, piece_length: 312, pieces: [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20] }, created_by: None, comment: None }");
}

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

#[test]
#[should_panic = "pieces is not a multiple of 20"]
fn test_bad_pieces_count() {
    let info = InfoSingleFile {
        name: "test.txt".to_string(),
        length: 296,
        piece_length: 312,
        pieces: TEST_PIECES[..43].to_vec(),
    };
    info.pieces_count();
}

#[test]
#[should_panic = "pieces is not a multiple of 20"]
fn test_bad_pieces_hashes() {
    let info = InfoSingleFile {
        name: "test.txt".to_string(),
        length: 296,
        piece_length: 312,
        pieces: TEST_PIECES[..43].to_vec(),
    };
    info.pieces_hashes();
}

#[test]
fn test_pieces_hashes() {
    let info = InfoSingleFile {
        name: "test.txt".to_string(),
        length: 296,
        piece_length: 312,
        pieces: TEST_PIECES.to_vec(),
    };

    assert_eq!(info.pieces_count(), 3);
    assert_eq!(
        info.pieces_hashes(),
        vec![
            "efbfbd76efbfbd7a2aefbfbdefbfbdefbfbdefbf",
            "bd6b136726efbfbd0fefbfbdefbfbd03022d6e22",
            "75efbfbd04efbfbd766656736eefbfbdefbfbd10"
        ]
    );
}
