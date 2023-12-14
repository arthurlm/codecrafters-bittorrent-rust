use bittorrent_starter_rust::error::TorrentError;

#[test]
fn test_derive() {
    // Debug
    assert_eq!(
        format!("{:?}", TorrentError::Http("foo".to_string())),
        "Http(\"foo\")"
    );

    // Display + Error
    assert_eq!(
        format!("{}", TorrentError::Http("foo".to_string())),
        "HTTP: foo"
    );
}
