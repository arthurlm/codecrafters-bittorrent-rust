use std::collections::BTreeMap;

use hex::ToHex;
use serde::Deserialize;
use sha1::{Digest, Sha1};

use crate::bencode_format::*;

#[derive(Debug, Deserialize)]
pub struct MetaInfoFile {
    pub announce: String,
    pub info: InfoSingleFile,
    pub created_by: Option<String>,
    pub comment: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct InfoSingleFile {
    pub name: String,
    pub length: i64,
    #[serde(rename = "piece length")]
    pub piece_length: i64,
    pub pieces: Vec<u8>,
}

impl InfoSingleFile {
    pub fn info_hash(&self) -> String {
        let content = BencodeValue::Dict(BTreeMap::from([
            (
                BencodeText::new(b"name"),
                BencodeValue::Data(BencodeText::new(self.name.as_bytes())),
            ),
            (
                BencodeText::new(b"length"),
                BencodeValue::Integer(self.length),
            ),
            (
                BencodeText::new(b"piece length"),
                BencodeValue::Integer(self.piece_length),
            ),
            (
                BencodeText::new(b"pieces"),
                BencodeValue::Data(BencodeText::new(&self.pieces)),
            ),
        ]));

        let mut buf = Vec::with_capacity(512);
        content
            .encode(&mut buf)
            .expect("Fail to bencode info in memory");

        let mut hasher = Sha1::new();
        hasher.update(buf);
        hasher.finalize().encode_hex()
    }

    pub fn pieces_hashes(&self) -> Vec<String> {
        assert_eq!(self.pieces.len() % 20, 0, "pieces is not a multiple of 20");
        self.pieces
            .chunks(20)
            .map(|piece| piece.encode_hex())
            .collect()
    }
}
