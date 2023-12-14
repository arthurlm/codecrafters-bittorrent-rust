use serde::Deserialize;

use crate::{
    bencode_format::BencodeValue, error::TorrentError, torrent_file::MetaInfoFile,
    url_encode::url_encode,
};

pub async fn query(meta_info: MetaInfoFile) -> Result<TrackerResponse, TorrentError> {
    let client = reqwest::Client::new();
    let raw_data = client
        .get(format!(
            "{}?info_hash={}",
            meta_info.announce,
            url_encode(&meta_info.info.info_hash_bytes()),
            // url_encode(&hex::decode("d69f91e6b2ae4c542468d1073a71d4ea13879a7f").unwrap()), // WTF:
        ))
        .query(&[("left", meta_info.info.length.to_string())])
        .query(&[
            ("peer_id", "alemoigne-bittorrent"),
            ("port", "6881"),
            ("uploaded", "0"),
            ("downloaded", "0"),
            ("compact", "1"),
        ])
        .send()
        .await?
        .error_for_status()?
        .bytes()
        .await?;

    let (_, content) = BencodeValue::parse(&raw_data)?;
    let response = serde_json::from_value(content.into())?;

    Ok(response)
}

#[derive(Debug, Deserialize)]
pub struct TrackerResponse {
    pub interval: i64,
    #[serde(default)]
    pub peers: Vec<u8>,
}

impl TrackerResponse {
    pub fn peer_addrs(&self) -> Vec<String> {
        assert_eq!(
            self.peers.len() % 6,
            0,
            "Peers is not a multiple of 6 bytes"
        );

        self.peers
            .chunks(6)
            .map(|n| {
                format!(
                    "{}.{}.{}.{}:{}",
                    n[0],
                    n[1],
                    n[2],
                    n[3],
                    u16::from_be_bytes([n[4], n[5]])
                )
            })
            .collect()
    }
}
