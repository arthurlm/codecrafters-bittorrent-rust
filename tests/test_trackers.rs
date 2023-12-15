use std::net::{IpAddr, Ipv4Addr, SocketAddr};

use bittorrent_starter_rust::trackers::TrackerResponse;
use serde_json::json;

#[test]
fn test_response_derive() {
    // Deserialize
    let response: TrackerResponse = serde_json::from_value(json!({
        "interval": 60,
        "peers": [
            // IP 1
            127, 0, 0, 1, 86, 20,
            // IP 2
            255, 10, 10, 36, 38, 42,
        ],
    }))
    .unwrap();

    // Debug
    assert_eq!(
        format!("{response:?}"),
        "TrackerResponse { interval: 60, peers: [127, 0, 0, 1, 86, 20, 255, 10, 10, 36, 38, 42] }"
    );
}

#[test]
#[should_panic = "Peers is not a multiple of 6 bytes"]
fn test_peers_invalid() {
    let response: TrackerResponse = serde_json::from_value(json!({
        "interval": 60,
        "peers": [127, 0],
    }))
    .unwrap();

    response.peer_addrs();
}

#[test]
fn test_peers_valid() {
    let response: TrackerResponse = serde_json::from_value(json!({
        "interval": 60,
        "peers": [
            // IP 1
            127, 0, 0, 1, 86, 20,
            // IP 2
            255, 10, 10, 36, 38, 42,
        ],
    }))
    .unwrap();

    assert_eq!(
        response.peer_addrs(),
        vec![
            SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), (86 << 8) + 20),
            SocketAddr::new(IpAddr::V4(Ipv4Addr::new(255, 10, 10, 36)), (38 << 8) + 42)
        ]
    );
}
