use bittorrent_starter_rust::peers::PeerMessage;
use tokio::io::{BufReader, BufWriter};

#[test]
fn test_peer_message_derive() {
    // Debug
    assert_eq!(format!("{:?}", PeerMessage::Choke), "Choke");

    // PartialEq + Eq
    assert_eq!(PeerMessage::Choke, PeerMessage::Choke);
    assert_ne!(PeerMessage::Choke, PeerMessage::Interested);
}

async fn check_rw(buf: &[u8], expected: PeerMessage) {
    // Check decode
    let mut reader = BufReader::new(buf);
    let msg = PeerMessage::read(&mut reader).await.unwrap();
    assert_eq!(msg, expected);

    // Check encode
    let mut writer = BufWriter::new(Vec::new());
    msg.write(&mut writer).await.unwrap();
    assert_eq!(writer.buffer(), buf);
}

#[tokio::test]
async fn test_peer_message_rw() {
    // Basic message
    check_rw(&[0, 0, 0, 1, 0], PeerMessage::Choke).await;
    check_rw(&[0, 0, 0, 1, 1], PeerMessage::Unchoke).await;
    check_rw(&[0, 0, 0, 1, 2], PeerMessage::Interested).await;
    check_rw(&[0, 0, 0, 1, 3], PeerMessage::NotInterested).await;

    // Have
    check_rw(&[0, 0, 0, 5, 4, 0, 0, 0, 42], PeerMessage::Have(42)).await;

    // Bit fields
    check_rw(&[0, 0, 0, 1, 5], PeerMessage::BitField(vec![])).await;
    check_rw(
        &[0, 0, 0, 3, 5, 86, 72],
        PeerMessage::BitField(vec![86, 72]),
    )
    .await;
    check_rw(
        &[0, 0, 0, 4, 5, 86, 72, 2],
        PeerMessage::BitField(vec![86, 72, 2]),
    )
    .await;

    // Request
    check_rw(
        &[0, 0, 0, 13, 6, 0, 0, 0, 42, 0, 0, 0, 43, 0, 0, 0, 44],
        PeerMessage::Request {
            index: 42,
            begin: 43,
            length: 44,
        },
    )
    .await;

    // Piece
    check_rw(
        &[0, 0, 0, 12, 7, 0, 0, 0, 41, 0, 0, 0, 63, 14, 15, 16],
        PeerMessage::Piece {
            index: 41,
            begin: 63,
            block: vec![14, 15, 16],
        },
    )
    .await;

    // Cancel
    check_rw(
        &[0, 0, 0, 13, 8, 0, 0, 0, 42, 0, 0, 0, 43, 0, 0, 0, 44],
        PeerMessage::Cancel {
            index: 42,
            begin: 43,
            length: 44,
        },
    )
    .await;
}
